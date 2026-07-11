//! Application-wide error type. Every Tauri command returns
//! `AppResult<T>`; `AppError` serializes as `{ "kind": "...", "message": "..." }`
//! so the frontend can surface `message` in toasts and branch on `kind` if it
//! ever needs to. Add variants (and `From` impls) here as tasks require them —
//! never a second error type.

use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("runner error: {0}")]
    Runner(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Storage(e.to_string())
    }
}

impl AppError {
    /// Stable machine-readable discriminant exposed to the frontend.
    pub fn kind(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "not-found",
            AppError::Validation(_) => "validation",
            AppError::Runner(_) => "runner",
            AppError::Storage(_) => "storage",
            AppError::Io(_) => "io",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("kind", self.kind())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
