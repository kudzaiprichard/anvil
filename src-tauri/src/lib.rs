//! Tauri builder wiring: plugins, managed state, and command registration.
//! Nothing else lives here — commands are declared in `commands/`, logic in
//! `services/`, types in `domain/`. Later tasks only append to
//! `generate_handler![]` and to `AppState` construction in `setup`.

pub mod commands;
pub mod domain;
pub mod error;
pub mod services;
pub mod state;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            log::info!("Anvil v{} starting", app.package_info().version);
            match app.path().app_data_dir() {
                Ok(dir) => log::info!("app data dir: {}", dir.display()),
                Err(e) => log::warn!("could not resolve app data dir: {e}"),
            }
            let resources_dir = app.path().resource_dir()?.join("resources");
            // Expose bundled pure-Python libs (e.g. sortedcontainers, which
            // leetcode.com provides) to the sandboxed Python runner: the harness
            // reads ANVIL_PYLIB and prepends it to sys.path. Spawned interpreters
            // inherit the process env, so setting it once at startup suffices.
            let pylib = resources_dir.join("pylib");
            if pylib.is_dir() {
                std::env::set_var("ANVIL_PYLIB", &pylib);
            }
            let presets =
                services::preset_store::PresetStore::load(&resources_dir.join("presets"))?;
            // Lazy: the gzip bundle opens on first lookup, not at startup.
            let packs =
                services::pack_store::PackStore::new(resources_dir.join("test-packs.json.gz"));
            // The catalog is the LeetCode questions in the user's local scrape
            // (`.docs/my_questions.json`), merged with their packs. Built-ins and
            // the interactive importer were removed; this is the single source.
            let store = services::problem_store::ProblemStore::empty();
            match services::catalog::scrape_path() {
                Some(path) => match services::catalog::load(&packs, &presets, &path) {
                    Ok(problems) => {
                        log::info!(
                            "catalog: {} LeetCode problems from {}",
                            problems.len(),
                            path.display()
                        );
                        store.set_imported_problems(problems);
                    }
                    Err(e) => log::error!("scrape load failed: {e}"),
                },
                None => log::warn!("no .docs/my_questions.json found — empty catalog"),
            }
            let db = services::db::Db::open(&app.path().app_data_dir()?)?;
            store.set_user_problems(services::db::user_problems::list(&db)?);
            let runtimes = services::runtime_detect::detect();
            app.manage(state::AppState::new(store, presets, packs, db, runtimes));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::problems::list_problems,
            commands::problems::get_problem,
            commands::runner::run_code,
            commands::runner::submit_code,
            commands::runtimes::detect_runtimes,
            commands::progress::get_progress,
            commands::progress::get_dashboard,
            commands::progress::set_problem_status,
            commands::progress::toggle_bookmark,
            commands::progress::get_problem_user_state,
            commands::authoring::validate_user_problem,
            commands::authoring::save_user_problem,
            commands::authoring::save_draft,
            commands::authoring::list_drafts,
            commands::authoring::get_draft,
            commands::authoring::delete_draft,
            commands::authoring::export_problem,
            commands::authoring::import_problems,
            commands::authoring::export_pack,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
