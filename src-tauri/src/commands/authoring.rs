//! IPC for the Create page (`validateUserProblem` / `saveUserProblem` /
//! draft functions in `src/lib/api/tauri.ts`). Validation re-runs
//! server-side on save — commands never trust the form. The reference
//! solution executes inside the sandboxed runner like any user code.

use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::domain::draft::{
    self, DraftSummary, UserProblemDraft, ValidationIssue, ValidationResult,
};
use crate::domain::problem::{Problem, ProblemSource};
use crate::domain::run::{Language, RunStatus};
use crate::error::{AppError, AppResult};
use crate::services::db::{self, drafts, user_problems};
use crate::services::{import_export, runner};
use crate::state::AppState;

/// Schema checks + (when a reference solution exists and a runtime is
/// available) a sandboxed run of that solution against ALL test cases
/// (spec §8.1 step 3). No reference solution → `ok` with no `caseResults`;
/// the UI shows "saved without verification".
#[tauri::command]
pub async fn validate_user_problem(
    state: State<'_, AppState>,
    draft: UserProblemDraft,
) -> AppResult<ValidationResult> {
    log::debug!("validate_user_problem: {}", draft.title);
    let issues = draft::validate(&draft);
    if !issues.is_empty() {
        return Ok(ValidationResult {
            ok: false,
            issues,
            case_results: None,
        });
    }

    let solution = [
        (Language::Python, draft.reference_solution.python.clone()),
        (
            Language::Javascript,
            draft.reference_solution.javascript.clone(),
        ),
    ]
    .into_iter()
    .find_map(|(lang, src)| src.filter(|s| !s.trim().is_empty()).map(|s| (lang, s)));

    let Some((language, source)) = solution else {
        return Ok(ValidationResult {
            ok: true,
            issues: vec![],
            case_results: None,
        });
    };
    let Some(program) = state.runtime_path(language) else {
        log::warn!("no runtime for {language:?}; saving without verification");
        return Ok(ValidationResult {
            ok: true,
            issues: vec![],
            case_results: None,
        });
    };

    let problem = draft::build_problem(&draft, "validation-transient", 0)?;
    let result = tauri::async_runtime::spawn_blocking(move || {
        runner::execute_with_program(&problem, language, &source, true, &program)
    })
    .await
    .map_err(|e| AppError::Runner(format!("validation task panicked: {e}")))??;

    match result.status {
        RunStatus::Pass | RunStatus::Fail => Ok(ValidationResult {
            ok: true,
            issues: vec![],
            case_results: Some(result.cases),
        }),
        RunStatus::Error | RunStatus::Timeout => Ok(ValidationResult {
            ok: false,
            issues: vec![ValidationIssue {
                field: "Reference solution".into(),
                message: result
                    .error
                    .unwrap_or_else(|| "failed to run against the test cases".into()),
            }],
            case_results: None,
        }),
    }
}

#[tauri::command]
pub fn save_user_problem(state: State<AppState>, draft: UserProblemDraft) -> AppResult<Problem> {
    log::debug!("save_user_problem: {}", draft.title);
    let issues = draft::validate(&draft);
    if let Some(first) = issues.first() {
        return Err(AppError::Validation(format!(
            "{} {}",
            first.field, first.message
        )));
    }

    let (id, number) = match &draft.id {
        Some(existing_id) => {
            let existing = state
                .problems
                .get(existing_id)
                .ok_or_else(|| AppError::NotFound(format!("Problem not found: {existing_id}")))?;
            if existing.source == ProblemSource::BuiltIn {
                return Err(AppError::Validation(
                    "Built-in problems can't be edited.".into(),
                ));
            }
            (existing_id.clone(), existing.number)
        }
        None => {
            let millis = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let mut candidate = draft::new_problem_id(&draft.title, millis);
            let mut bump = 1;
            while state.problems.get(&candidate).is_some() {
                candidate = draft::new_problem_id(&draft.title, millis + bump);
                bump += 1;
            }
            (candidate, state.problems.max_number() + 1)
        }
    };

    let problem = draft::build_problem(&draft, &id, number)?;
    user_problems::upsert(&state.db, &problem, &db::now_local_iso())?;
    state
        .problems
        .set_user_problems(user_problems::list(&state.db)?);
    Ok(problem)
}

#[tauri::command]
pub fn save_draft(
    state: State<AppState>,
    draft: UserProblemDraft,
    draft_id: Option<String>,
) -> AppResult<String> {
    let id = draft_id.unwrap_or_else(|| {
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        format!("draft-{millis}")
    });
    let json = serde_json::to_string(&draft)
        .map_err(|e| AppError::Storage(format!("failed to encode draft: {e}")))?;
    drafts::save(&state.db, &id, &json, &db::now_local_iso())?;
    Ok(id)
}

#[tauri::command]
pub fn list_drafts(state: State<AppState>) -> AppResult<Vec<DraftSummary>> {
    Ok(drafts::list(&state.db)?
        .into_iter()
        .map(|(id, json, updated_at)| {
            let title = serde_json::from_str::<UserProblemDraft>(&json)
                .map(|d| d.title)
                .ok()
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| "Untitled draft".into());
            DraftSummary {
                id,
                title,
                updated_at,
            }
        })
        .collect())
}

