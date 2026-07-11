//! Problem bank: bundled built-ins (walked from
//! `resources/problems/**/*.json`, validated — a bad file fails startup
//! loudly, naming the file) merged with user problems from SQLite. Built-ins
//! are immutable for the app's lifetime; the user set is replaced wholesale
//! after every save/import so `all()`/`get()` see one consistent library
//! with zero special-casing (spec §8.2).

use std::collections::HashSet;
use std::path::Path;
use std::sync::RwLock;

use crate::domain::problem::Problem;
use crate::error::{AppError, AppResult};

#[derive(Debug)]
pub struct ProblemStore {
    builtins: Vec<Problem>,
    users: RwLock<Vec<Problem>>,
    /// The shipped question catalog (loaded at startup from the bundled
    /// resource). A first-class part of the library, held exactly like the
    /// user set — `all()`/`get()` see one merged library with no per-source
    /// special-casing.
    catalog: RwLock<Vec<Problem>>,
}

impl ProblemStore {
    /// Loads and validates every `*.json` under `problems_dir` (recursively).
    pub fn load(problems_dir: &Path) -> AppResult<Self> {
        let mut files = Vec::new();
        collect_json_files(problems_dir, &mut files)?;
        if files.is_empty() {
            return Err(AppError::Validation(format!(
                "no problem files found under {}",
                problems_dir.display()
            )));
        }

        let mut builtins = Vec::with_capacity(files.len());
        for file in &files {
            let raw = std::fs::read_to_string(file)?;
            let problem: Problem = serde_json::from_str(&raw).map_err(|e| {
                AppError::Validation(format!("{}: invalid problem JSON: {e}", file.display()))
            })?;
            problem
                .validate_structure()
                .map_err(|msg| AppError::Validation(format!("{}: {msg}", file.display())))?;
            builtins.push(problem);
        }

        let mut ids = HashSet::new();
        let mut numbers = HashSet::new();
        for p in &builtins {
            if !ids.insert(p.id.as_str()) {
                return Err(AppError::Validation(format!(
                    "duplicate problem id '{}'",
                    p.id
                )));
            }
            if !numbers.insert(p.number) {
                return Err(AppError::Validation(format!(
                    "duplicate problem number {} (id '{}')",
                    p.number, p.id
                )));
            }
        }

        builtins.sort_by_key(|p| p.number);
        log::info!("loaded {} built-in problems", builtins.len());
        Ok(Self {
            builtins,
            users: RwLock::new(Vec::new()),
            catalog: RwLock::new(Vec::new()),
        })
    }

    /// An empty store (no built-ins). The catalog is loaded at startup from the
    /// bundled resource via `set_catalog_problems`; built-ins were removed.
    pub fn empty() -> Self {
        Self {
            builtins: Vec::new(),
            users: RwLock::new(Vec::new()),
            catalog: RwLock::new(Vec::new()),
        }
    }

    /// Replaces the user-problem set (startup load and after every save).
    pub fn set_user_problems(&self, mut problems: Vec<Problem>) {
        problems.sort_by_key(|p| p.number);
        if let Ok(mut users) = self.users.write() {
            *users = problems;
        }
    }

    /// Replaces the catalog set (startup load).
    pub fn set_catalog_problems(&self, mut problems: Vec<Problem>) {
        problems.sort_by_key(|p| p.number);
        if let Ok(mut catalog) = self.catalog.write() {
            *catalog = problems;
        }
    }

    /// Built-ins + user + catalog problems, sorted by number.
    pub fn all(&self) -> Vec<Problem> {
        let users = self.users.read().map(|u| u.clone()).unwrap_or_default();
        let catalog = self.catalog.read().map(|c| c.clone()).unwrap_or_default();
        let mut merged = Vec::with_capacity(self.builtins.len() + users.len() + catalog.len());
        merged.extend(self.builtins.iter().cloned());
        merged.extend(users);
        merged.extend(catalog);
        merged.sort_by_key(|p| p.number);
        merged
    }

    pub fn get(&self, id: &str) -> Option<Problem> {
        if let Some(p) = self.builtins.iter().find(|p| p.id == id) {
            return Some(p.clone());
        }
        if let Some(p) = self
            .users
            .read()
            .ok()
            .and_then(|users| users.iter().find(|p| p.id == id).cloned())
        {
            return Some(p);
        }
        self.catalog
            .read()
            .ok()
            .and_then(|catalog| catalog.iter().find(|p| p.id == id).cloned())
    }

    /// Highest number across the whole library — new user problems get
    /// `max + 1`, stable across edits.
    pub fn max_number(&self) -> u32 {
        self.all().iter().map(|p| p.number).max().unwrap_or(0)
    }
}

fn collect_json_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> AppResult<()> {
    if !dir.is_dir() {
        return Err(AppError::Validation(format!(
            "problem resource dir missing: {}",
            dir.display()
        )));
    }
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "json") {
            out.push(path);
        }
    }
    Ok(())
}
