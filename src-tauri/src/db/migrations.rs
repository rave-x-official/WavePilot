use rusqlite::{Connection, Result};

pub fn run(conn: &Connection) -> Result<()> {
    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version < 1 {
        run_v1(conn)?;
    }
    if version < 2 {
        run_v2(conn)?;
    }
    if version < 3 {
        run_v3(conn)?;
    }
    if version < 4 {
        run_v4(conn)?;
    }

    log::info!("Database migrations applied (current version: 4)");
    Ok(())
}

fn run_v1(conn: &Connection) -> Result<()> {
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

        INSERT INTO schema_version (version) VALUES (1);
        ",
    )?;
    log::info!("Applied migration v1");
    Ok(())
}

fn run_v2(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        ALTER TABLE projects ADD COLUMN keywords TEXT;
        ALTER TABLE projects ADD COLUMN notes TEXT NOT NULL DEFAULT '';
        ALTER TABLE projects ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0;

        CREATE TABLE IF NOT EXISTS project_tags (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            tag TEXT NOT NULL COLLATE NOCASE,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            UNIQUE(project_id, tag)
        );

        CREATE INDEX IF NOT EXISTS idx_project_tags_project ON project_tags(project_id);
        CREATE INDEX IF NOT EXISTS idx_project_tags_tag ON project_tags(tag);
        CREATE INDEX IF NOT EXISTS idx_projects_favorite ON projects(favorite);
        CREATE INDEX IF NOT EXISTS idx_projects_bpm ON projects(bpm);

        INSERT INTO schema_version (version) VALUES (2);
        ",
    )?;
    log::info!("Applied migration v2 (keywords, notes, favorite, normalized tags)");
    Ok(())
}

fn run_v3(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS backup_directories (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            label TEXT,
            recursive INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS backup_excluded_paths (
            id TEXT PRIMARY KEY,
            directory_id TEXT NOT NULL,
            pattern TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (directory_id) REFERENCES backup_directories(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS backup_scan_history (
            id TEXT PRIMARY KEY,
            directory_id TEXT NOT NULL,
            scanned_at TEXT NOT NULL,
            total_files INTEGER NOT NULL DEFAULT 0,
            files_deleted INTEGER NOT NULL DEFAULT 0,
            space_freed_bytes INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'completed',
            error TEXT,
            FOREIGN KEY (directory_id) REFERENCES backup_directories(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_backup_history_date ON backup_scan_history(scanned_at);
        CREATE INDEX IF NOT EXISTS idx_backup_excluded_dir ON backup_excluded_paths(directory_id);

        INSERT INTO schema_version (version) VALUES (3);
        ",
    )?;
    log::info!("Applied migration v3 (backup cleaner tables)");
    Ok(())
}

fn run_v4(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        ALTER TABLE lyrics ADD COLUMN title TEXT;
        ALTER TABLE lyrics ADD COLUMN section TEXT;
        ALTER TABLE lyrics ADD COLUMN language TEXT;

        ALTER TABLE projects ADD COLUMN description TEXT NOT NULL DEFAULT '';

        CREATE INDEX IF NOT EXISTS idx_lyrics_project ON lyrics(project_id);
        CREATE INDEX IF NOT EXISTS idx_checklists_project ON release_checklists(project_id);
        CREATE INDEX IF NOT EXISTS idx_analysis_project ON analysis_cache(project_id);

        INSERT INTO schema_version (version) VALUES (4);
        ",
    )?;
    log::info!("Applied migration v4 (lyrics columns, project description, indexes)");
    Ok(())
}