#[tauri::command]
pub fn get_draft(state: State<AppState>, id: String) -> AppResult<Option<UserProblemDraft>> {
    match drafts::get(&state.db, &id)? {
        Some(json) => serde_json::from_str(&json)
            .map(Some)
            .map_err(|e| AppError::Storage(format!("corrupt draft record: {e}"))),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn delete_draft(state: State<AppState>, id: String) -> AppResult<()> {
    drafts::delete(&state.db, &id)
}

/// Exports a user/imported problem to a JSON file via the OS save dialog.
/// Returns `false` (no error) when the user cancels. Built-ins ship with
/// the app — exporting them is pointless and muddies licensing.
#[tauri::command]
pub async fn export_problem(app: AppHandle, id: String) -> AppResult<bool> {
    log::debug!("export_problem: {id}");
    let state = app.state::<AppState>();
    let problem = state
        .problems
        .get(&id)
        .ok_or_else(|| AppError::NotFound(format!("Problem not found: {id}")))?;
    if problem.source == ProblemSource::BuiltIn {
        return Err(AppError::Validation(
            "Built-in problems ship with Anvil and can't be exported.".into(),
        ));
    }
    let json = import_export::export_envelope(&problem, &db::now_local_iso())?;

    let dialog = app.dialog().clone();
    let file_name = format!("{id}.anvil.json");
    let picked = tauri::async_runtime::spawn_blocking(move || {
        dialog
            .file()
            .set_file_name(&file_name)
            .add_filter("Anvil problem", &["json"])
            .blocking_save_file()
    })
    .await
    .map_err(|e| AppError::Runner(format!("dialog task panicked: {e}")))?;

    let Some(path) = picked else {
        return Ok(false); // user cancelled — not an error
    };
    let path = path
        .into_path()
        .map_err(|e| AppError::Validation(format!("unsupported save location: {e}")))?;
    std::fs::write(&path, json)?;
    log::info!("exported {id} to {}", path.display());
    Ok(true)
}

/// Imports a single-problem file OR a multi-problem pack via the OS open
/// dialog (autodetected): full validation per problem, id collisions
/// suffixed, `source: "imported"`, fresh local numbers. Returns the stored
/// problems (the UI navigates to the first) or `None` on cancel.
#[tauri::command]
pub async fn import_problems(app: AppHandle) -> AppResult<Option<Vec<Problem>>> {
    log::debug!("import_problems");
    let dialog = app.dialog().clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        dialog
            .file()
            .add_filter("Anvil problem or pack", &["json"])
            .blocking_pick_file()
    })
    .await
    .map_err(|e| AppError::Runner(format!("dialog task panicked: {e}")))?;

    let Some(path) = picked else {
        return Ok(None); // user cancelled
    };
    let path = path
        .into_path()
        .map_err(|e| AppError::Validation(format!("unsupported file location: {e}")))?;
    let json = std::fs::read_to_string(&path)?;

    let state = app.state::<AppState>();
    let problems = import_export::parse_any_import(
        &json,
        |id| state.problems.get(id).is_some(),
        state.problems.max_number() + 1,
    )?;
    let now = db::now_local_iso();
    for problem in &problems {
        user_problems::upsert(&state.db, problem, &now)?;
    }
    state
        .problems
        .set_user_problems(user_problems::list(&state.db)?);
    log::info!(
        "imported {} problem(s) from {}",
        problems.len(),
        path.display()
    );
    Ok(Some(problems))
}

/// Exports every user-authored + imported problem as one shareable pack via
/// the OS save dialog (built-ins ship with the app, so they're excluded).
/// Returns `false` when the user cancels or there's nothing to export.
#[tauri::command]
pub async fn export_pack(app: AppHandle) -> AppResult<bool> {
    log::debug!("export_pack");
    let state = app.state::<AppState>();
    let problems: Vec<Problem> = state
        .problems
        .all()
        .into_iter()
        .filter(|p| p.source != ProblemSource::BuiltIn)
        .collect();
    if problems.is_empty() {
        return Err(AppError::Validation(
            "There are no user or imported problems to export yet.".into(),
        ));
    }
    let json = import_export::export_pack("Anvil problem pack", &problems, &db::now_local_iso())?;

    let dialog = app.dialog().clone();
    let count = problems.len();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        dialog
            .file()
            .set_file_name("problems.anvilpack.json")
            .add_filter("Anvil pack", &["json"])
            .blocking_save_file()
    })
    .await
    .map_err(|e| AppError::Runner(format!("dialog task panicked: {e}")))?;

    let Some(path) = picked else {
        return Ok(false);
    };
    let path = path
        .into_path()
        .map_err(|e| AppError::Validation(format!("unsupported save location: {e}")))?;
    std::fs::write(&path, json)?;
    log::info!("exported {count} problem(s) to {}", path.display());
    Ok(true)
}
