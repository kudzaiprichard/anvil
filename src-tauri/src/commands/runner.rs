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

fn hidden_work_note(ops_class: &str, measured: &str) -> String {
    format!(
        " Your Python lines grow like ~{ops_class}, but the running time grows like \
         ~{measured}: real work is hiding inside built-ins — e.g. `x in list`, \
         `list.index`/`remove`/`insert`, slicing `a[i:j]`, or building a string \
         with `+=` in a loop. Each of those is O(n) per call."
    )
}

fn compose_space_note(
    measured: &str,
    optimal: Option<&str>,
    verdict: ComplexityVerdict,
    deep_recursion: bool,
    incomparable: Option<&str>,
) -> String {
    if let Some(stated) = incomparable {
        return format!(
            "Extra space ~{measured}. The stated optimal {stated} is in terms of other \
             variables than the input size, so it isn't compared on this single size ladder."
        );
    }
    let stack = if deep_recursion {
        " Recursion depth counts: every pending call holds a stack frame."
    } else {
        ""
    };
    match (optimal, verdict) {
        (Some(opt), ComplexityVerdict::Slower) => format!(
            "Extra space ~{measured}; the optimal uses {opt}. Look for a structure \
             you can drop or reuse in place.{stack}"
        ),
        (Some(opt), ComplexityVerdict::Optimal) => {
            format!("Extra space ~{measured}, matching the optimal {opt}.{stack}")
        }
        (Some(opt), ComplexityVerdict::Faster) => format!(
            "Extra space ~{measured}, below the stated {opt} — usually because the \
             reference keeps a structure yours avoids, or reuses the input.{stack}"
        ),
        _ => format!(
            "Extra space ~{measured} (heap allocated during the call, not counting \
             the returned answer, plus recursion depth).{stack}"
        ),
    }
}

/// Builds the learner-facing report. `reference_time` is the pack's reference
/// solution timed on the same inputs (calibrates hidden-work detection).
fn build_report(
    outcome: ProbeOutcome,
    optimal: Option<String>,
    optimal_space: Option<String>,
    reference_time: Option<Vec<complexity::ComplexitySample>>,
) -> ComplexityReport {
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
        ProbeOutcome::Samples(samples) => match complexity::classify(&samples.ops) {
            None => {
                let mut r = ComplexityReport::unavailable(
                    "Not enough signal to classify the growth — the inputs may be too small \
                     or the work too flat to measure.",
                );
                r.optimal = optimal;
                r.samples = samples.ops;
                r
            }
            Some(ops_class) => {
                // Time: the op count is the deterministic baseline; when wall
                // time grows clearly faster (than the op count AND than the
                // reference solution on the same inputs), built-ins are doing
                // unseen work and the timing class is the honest one.
                let hidden = complexity::hidden_work(
                    &samples.ops,
                    &samples.time_ns,
                    reference_time.as_deref(),
                );
                let measured = if hidden {
                    complexity::classify(&samples.time_ns)
                        .map_or(ops_class, |t| complexity::max_label(ops_class, t))
                } else {
                    ops_class
                };
                let hidden = hidden && measured != ops_class;
                let verdict = complexity::verdict(measured, optimal.as_deref());
                let mut note = compose_note(measured, optimal.as_deref(), verdict);
                if hidden {
                    note.push_str(&hidden_work_note(ops_class, measured));
                }

                // Space: heap + recursion depth, capped at the measured time
                // class (a program can't allocate more objects than it runs
                // steps — byte growth beyond that is large values, e.g. big
                // integers, not extra structure). Never capped below O(n): an
                // O(n) structure can be built entirely inside C (`set(nums)`,
                // a slice) where neither the op count nor a short timing sees
                // it. Only compared against the pack when its declared space
                // is in terms of n alone.
                let cap = complexity::max_label(measured, "O(n)");
                let space = complexity::classify_space(&samples.heap_bytes, &samples.depth)
                    .map(|s| complexity::min_label(s, cap));
                let space_cmp = optimal_space
                    .as_deref()
                    .filter(|s| complexity::only_in_n(s));
                let space_verdict = match space {
                    Some(m) => complexity::verdict(m, space_cmp),
                    None => ComplexityVerdict::Unknown,
                };
                let deep_recursion =
                    complexity::depth_class(&samples.depth).is_some_and(|d| d != "O(1)");
                let incomparable = optimal_space.as_deref().filter(|_| space_cmp.is_none());
                let space_note = space.map(|m| {
                    compose_space_note(m, space_cmp, space_verdict, deep_recursion, incomparable)
                });
                let time_samples = if hidden {
                    samples
                        .time_ns
                        .iter()
                        .map(|s| complexity::ComplexitySample {
                            n: s.n,
                            ops: (s.ops / 1000).max(1),
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                ComplexityReport {
                    available: true,
                    measured: Some(measured.to_string()),
                    optimal,
                    verdict,
                    note,
                    samples: samples.ops,
                    hidden_work: hidden,
                    time_samples,
                    space_measured: space.map(str::to_string),
                    space_optimal: optimal_space,
                    space_verdict,
                    space_note,
                }
            }
        },
    }
}

/// Deterministic complexity feedback (Phase 5).
/// Profiles the learner's *own* Python solution on growing inputs (op-count,
/// untraced timing, heap and recursion depth via the runner, no AI) and
/// compares the measured time and space classes to the pack's declared
/// optimal. The pack's reference solution is profiled on the same inputs to
/// calibrate the timing check. Never records anything — it's pure feedback,
/// and it reuses the pack's verified stress generator so it works fully offline.
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
    let optimal_space = pack.solutions.complexity.as_ref().map(|c| c.space.clone());
    let sizes = complexity_ladder(spec.size);
    let entry = pack.entry_point.python.clone();
    let io_types = pack.entry_point.io_types.clone();
    let generator = spec.generator_python.clone();
    let seed = spec.seed;
    let reference = pack.solutions.python.clone();

    let program = state.runtime_path(Language::Python).ok_or_else(|| {
        AppError::Runner("Python 3.10+ not found — see Settings → Runtime".into())
    })?;
    let code = req.code.clone();
    let (outcome, reference_time) = tauri::async_runtime::spawn_blocking(move || {
        let probe = |src: &str| {
            runner::count_ops(
                src,
                &entry,
                io_types.as_ref(),
                &generator,
                seed,
                &sizes,
                &program,
            )
        };
        let outcome = probe(&code)?;
        // Calibration only, and only when it can matter: the learner's run
        // produced timings. Timing-only mode skips the trace and heap passes.
        // If the reference can't be timed, the check falls back to comparing
        // against the learner's own op count.
        let wants_reference = matches!(&outcome, ProbeOutcome::Samples(s) if !s.time_ns.is_empty());
        let reference_time = if wants_reference {
            match runner::time_only(
                &reference,
                &entry,
                io_types.as_ref(),
                &generator,
                seed,
                &sizes,
                &program,
            ) {
                Ok(ProbeOutcome::Samples(s)) if !s.time_ns.is_empty() => Some(s.time_ns),
                _ => None,
            }
        } else {
            None
        };
        AppResult::Ok((outcome, reference_time))
    })
    .await
    .map_err(|e| AppError::Runner(format!("probe task panicked: {e}")))??;

    Ok(build_report(
        outcome,
        optimal,
        optimal_space,
        reference_time,
    ))
}
