use rusqlite::{Connection, Result};

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            artist TEXT,
            bpm REAL,
            musical_key TEXT,
            root_note TEXT,
            tags TEXT,
            daw_type TEXT,
            last_opened TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS lyrics (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS backup_rules (
            id TEXT PRIMARY KEY,
            project_id TEXT,
            max_backups INTEGER NOT NULL DEFAULT 5,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS release_checklists (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            items TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS analysis_cache (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            analysis_type TEXT NOT NULL,
            result TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        INSERT OR IGNORE INTO schema_version (version) VALUES (1);
        ",
    )?;
    log::info!("Database migrations applied successfully");
    Ok(())
}
