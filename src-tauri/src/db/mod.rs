pub mod migrations;
pub mod schema;

use rusqlite::{Connection, Result};
use std::sync::{Mutex, MutexGuard};

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        log::info!("Opening database at: {}", db_path);
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn lock(&self) -> Result<MutexGuard<'_, Connection>, String> {
        self.conn.lock().map_err(|e| format!("Database lock error: {}", e))
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .expect("Database mutex poisoned during migration");
        migrations::run(&conn)
    }
}
