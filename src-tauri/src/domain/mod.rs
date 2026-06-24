//! Pure types + rules. No `tauri::` dependencies, no IO — everything in this
//! layer unit-tests as a plain Rust crate. Serde shapes here are the IPC
//! contract and must serialize byte-for-byte like `src/lib/types.ts`; every
//! type gets a round-trip contract test.

pub mod draft;
pub mod lc_import;
pub mod pack;
pub mod preset;
pub mod problem;
pub mod progress;
pub mod run;
