use std::sync::Mutex;

use sqlx::SqlitePool;
use tauri::{State, async_runtime::RwLock};

/// Swappable SQLite pool. The write lock allows the backup restore flow to
/// close the pool, replace the database file and reopen it without restarting
/// the application.
pub struct DbState(RwLock<SqlitePool>);

impl DbState {
    pub fn new(pool: SqlitePool) -> Self {
        Self(RwLock::new(pool))
    }

    pub(crate) fn pool(&self) -> &RwLock<SqlitePool> {
        &self.0
    }
}

/// One-shot startup message for the frontend, such as the notice produced
/// after quarantining a corrupt database.
#[derive(Default)]
pub struct StartupNotice(Mutex<Option<String>>);

impl StartupNotice {
    pub fn set(&self, message: String) {
        if let Ok(mut guard) = self.0.lock() {
            *guard = Some(message);
        }
    }

    fn take(&self) -> Option<String> {
        self.0.lock().ok().and_then(|mut guard| guard.take())
    }
}

#[tauri::command]
pub fn take_startup_notice(state: State<'_, StartupNotice>) -> Option<String> {
    state.take()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_notice_is_consumed_once() {
        let notice = StartupNotice::default();
        notice.set("Aviso de prueba".to_string());
        assert_eq!(notice.take().as_deref(), Some("Aviso de prueba"));
        assert!(notice.take().is_none());
    }
}
