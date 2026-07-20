//! IPC for the workspace Run/Submit buttons (`runCode` / `submitCode` in
//! `src/lib/api/tauri.ts`). Async + `spawn_blocking` so the up-to-3-second
//! subprocess never blocks the main thread. Both commands record history +
//! state (task 0008); only passing SUBMITS count toward solves/streaks.

use tauri::State;

use crate::domain::complexity::{self, ComplexityReport, ComplexityVerdict};
use crate::domain::run::{Language, RunRequest, RunResult, RunStatus};
use crate::error::{AppError, AppResult};
use crate::services::db::{self, attempts};
use crate::services::review;
use crate::services::runner::{self, ProbeOutcome};
use crate::state::AppState;

async fn run(
    state: State<'_, AppState>,
    req: RunRequest,
    include_hidden: bool,
) -> AppResult<RunResult> {
    let problem = state
        .problems
        .get(&req.id)
        .ok_or_else(|| AppError::NotFound(format!("Problem not found: {}", req.id)))?;
    let program = state.runtime_path(req.language).ok_or_else(|| {
        AppError::Runner(match req.language {
            Language::Python => "Python 3.10+ not found — see Settings → Runtime".into(),
            Language::Javascript => "Node.js 18+ not found — see Settings → Runtime".into(),
        })
    })?;
    let tier = problem.experience_tier();
    let (language, code) = (req.language, req.code.clone());
    let result = tauri::async_runtime::spawn_blocking(move || {
        runner::execute_with_program(&problem, language, &code, include_hidden, &program)
    })
    .await
    .map_err(|e| AppError::Runner(format!("runner task panicked: {e}")))??;

    // Record history + state. A DB failure must never eat the run result —
    // the user's feedback matters more than the stat; log and move on.
    let record = attempts::AttemptRecord {
        problem_id: &req.id,
        language: match req.language {
            Language::Python => "python",
            Language::Javascript => "javascript",
        },
        kind: if include_hidden { "submit" } else { "run" },
        status: match result.status {
            RunStatus::Pass => "pass",
            RunStatus::Fail => "fail",
            RunStatus::Error => "error",
            RunStatus::Timeout => "timeout",
        },
        tier,
        runtime_ms: result.runtime_ms,
        code: &req.code,
        attempted_at: &db::now_local_iso(),
    };
    if let Err(e) = attempts::record_attempt(&state.db, &record) {
        log::error!("failed to record attempt for {}: {e}", req.id);
    }

    // A passing SUBMIT of a course problem enters the FSRS spaced-review queue
    // (Phase 6): solved Stage-1 problems come back to be
    // re-solved cold. Non-course library problems are ignored by `enqueue`.
    // A DB failure here must never eat the run result — log and move on.
    if include_hidden && result.status == RunStatus::Pass {
        let now = chrono::Utc::now();
        if let Err(e) = review::enqueue(&state.curriculum, &state.db, &req.id, now) {
            log::error!("failed to enqueue review for {}: {e}", req.id);
        }
        // Partial-prereq-credit (Phase 7): solving this problem exercises the
        // patterns it builds on, so their due review cards get partial credit —
        // repetition compression that keeps review from ballooning up the ladder.
        if let Err(e) = review::partial_prereq_credit(&state.curriculum, &state.db, &req.id, now) {
            log::error!("failed to apply partial prereq credit for {}: {e}", req.id);
        }
    }

    Ok(result)
}

/// Run against VISIBLE test cases only.
#[tauri::command]
pub async fn run_code(state: State<'_, AppState>, req: RunRequest) -> AppResult<RunResult> {
    log::debug!("run_code: {} ({:?})", req.id, req.language);
    run(state, req, false).await
}

/// Run against ALL test cases (visible + hidden).
#[tauri::command]
pub async fn submit_code(state: State<'_, AppState>, req: RunRequest) -> AppResult<RunResult> {
    log::debug!("submit_code: {} ({:?})", req.id, req.language);
    run(state, req, true).await
}

/// A 4-point size ladder for the complexity probe, ending near the pack's
/// stress size but capped so an O(n²) solution still profiles in time.
fn complexity_ladder(stress_size: u64) -> Vec<u64> {
    let top = stress_size.clamp(200, 800);
    let mut sizes: Vec<u64> = [top / 8, top / 4, top / 2, top]
        .into_iter()
        .map(|n| n.max(25))
        .collect();
    sizes.dedup();
    sizes
}

