use crate::db::Database;
use crate::db::schema::Project;
use crate::models::project::{
    ImportProjectRequest, ListProjectsQuery, SortField, SortOrder, UpdateProjectRequest,
};
use chrono::Utc;
use uuid::Uuid;

const PROJECT_COLUMNS: &str = "id, name, path, artist, bpm, musical_key, root_note, tags, keywords, notes, favorite, daw_type, last_opened, created_at, updated_at";

fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        artist: row.get(3)?,
        bpm: row.get(4)?,
        musical_key: row.get(5)?,
        root_note: row.get(6)?,
        tags: row.get(7)?,
        keywords: row.get(8)?,
        notes: row.get(9)?,
        favorite: row.get::<_, i32>(10)? != 0,
        daw_type: row.get(11)?,
        last_opened: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

fn sort_clause(field: &SortField, order: &SortOrder) -> String {
    let col = match field {
        SortField::Name => "name",
        SortField::DateAdded => "created_at",
        SortField::LastOpened => "last_opened",
        SortField::Bpm => "bpm",
        SortField::Artist => "artist",
    };
    let dir = match order {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };
    // Push nulls to end for bpm/last_opened ordering
    match field {
        SortField::Bpm | SortField::LastOpened => {
            format!("{} IS NULL, {} {}", col, col, dir)
        }
        _ => format!("{} {}", col, dir),
    }
}

pub fn project_exists_by_path(db: &Database, path: &str) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM projects WHERE path = ?1",
            rusqlite::params![path],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub fn import_project(db: &Database, req: ImportProjectRequest) -> Result<Project, String> {
    let path = std::path::Path::new(&req.path);
    if !path.exists() {
        return Err(format!("Path does not exist: {}", req.path));
    }

    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;
    let canonical_str = canonical.to_str().unwrap_or(&req.path);

    if project_exists_by_path(db, canonical_str)? {
        return Err(format!("Project already imported: {}", req.name));
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO projects (id, name, path, artist, daw_type, keywords, notes, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, req.name, canonical_str, req.artist, req.daw_type, req.keywords, req.notes, now, now],
    )
    .map_err(|e| e.to_string())?;

    if let Some(tags) = &req.tags {
        let mut stmt = conn
            .prepare("INSERT OR IGNORE INTO project_tags (id, project_id, tag) VALUES (?1, ?2, ?3)")
            .map_err(|e| e.to_string())?;
        for tag in tags {
            let tid = Uuid::new_v4().to_string();
            stmt.execute(rusqlite::params![tid, id, tag])
                .map_err(|e| e.to_string())?;
        }
    }

    let tags_json = if let Some(t) = &req.tags {
        Some(serde_json::to_string(t).unwrap_or_default())
    } else {
        None
    };

    Ok(Project {
        id,
        name: req.name,
        path: canonical_str.to_string(),
        artist: req.artist,
        bpm: None,
        musical_key: None,
        root_note: None,
        tags: tags_json,
        keywords: req.keywords,
        notes: req.notes,
        favorite: false,
        daw_type: req.daw_type,
        last_opened: Some(now.clone()),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn list_projects(db: &Database, query: ListProjectsQuery) -> Result<Vec<Project>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut sql = format!("SELECT {} FROM projects WHERE 1=1", PROJECT_COLUMNS);
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref search) = query.search {
        let idx = params.len() + 1;
        let pattern = format!("%{}%", search);
        sql.push_str(&format!(
            " AND (name LIKE ?{idx} OR artist LIKE ?{idx} OR keywords LIKE ?{idx})"
        ));
        params.push(Box::new(pattern));
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
    if let Some(ref tags) = query.tags {
        if !tags.is_empty() {
            let idx = params.len() + 1;
            let like_clauses: Vec<String> = tags
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let p = idx + i;
                    format!("id IN (SELECT project_id FROM project_tags WHERE tag LIKE ?{p})")
                })
                .collect();
            sql.push_str(&format!(" AND ({})", like_clauses.join(" AND ")));
            for tag in tags {
                params.push(Box::new(tag.clone()));
            }
        }
    }
    if let Some(ref kw) = query.keywords {
        let idx = params.len() + 1;
        sql.push_str(&format!(" AND keywords LIKE ?{}", idx));
        params.push(Box::new(format!("%{}%", kw)));
    }
    if let Some(true) = query.favorite_only {
        sql.push_str(" AND favorite = 1");
    }

    let sort_by = query.sort_by.unwrap_or(SortField::DateAdded);
    let sort_order = query.sort_order.unwrap_or(SortOrder::Desc);
    sql.push_str(&format!(" ORDER BY {}", sort_clause(&sort_by, &sort_order)));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let projects = stmt
        .query_map(param_refs.as_slice(), row_to_project)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(projects)
}

