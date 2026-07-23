use crate::db::Database;
use crate::db::schema::Lyric;
use crate::models::lyrics::{CreateLyricRequest, UpdateLyricRequest};
use crate::utils::{collect_ok, new_id, now_timestamp};

const LYRIC_COLUMNS: &str =
    "id, project_id, title, content, section, language, created_at, updated_at";

fn row_to_lyric(row: &rusqlite::Row) -> rusqlite::Result<Lyric> {
    Ok(Lyric {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        content: row.get(3)?,
        section: row.get(4)?,
        language: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

pub fn create_lyric(db: &Database, req: CreateLyricRequest) -> Result<Lyric, String> {
    let conn = db.lock()?;
    let id = new_id();
    let now = now_timestamp();

    conn.execute(
        "INSERT INTO lyrics (id, project_id, title, content, section, language, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, req.project_id, req.title, req.content, req.section, req.language, now, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(Lyric {
        id,
        project_id: req.project_id,
        title: req.title,
        content: req.content,
        section: req.section,
        language: req.language,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_lyric(db: &Database, id: &str) -> Result<Lyric, String> {
    let conn = db.lock()?;
    let sql = format!("SELECT {} FROM lyrics WHERE id = ?1", LYRIC_COLUMNS);
    conn.query_row(&sql, rusqlite::params![id], row_to_lyric)
        .map_err(|e| e.to_string())
}

pub fn list_lyrics_for_project(db: &Database, project_id: &str) -> Result<Vec<Lyric>, String> {
    let conn = db.lock()?;
    let sql = format!(
        "SELECT {} FROM lyrics WHERE project_id = ?1 ORDER BY created_at DESC",
        LYRIC_COLUMNS
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let lyrics = collect_ok(
        stmt.query_map(rusqlite::params![project_id], row_to_lyric)
            .map_err(|e| e.to_string())?,
    );
    Ok(lyrics)
}

pub fn update_lyric(db: &Database, req: UpdateLyricRequest) -> Result<Lyric, String> {
    let conn = db.lock()?;
    let now = now_timestamp();

    let mut set_parts: Vec<&str> = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if req.title.is_some() { set_parts.push("title = ?"); }
    if req.content.is_some() { set_parts.push("content = ?"); }
    if req.section.is_some() { set_parts.push("section = ?"); }
    if req.language.is_some() { set_parts.push("language = ?"); }

    if set_parts.is_empty() {
        return get_lyric(db, &req.id);
    }

    if let Some(ref v) = req.title { values.push(Box::new(v.clone())); }
    if let Some(ref v) = req.content { values.push(Box::new(v.clone())); }
    if let Some(ref v) = req.section { values.push(Box::new(v.clone())); }
    if let Some(ref v) = req.language { values.push(Box::new(v.clone())); }

    set_parts.push("updated_at = ?");
    values.push(Box::new(now));
    values.push(Box::new(req.id.clone()));

    let sql = format!(
        "UPDATE lyrics SET {} WHERE id = ?",
        set_parts.join(", ")
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, param_refs.as_slice())
        .map_err(|e| e.to_string())?;

    drop(conn);
    get_lyric(db, &req.id)
}

pub fn delete_lyric(db: &Database, id: &str) -> Result<(), String> {
    let conn = db.lock()?;
    conn.execute("DELETE FROM lyrics WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn search_lyrics(db: &Database, query: &str) -> Result<Vec<Lyric>, String> {
    let conn = db.lock()?;
    let pattern = format!("%{}%", query);
    let sql = format!(
        "SELECT {} FROM lyrics WHERE content LIKE ?1 OR title LIKE ?1 ORDER BY updated_at DESC",
        LYRIC_COLUMNS
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let lyrics = collect_ok(
        stmt.query_map(rusqlite::params![pattern], row_to_lyric)
            .map_err(|e| e.to_string())?,
    );
    Ok(lyrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::services::project_service;
    use crate::models::project::ImportProjectRequest;

    fn setup_db() -> Database {
        Database::new_in_memory().expect("Failed to create in-memory database")
    }

    fn create_test_project(db: &Database) -> String {
        let req = ImportProjectRequest {
            name: "Test Project".to_string(),
            path: "/tmp/wp-lyrics-test".to_string(),
            artist: None,
            daw_type: None,
            tags: None,
            keywords: None,
            notes: Some(String::new()),
        };
        let _ = std::fs::create_dir_all(&req.path);
        let project = project_service::import_project(db, req).expect("Import should succeed");
        project.id
    }

    #[test]
    fn test_create_lyric() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        let lyric = create_lyric(
            &db,
            CreateLyricRequest {
                project_id: project_id.clone(),
                title: Some("Verse 1".to_string()),
                content: "Hello world".to_string(),
                section: Some("verse".to_string()),
                language: Some("en".to_string()),
            },
        )
        .expect("Should create lyric");

        assert_eq!(lyric.project_id, project_id);
        assert_eq!(lyric.title, Some("Verse 1".to_string()));
        assert_eq!(lyric.content, "Hello world");
        assert_eq!(lyric.section, Some("verse".to_string()));
        assert!(!lyric.id.is_empty());
    }

    #[test]
    fn test_list_lyrics_for_project() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        create_lyric(
            &db,
            CreateLyricRequest {
                project_id: project_id.clone(),
                title: Some("Verse 1".to_string()),
                content: "First".to_string(),
                section: None,
                language: None,
            },
        )
        .expect("Should create");
        create_lyric(
            &db,
            CreateLyricRequest {
                project_id: project_id.clone(),
                title: Some("Chorus".to_string()),
                content: "Second".to_string(),
                section: None,
                language: None,
            },
        )
        .expect("Should create");

        let lyrics = list_lyrics_for_project(&db, &project_id).expect("Should list");
        assert_eq!(lyrics.len(), 2);
    }

    #[test]
    fn test_update_lyric() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        let lyric = create_lyric(
            &db,
            CreateLyricRequest {
                project_id,
                title: Some("Old Title".to_string()),
                content: "Old content".to_string(),
                section: None,
                language: None,
            },
        )
        .expect("Should create");

        let updated = update_lyric(
            &db,
            UpdateLyricRequest {
                id: lyric.id,
                title: Some("New Title".to_string()),
                content: Some("New content".to_string()),
                section: None,
                language: None,
            },
        )
        .expect("Should update");

        assert_eq!(updated.title, Some("New Title".to_string()));
        assert_eq!(updated.content, "New content");
    }

    #[test]
    fn test_delete_lyric() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        let lyric = create_lyric(
            &db,
            CreateLyricRequest {
                project_id: project_id.clone(),
                title: None,
                content: "Delete me".to_string(),
                section: None,
                language: None,
            },
        )
        .expect("Should create");

        delete_lyric(&db, &lyric.id).expect("Should delete");
        let lyrics = list_lyrics_for_project(&db, &project_id).expect("Should list");
        assert!(lyrics.is_empty());
    }

    #[test]
    fn test_search_lyrics() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        create_lyric(
            &db,
            CreateLyricRequest {
                project_id,
                title: Some("Verse 1".to_string()),
                content: "The quick brown fox".to_string(),
                section: None,
                language: None,
            },
        )
        .expect("Should create");

        let results = search_lyrics(&db, "fox").expect("Should search");
        assert_eq!(results.len(), 1);

        let results = search_lyrics(&db, "nothing").expect("Should search");
        assert!(results.is_empty());
    }

    #[test]
    fn test_create_lyric_minimal() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        let lyric = create_lyric(
            &db,
            CreateLyricRequest {
                project_id,
                title: None,
                content: "Just content".to_string(),
                section: None,
                language: None,
            },
        )
        .expect("Should create with minimal fields");

        assert!(lyric.title.is_none());
        assert_eq!(lyric.content, "Just content");
    }
}