fn compose_note(measured: &str, optimal: Option<&str>, verdict: ComplexityVerdict) -> String {
    match (optimal, verdict) {
        (Some(opt), ComplexityVerdict::Slower) => format!(
            "You wrote ~{measured}. The optimal here is {opt} — there's a faster \
             approach; look for repeated work you can trade memory to avoid."
        ),
        (Some(opt), ComplexityVerdict::Optimal) => {
            format!("Measured ~{measured}, matching the optimal {opt}. Nicely done.")
        }
        (Some(opt), ComplexityVerdict::Faster) => format!(
            "Measured ~{measured}, which reads faster than the stated optimal {opt} \
             — likely because the heavy lifting sits in built-ins the op-counter \
             doesn't see."
        ),
        _ => format!("Measured ~{measured} (Python operations executed as the input grows)."),
    }
}

fn build_report(outcome: ProbeOutcome, optimal: Option<String>) -> ComplexityReport {
    match outcome {
        ProbeOutcome::TooSlow => {
            let mut r = ComplexityReport::unavailable(
                "Your solution ran past the profiler's time limit on larger inputs — a \
                 strong sign it's slower than optimal here. Look for nested work you can cut.",
            );
            r.optimal = optimal;
            r
        }
        ProbeOutcome::Failed(err) => {
            ComplexityReport::unavailable(format!("Couldn't profile this run: {err}"))
        }
        ProbeOutcome::Samples(samples) => match complexity::classify(&samples) {
            None => {
                let mut r = ComplexityReport::unavailable(
                    "Not enough signal to classify the growth — the inputs may be too small \
                     or the work too flat to measure.",
                );
                r.optimal = optimal;
                r.samples = samples;
                r
            }
            Some(measured) => {
                let verdict = complexity::verdict(measured, optimal.as_deref());
                let note = compose_note(measured, optimal.as_deref(), verdict);
                ComplexityReport {
                    available: true,
                    measured: Some(measured.to_string()),
                    optimal,
                    verdict,
                    note,
                    samples,
                }
            }
        },
    }
}

/// Deterministic complexity feedback (Phase 5).
/// Profiles the learner's *own* Python solution on growing inputs (op-count via
/// the runner, no AI) and compares the measured growth class to the pack's
/// declared optimal. Never records anything — it's pure feedback, and it reuses
/// the pack's verified stress generator so it works fully offline.
#[tauri::command]
pub async fn analyze_complexity(
    state: State<'_, AppState>,
    req: RunRequest,
) -> AppResult<ComplexityReport> {
    log::debug!("analyze_complexity: {} ({:?})", req.id, req.language);
    if req.language != Language::Python {
        return Ok(ComplexityReport::unavailable(
            "Complexity analysis runs on your Python solution — switch to Python to measure it.",
        ));
    }
    let Some(pack) = state.packs.get(&req.id) else {
        return Ok(ComplexityReport::unavailable(
            "No verified pack for this problem, so its growth can't be profiled here.",
        ));
    };
    // Pick the largest stress generator as the input synthesizer.
    let Some(spec) = pack.stress.iter().max_by_key(|s| s.size) else {
        return Ok(ComplexityReport::unavailable(
            "This problem has no input generator to profile against.",
        ));
    };
    let optimal = pack.solutions.complexity.as_ref().map(|c| c.time.clone());
    let sizes = complexity_ladder(spec.size);
    let entry = pack.entry_point.python.clone();
    let io_types = pack.entry_point.io_types.clone();
    let generator = spec.generator_python.clone();
    let seed = spec.seed;

    let program = state.runtime_path(Language::Python).ok_or_else(|| {
        AppError::Runner("Python 3.10+ not found — see Settings → Runtime".into())
    })?;
    let code = req.code.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        runner::count_ops(
            &code,
            &entry,
            io_types.as_ref(),
            &generator,
            seed,
            &sizes,
            &program,
        )
    })
    .await
    .map_err(|e| AppError::Runner(format!("probe task panicked: {e}")))??;

    Ok(build_report(outcome, optimal))
}
