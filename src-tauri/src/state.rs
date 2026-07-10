//! Shared application state, managed once in `lib.rs` via `.manage()` and
//! injected into commands as `tauri::State<AppState>`. Database is added by
//! task 0008.

use std::sync::Mutex;

use crate::domain::run::Language;
use crate::services::curriculum::CurriculumStore;
use crate::services::db::Db;
use crate::services::pack_store::PackStore;
use crate::services::preset_store::PresetStore;
use crate::services::problem_store::ProblemStore;
use crate::services::runtime_detect::RuntimeInfo;

pub struct AppState {
    pub problems: ProblemStore,
    pub presets: PresetStore,
    /// Shipped test-pack bundle (lazy-loaded on first import). Read by the
    /// importer for coverage + full-tier matching.
    pub packs: PackStore,
    pub db: Db,
    /// Validated curriculum/unit/lesson content, loaded fail-closed at
    /// startup (`services::curriculum`).
    pub curriculum: CurriculumStore,
    /// Detection cache: filled at startup, refreshed by `detect_runtimes`
    /// (the Settings pane is the refresh trigger); the runner reads it.
    runtimes: Mutex<Vec<RuntimeInfo>>,
}

impl AppState {
    pub fn new(
        problems: ProblemStore,
        presets: PresetStore,
        packs: PackStore,
        db: Db,
        curriculum: CurriculumStore,
        runtimes: Vec<RuntimeInfo>,
    ) -> Self {
        Self {
            problems,
            presets,
            packs,
            db,
            curriculum,
            runtimes: Mutex::new(runtimes),
        }
    }

    pub fn set_runtimes(&self, detected: Vec<RuntimeInfo>) {
        if let Ok(mut cache) = self.runtimes.lock() {
            *cache = detected;
        }
    }

    /// Absolute interpreter path for `language`, if a compatible runtime
    /// was detected.
    pub fn runtime_path(&self, language: Language) -> Option<String> {
        let tag = match language {
            Language::Python => "Py",
            Language::Javascript => "JS",
        };
        let cache = self.runtimes.lock().ok()?;
        cache
            .iter()
            .find(|rt| rt.tag == tag && rt.found && !rt.path.is_empty())
            .map(|rt| rt.path.clone())
    }
}
