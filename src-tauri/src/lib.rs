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
        // Remember window size/position/maximized across launches. Visibility
        // stays out: the window is configured hidden and the front end shows
        // it after first paint (no white startup flash).
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::all()
                        & !tauri_plugin_window_state::StateFlags::VISIBLE,
                )
                .build(),
        )
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
            // The question catalog is a first-class bundled resource, loaded at
            // startup like presets and test-packs. Built-ins and the interactive
            // importer were removed; this is the library's question source. See
            // `services::catalog`. It lives in its own subdirectory (not the
            // resources root) so the bundler can reference it as a plain
            // directory path — a glob with zero matches hard-fails the Tauri
            // build, and the dev scrape is gitignored (CI never has it).
            let catalog_dir = resources_dir.join("catalog");
            let store = services::problem_store::ProblemStore::empty();
            match services::catalog::load_all(&packs, &presets, &catalog_dir) {
                Ok(problems) if !problems.is_empty() => {
                    let verified = problems.iter().filter(|p| p.judge.is_some()).count();
                    log::info!(
                        "catalog: {} problems ({} mapped to verified packs) from {}",
                        problems.len(),
                        verified,
                        catalog_dir.display()
                    );
                    store.set_catalog_problems(problems);
                }
                Ok(_) => log::warn!("no catalog resource bundled — library is empty"),
                Err(e) => log::error!("catalog load failed: {e}"),
            }
            let db = services::db::Db::open(&app.path().app_data_dir()?)?;
            store.set_user_problems(services::db::user_problems::list(&db)?);
            // Course content (LESSON_COURSE_DESIGN.md §6.4): validated
            // fail-closed, like presets — a malformed curriculum/unit/lesson
            // file aborts startup rather than loading partial content.
            let curriculum = services::curriculum::CurriculumStore::load(&resources_dir, &packs)?;
            let runtimes = services::runtime_detect::detect();
            app.manage(state::AppState::new(
                store, presets, packs, db, curriculum, runtimes,
            ));
            // Safety net: the front end shows the hidden window after first
            // paint; if it ever fails to boot, reveal the window anyway so
            // the app can't become an invisible zombie process.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(5));
                if let Some(win) = handle.get_webview_window("main") {
                    let _ = win.show();
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::problems::list_problems,
            commands::problems::get_problem,
            commands::course::get_curriculum,
            commands::course::get_unit,
            commands::course::get_lesson,
            commands::course::get_quiz,
            commands::course::get_pattern_pool,
            commands::course::submit_quiz,
            commands::course::record_lesson_progress,
            commands::course::get_lesson_progress,
            commands::course::get_progression,
            commands::course::evaluate_gate,
            commands::course::get_capstone,
            commands::course::evaluate_capstone,
            commands::course::get_placement,
            commands::course::apply_placement,
            commands::course::get_readiness,
            commands::course::get_review_queue,
            commands::course::record_review,
            commands::runner::run_code,
            commands::runner::submit_code,
            commands::runner::analyze_complexity,
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
