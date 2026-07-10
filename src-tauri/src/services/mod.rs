//! Business logic. No `tauri::` dependencies — services receive plain paths,
//! connections, and domain types so they unit-test without a Tauri runtime.
//! All sensitive work (sandboxed code execution, SQLite, filesystem) lives
//! here and only here; the WebView never touches any of it directly.

pub mod catalog;
pub mod curriculum;
pub mod db;
pub mod example_parse;
pub mod import_export;
pub mod lc_import;
pub mod pack_store;
pub mod preset_store;
pub mod problem_store;
pub mod progress;
pub mod progression;
pub mod quiz;
pub mod runner;
pub mod runtime_detect;