pub fn get_project_by_id(db: &Database, id: &str) -> Result<Project, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let sql = format!("SELECT {} FROM projects WHERE id = ?1", PROJECT_COLUMNS);
    conn.query_row(&sql, rusqlite::params![id], row_to_project)
        .map_err(|e| e.to_string())
}

pub fn update_project(db: &Database, req: UpdateProjectRequest) -> Result<Project, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    if let Some(ref name) = req.name {
        conn.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![name, now, req.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref artist) = req.artist {
        conn.execute(
            "UPDATE projects SET artist = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![artist, now, req.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(bpm) = req.bpm {
        conn.execute(
            "UPDATE projects SET bpm = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![bpm, now, req.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref key) = req.musical_key {
        conn.execute(
            "UPDATE projects SET musical_key = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![key, now, req.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref note) = req.root_note {
        conn.execute(
            "UPDATE projects SET root_note = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![note, now, req.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref keywords) = req.keywords {
        conn.execute(
            "UPDATE projects SET keywords = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![keywords, now, req.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref notes) = req.notes {
        conn.execute(
            "UPDATE projects SET notes = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![notes, now, req.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(fav) = req.favorite {
        conn.execute(
            "UPDATE projects SET favorite = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![fav as i32, now, req.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(ref daw) = req.daw_type {
        conn.execute(
            "UPDATE projects SET daw_type = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![daw, now, req.id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Replace tags
    if let Some(ref tags) = req.tags {
        conn.execute(
            "DELETE FROM project_tags WHERE project_id = ?1",
            rusqlite::params![req.id],
        )
        .map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("INSERT INTO project_tags (id, project_id, tag) VALUES (?1, ?2, ?3)")
            .map_err(|e| e.to_string())?;
        for tag in tags {
            let tid = Uuid::new_v4().to_string();
            stmt.execute(rusqlite::params![tid, req.id, tag])
                .map_err(|e| e.to_string())?;
        }
    }

    drop(conn);
    get_project_by_id(db, &req.id)
}

pub fn toggle_favorite(db: &Database, id: &str) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE projects SET favorite = CASE WHEN favorite = 0 THEN 1 ELSE 0 END, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, id],
    )
    .map_err(|e| e.to_string())?;

    let fav: i32 = conn
        .query_row(
            "SELECT favorite FROM projects WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(fav != 0)
}

pub fn delete_project(db: &Database, id: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM projects WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_project_tags(db: &Database, project_id: &str) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT tag FROM project_tags WHERE project_id = ?1 ORDER BY tag")
        .map_err(|e| e.to_string())?;
    let tags = stmt
        .query_map(rusqlite::params![project_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn setup_db() -> Database {
        Database::new_in_memory().expect("Failed to create in-memory database")
    }

    fn import_test_project(db: &Database, name: &str, path: &str, artist: Option<&str>) -> Project {
        // Ensure the directory exists for path validation
        let _ = std::fs::create_dir_all(path);
        let req = ImportProjectRequest {
            name: name.to_string(),
            path: path.to_string(),
            artist: artist.map(String::from),
            daw_type: Some("ableton".to_string()),
            tags: Some(vec!["electronic".to_string(), "ambient".to_string()]),
            keywords: Some("chill, study".to_string()),
            notes: Some("Test project".to_string()),
        };
        import_project(db, req).expect("Import should succeed")
    }

    #[test]
    fn test_import_project() {
        let db = setup_db();
        let project = import_test_project(&db, "Test Track", "/tmp/test-project", Some("Artist"));

        assert_eq!(project.name, "Test Track");
        assert_eq!(project.artist, Some("Artist".to_string()));
        assert_eq!(project.daw_type, Some("ableton".to_string()));
        assert!(!project.id.is_empty());
        assert!(project.favorite == false);
        assert!(project.bpm.is_none());

        let tags = get_project_tags(&db, &project.id).expect("Should get tags");
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"electronic".to_string()));
        assert!(tags.contains(&"ambient".to_string()));
    }

    #[test]
    fn test_duplicate_detection() {
        let db = setup_db();
        import_test_project(&db, "Project A", "/tmp/project-a", None);

        let req = ImportProjectRequest {
            name: "Project A Duplicate".to_string(),
            path: "/tmp/project-a".to_string(),
            artist: None,
            daw_type: None,
            tags: None,
            keywords: None,
            notes: None,
        };
        let result = import_project(&db, req);
        assert!(result.is_err(), "Duplicate import should fail");
        assert!(
            result.unwrap_err().contains("already imported"),
            "Error should mention duplicate"
        );
    }

    #[test]
    fn test_path_validation() {
        let db = setup_db();
        let req = ImportProjectRequest {
            name: "Nonexistent".to_string(),
            path: "/tmp/does-not-exist-12345".to_string(),
            artist: None,
            daw_type: None,
            tags: None,
            keywords: None,
            notes: None,
        };
        let result = import_project(&db, req);
        assert!(result.is_err(), "Import of nonexistent path should fail");
        assert!(
            result.unwrap_err().contains("does not exist"),
            "Error should mention path not found"
        );
    }

    #[test]
    fn test_list_projects_default_order() {
        let db = setup_db();
        import_test_project(&db, "B Project", "/tmp/b-path", None);

        // Small delay so timestamps differ
        std::thread::sleep(std::time::Duration::from_millis(10));
        import_test_project(&db, "A Project", "/tmp/a-path", None);

        // Default: newest first
        let query = ListProjectsQuery {
            search: None,
            artist: None,
            bpm_min: None,
            bpm_max: None,
            musical_key: None,
            root_note: None,
            tags: None,
            keywords: None,
            favorite_only: None,
            sort_by: None,
            sort_order: None,
            view: None,
        };
        let projects = list_projects(&db, query).expect("Should list projects");
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].name, "A Project");
        assert_eq!(projects[1].name, "B Project");
    }

    #[test]
    fn test_sort_by_name() {
        let db = setup_db();
        import_test_project(&db, "Beta", "/tmp/beta", None);
        import_test_project(&db, "Alpha", "/tmp/alpha", None);

        let query = ListProjectsQuery {
            sort_by: Some(SortField::Name),
            sort_order: Some(SortOrder::Asc),
            ..Default::default()
        };
        let projects = list_projects(&db, query).expect("Should list projects");
        assert_eq!(projects[0].name, "Alpha");
        assert_eq!(projects[1].name, "Beta");
    }

    #[test]
    fn test_search_by_name() {
        let db = setup_db();
        import_test_project(&db, "My Track", "/tmp/my-track", None);
        import_test_project(&db, "Another Track", "/tmp/another-track", None);
        import_test_project(&db, "Something Else", "/tmp/else", None);

        let query = ListProjectsQuery {
            search: Some("Track".to_string()),
            ..Default::default()
        };
        let projects = list_projects(&db, query).expect("Should search");
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_search_by_artist() {
        let db = setup_db();
        import_test_project(&db, "Song 1", "/tmp/s1", Some("Artist1"));
        import_test_project(&db, "Song 2", "/tmp/s2", Some("Artist2"));

        let query = ListProjectsQuery {
            artist: Some("Artist1".to_string()),
            ..Default::default()
        };
        let projects = list_projects(&db, query).expect("Should search by artist");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Song 1");
    }

    #[test]
    fn test_toggle_favorite() {
        let db = setup_db();
        let project = import_test_project(&db, "Fav Test", "/tmp/fav-test", None);
        assert!(!project.favorite);

        let is_fav = toggle_favorite(&db, &project.id).expect("Should toggle");
        assert!(is_fav);

        let updated = get_project_by_id(&db, &project.id).expect("Should get");
        assert!(updated.favorite);

        let is_fav2 = toggle_favorite(&db, &project.id).expect("Should toggle again");
        assert!(!is_fav2);
    }

    #[test]
    fn test_update_project() {
        let db = setup_db();
        let project = import_test_project(&db, "Original", "/tmp/original", None);

        let req = UpdateProjectRequest {
            id: project.id.clone(),
            name: Some("Updated".to_string()),
            artist: None,
            bpm: Some(128.0),
            musical_key: Some("Cm".to_string()),
            root_note: None,
            tags: None,
            keywords: None,
            notes: None,
            favorite: None,
            daw_type: None,
        };
        let updated = update_project(&db, req).expect("Should update");
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.bpm, Some(128.0));
        assert_eq!(updated.musical_key, Some("Cm".to_string()));
    }

    #[test]
    fn test_delete_project() {
        let db = setup_db();
        let project = import_test_project(&db, "Delete Me", "/tmp/delete-me", None);

        delete_project(&db, &project.id).expect("Should delete");

        let query = ListProjectsQuery {
            ..Default::default()
        };
        let projects = list_projects(&db, query).expect("Should list");
        assert!(projects.is_empty());
    }

    #[test]
    fn test_favorite_only_filter() {
        let db = setup_db();
        let p1 = import_test_project(&db, "Fav 1", "/tmp/fav1", None);
        import_test_project(&db, "Not Fav", "/tmp/not-fav", None);

        toggle_favorite(&db, &p1.id).expect("Should toggle");

        let query = ListProjectsQuery {
            favorite_only: Some(true),
            ..Default::default()
        };
        let projects = list_projects(&db, query).expect("Should filter");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Fav 1");
        assert!(projects[0].favorite);
    }

    #[test]
    fn test_reimport_with_canonical_path() {
        let db = setup_db();
        // Import once
        import_test_project(&db, "First", "/tmp", None);

        // Try importing same canonical path
        let req = ImportProjectRequest {
            name: "Second".to_string(),
            path: "/tmp".to_string(),
            artist: None,
            daw_type: None,
            tags: None,
            keywords: None,
            notes: None,
        };
        let result = import_project(&db, req);
        assert!(
            result.is_err(),
            "Re-import of same canonical path should fail"
        );
    }
}
