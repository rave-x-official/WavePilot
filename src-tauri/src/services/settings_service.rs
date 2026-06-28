use crate::db::Database;
use crate::models::settings::Settings;

pub fn get_settings(db: &Database) -> Result<Settings, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut settings = Settings::default();
    for (key, value) in rows {
        match key.as_str() {
            "theme" => settings.theme = value,
            "default_backup_count" => {
                if let Ok(v) = value.parse::<i32>() {
                    settings.default_backup_count = v;
                }
            }
            "projects_directory" => settings.projects_directory = Some(value),
            "analysis_enabled" => {
                if let Ok(v) = value.parse::<bool>() {
                    settings.analysis_enabled = v;
                }
            }
            "autosave_interval_seconds" => {
                if let Ok(v) = value.parse::<u32>() {
                    settings.autosave_interval_seconds = v;
                }
            }
            _ => {}
        }
    }
    Ok(settings)
}

pub fn update_setting(db: &Database, key: &str, value: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now')) ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn reset_settings(db: &Database) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM settings", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}
