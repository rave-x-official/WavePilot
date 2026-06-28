use crate::db::Database;
use crate::db::schema::Project;
use crate::models::project::{ImportProjectRequest, ProjectSearchQuery, UpdateProjectRequest};
use chrono::Utc;
use uuid::Uuid;

pub fn import_project(db: &Database, req: ImportProjectRequest) -> Result<Project, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let tags_json = req.tags.map(|t| serde_json::to_string(&t).unwrap_or_default());

    conn.execute(
        "INSERT INTO projects (id, name, path, artist, daw_type, tags, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, req.name, req.path, req.artist, req.daw_type, tags_json, now, now],
    ).map_err(|e| e.to_string())?;

    let project = Project {
        id,
        name: req.name,
        path: req.path,
        artist: req.artist,
        bpm: None,
        musical_key: None,
        root_note: None,
        tags: tags_json,
        daw_type: req.daw_type,
        last_opened: Some(now.clone()),
        created_at: now.clone(),
        updated_at: now,
    };
    Ok(project)
}

pub fn list_projects(db: &Database) -> Result<Vec<Project>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, path, artist, bpm, musical_key, root_note, tags, daw_type, last_opened, created_at, updated_at FROM projects ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                artist: row.get(3)?,
                bpm: row.get(4)?,
                musical_key: row.get(5)?,
                root_note: row.get(6)?,
                tags: row.get(7)?,
                daw_type: row.get(8)?,
                last_opened: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(projects)
}

pub fn search_projects(db: &Database, query: ProjectSearchQuery) -> Result<Vec<Project>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut sql = String::from(
        "SELECT id, name, path, artist, bpm, musical_key, root_note, tags, daw_type, last_opened, created_at, updated_at FROM projects WHERE 1=1",
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref q) = query.query {
        sql.push_str(" AND (name LIKE ?1 OR artist LIKE ?1 OR tags LIKE ?1)");
        params.push(Box::new(format!("%{}%", q)));
    }
    if let Some(ref artist) = query.artist {
        let idx = params.len() + 1;
        sql.push_str(&format!(" AND artist LIKE ?{}", idx));
        params.push(Box::new(format!("%{}%", artist)));
    }
    if let Some(bpm_min) = query.bpm_min {
        let idx = params.len() + 1;
        sql.push_str(&format!(" AND (bpm >= ?{} OR bpm IS NULL)", idx));
        params.push(Box::new(bpm_min));
    }
    if let Some(bpm_max) = query.bpm_max {
        let idx = params.len() + 1;
        sql.push_str(&format!(" AND (bpm <= ?{} OR bpm IS NULL)", idx));
        params.push(Box::new(bpm_max));
    }
    if let Some(ref key) = query.musical_key {
        let idx = params.len() + 1;
        sql.push_str(&format!(" AND musical_key = ?{}", idx));
        params.push(Box::new(key.clone()));
    }
    if let Some(ref note) = query.root_note {
        let idx = params.len() + 1;
        sql.push_str(&format!(" AND root_note = ?{}", idx));
        params.push(Box::new(note.clone()));
    }
    if let Some(ref daw) = query.daw_type {
        let idx = params.len() + 1;
        sql.push_str(&format!(" AND daw_type = ?{}", idx));
        params.push(Box::new(daw.clone()));
    }

    sql.push_str(" ORDER BY updated_at DESC");

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let projects = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                artist: row.get(3)?,
                bpm: row.get(4)?,
                musical_key: row.get(5)?,
                root_note: row.get(6)?,
                tags: row.get(7)?,
                daw_type: row.get(8)?,
                last_opened: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(projects)
}

pub fn update_project(db: &Database, req: UpdateProjectRequest) -> Result<Project, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    if let Some(name) = &req.name {
        conn.execute("UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![name, now, req.id])
            .map_err(|e| e.to_string())?;
    }
    if let Some(artist) = &req.artist {
        conn.execute("UPDATE projects SET artist = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![artist, now, req.id])
            .map_err(|e| e.to_string())?;
    }
    if let Some(bpm) = req.bpm {
        conn.execute("UPDATE projects SET bpm = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![bpm, now, req.id])
            .map_err(|e| e.to_string())?;
    }
    if let Some(key) = &req.musical_key {
        conn.execute("UPDATE projects SET musical_key = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![key, now, req.id])
            .map_err(|e| e.to_string())?;
    }
    if let Some(note) = &req.root_note {
        conn.execute("UPDATE projects SET root_note = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![note, now, req.id])
            .map_err(|e| e.to_string())?;
    }
    if let Some(tags) = &req.tags {
        let tags_json = serde_json::to_string(tags).unwrap_or_default();
        conn.execute("UPDATE projects SET tags = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![tags_json, now, req.id])
            .map_err(|e| e.to_string())?;
    }
    if let Some(daw) = &req.daw_type {
        conn.execute("UPDATE projects SET daw_type = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![daw, now, req.id])
            .map_err(|e| e.to_string())?;
    }

    let mut stmt = conn
        .prepare("SELECT id, name, path, artist, bpm, musical_key, root_note, tags, daw_type, last_opened, created_at, updated_at FROM projects WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    stmt.query_row([&req.id], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            artist: row.get(3)?,
            bpm: row.get(4)?,
            musical_key: row.get(5)?,
            root_note: row.get(6)?,
            tags: row.get(7)?,
            daw_type: row.get(8)?,
            last_opened: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })
    .map_err(|e| e.to_string())
}

pub fn delete_project(db: &Database, id: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM projects WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
