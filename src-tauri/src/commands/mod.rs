//! Thin IPC layer only — deserialize the invoke payload, delegate to
//! `services`, serialize the result. No business logic lives here, and no
//! module below `commands/` may contain any. Each file maps to one UI domain
//! (problems, runner, progress, authoring, runtimes) and documents its caller.

pub mod authoring;
pub mod problems;
pub mod progress;
pub mod runner;
pub mod runtimes;
