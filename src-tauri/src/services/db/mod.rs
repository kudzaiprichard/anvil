//! SQLite access — open + migrations. The connection NEVER leaves this
//! module tree: submodules (`attempts`, `problem_state`, …) expose typed
//! functions over `&Db`, and everything above them sees domain types only
//! (spec §7.2: the WebView never touches the DB; neither does anything
//! outside `services/db/`).

pub mod attempts;
pub mod drafts;
pub mod imported_problems;
pub mod lesson_progress;
pub mod mastery;
pub mod problem_state;
pub mod quiz_result;
pub mod user_problems;

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;

use crate::error::{AppError, AppResult};

/// Single-user desktop app: one connection behind a mutex is correct —
/// do not add a pool.
pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    /// Opens (creating if needed) `anvil.db` in `app_data_dir`, enables WAL
    /// + foreign keys, and applies pending migrations.
    pub fn open(app_data_dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(app_data_dir)?;
        let path = app_data_dir.join("anvil.db");
        let conn = Connection::open(&path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        migrate(&conn)?;
        log::info!("database open at {}", path.display());
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// For submodules only — see the module doc.
    pub(in crate::services::db) fn lock(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| AppError::Storage("database lock poisoned".into()))
    }
}

/// Embedded migrations applied in order, tracked in `schema_migrations`.
/// Append-only: never edit a shipped migration, add a new file.
const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("migrations/0001_init.sql")),
    (2, include_str!("migrations/0002_imported_problems.sql")),
    (3, include_str!("migrations/0003_curriculum.sql")),
    (4, include_str!("migrations/0004_gate_solve.sql")),
    (5, include_str!("migrations/0005_quiz_result.sql")),
];

fn migrate(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
           version INTEGER PRIMARY KEY,
           applied_at TEXT NOT NULL
         )",
    )?;
    for (version, sql) in MIGRATIONS {
        let applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            [version],
            |row| row.get(0),
        )?;
        if !applied {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                rusqlite::params![version, now_local_iso()],
            )?;
            log::info!("applied migration {version}");
        }
    }
    Ok(())
}

/// Local-time ISO-8601 ("2026-06-12T14:03:55.123") — streaks are calendar
/// days in the user's timezone, so local is the right clock here.
pub fn now_local_iso() -> String {
    chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%.3f")
        .to_string()
}
