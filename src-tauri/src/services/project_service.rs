use crate::db::Database;
use crate::db::schema::Project;
use crate::models::project::{
    ImportProjectRequest, ListProjectsQuery, SortField, SortOrder, UpdateProjectRequest,
};
use crate::utils::{collect_ok, new_id, now_timestamp, resolve_canonical_path};

const PROJECT_COLUMNS: &str = "id, name, path, artist, bpm, musical_key, root_note, tags, keywords, notes, description, favorite, daw_type, last_opened, created_at, updated_at";

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
        description: row.get(10)?,
        favorite: row.get::<_, i32>(11)? != 0,
        daw_type: row.get(12)?,
        last_opened: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
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
    match field {
        SortField::Bpm | SortField::LastOpened => {
            format!("{} IS NULL, {} {}", col, col, dir)
        }
        _ => format!("{} {}", col, dir),
    }
}

fn insert_project_tags(conn: &rusqlite::Connection, project_id: &str, tags: &[String]) -> Result<(), String> {
    let mut stmt = conn
        .prepare("INSERT OR IGNORE INTO project_tags (id, project_id, tag) VALUES (?1, ?2, ?3)")
        .map_err(|e| e.to_string())?;
    for tag in tags {
        stmt.execute(rusqlite::params![new_id(), project_id, tag])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn project_exists_by_path(db: &Database, path: &str) -> Result<bool, String> {
    let conn = db.lock()?;
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
    let canonical = resolve_canonical_path(&req.path)?;

    if project_exists_by_path(db, &canonical)? {
        return Err(format!("Project already imported: {}", req.name));
    }

    let conn = db.lock()?;
    let id = new_id();
    let now = now_timestamp();

    conn.execute(
        "INSERT INTO projects (id, name, path, artist, daw_type, keywords, notes, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, req.name, canonical, req.artist, req.daw_type, req.keywords, req.notes, now, now],
    )
    .map_err(|e| e.to_string())?;

    if let Some(ref tags) = req.tags {
        insert_project_tags(&conn, &id, tags)?;
    }

    let tags_json = req.tags.as_ref().map(|t| serde_json::to_string(t).unwrap_or_default());

    Ok(Project {
        id,
        name: req.name,
        path: canonical,
        artist: req.artist,
        bpm: None,
        musical_key: None,
        root_note: None,
        tags: tags_json,
        keywords: req.keywords,
        notes: req.notes,
        description: String::new(),
        favorite: false,
        daw_type: req.daw_type,
        last_opened: Some(now.clone()),
        created_at: now.clone(),
        updated_at: now,
    })
}

fn apply_sort(mut sql: String, query: &ListProjectsQuery) -> String {
    let sort_by = query.sort_by.as_ref().unwrap_or(&SortField::DateAdded);
    let sort_order = query.sort_order.as_ref().unwrap_or(&SortOrder::Desc);
    sql.push_str(&format!(" ORDER BY {}", sort_clause(sort_by, sort_order)));
    sql
}

pub fn list_projects(db: &Database, query: ListProjectsQuery) -> Result<Vec<Project>, String> {
    let conn = db.lock()?;
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

    sql = apply_sort(sql, &query);

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let projects = collect_ok(
        stmt.query_map(param_refs.as_slice(), row_to_project)
            .map_err(|e| e.to_string())?,
    );
    Ok(projects)
}

pub fn get_project_by_id(db: &Database, id: &str) -> Result<Project, String> {
    let conn = db.lock()?;
    let sql = format!("SELECT {} FROM projects WHERE id = ?1", PROJECT_COLUMNS);
    conn.query_row(&sql, rusqlite::params![id], row_to_project)
        .map_err(|e| e.to_string())
}

/// Replaces the 11 duplicated single-field UPDATE blocks with a dynamic builder.
fn apply_updates(
    conn: &rusqlite::Connection,
    id: &str,
    now: &str,
    req: &UpdateProjectRequest,
) -> Result<(), String> {
    let mut set_parts: Vec<&str> = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if req.name.is_some() { set_parts.push("name = ?"); }
    if req.artist.is_some() { set_parts.push("artist = ?"); }
    if req.bpm.is_some() { set_parts.push("bpm = ?"); }
    if req.musical_key.is_some() { set_parts.push("musical_key = ?"); }
    if req.root_note.is_some() { set_parts.push("root_note = ?"); }
    if req.keywords.is_some() { set_parts.push("keywords = ?"); }
    if req.notes.is_some() { set_parts.push("notes = ?"); }
    if req.description.is_some() { set_parts.push("description = ?"); }
    if req.favorite.is_some() { set_parts.push("favorite = ?"); }
    if req.daw_type.is_some() { set_parts.push("daw_type = ?"); }

    if set_parts.is_empty() {
        return Ok(());
    }

    // Push values in the same order
    if let Some(ref v) = req.name { values.push(Box::new(v.clone())); }
    if let Some(ref v) = req.artist { values.push(Box::new(v.clone())); }
    if let Some(v) = req.bpm { values.push(Box::new(v)); }
    if let Some(ref v) = req.musical_key { values.push(Box::new(v.clone())); }
    if let Some(ref v) = req.root_note { values.push(Box::new(v.clone())); }
    if let Some(ref v) = req.keywords { values.push(Box::new(v.clone())); }
    if let Some(ref v) = req.notes { values.push(Box::new(v.clone())); }
    if let Some(ref v) = req.description { values.push(Box::new(v.clone())); }
    if let Some(v) = req.favorite { values.push(Box::new(v as i32)); }
    if let Some(ref v) = req.daw_type { values.push(Box::new(v.clone())); }

    // Append updated_at and id
    set_parts.push("updated_at = ?");
    values.push(Box::new(now.to_string()));
    values.push(Box::new(id.to_string()));

    let sql = format!(
        "UPDATE projects SET {} WHERE id = ?",
        set_parts.join(", ")
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, param_refs.as_slice())
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn update_project(db: &Database, req: UpdateProjectRequest) -> Result<Project, String> {
    let conn = db.lock()?;
    let now = now_timestamp();

    apply_updates(&conn, &req.id, &now, &req)?;

    // Replace tags
    if let Some(ref tags) = req.tags {
        conn.execute(
            "DELETE FROM project_tags WHERE project_id = ?1",
            rusqlite::params![req.id],
        )
        .map_err(|e| e.to_string())?;
        insert_project_tags(&conn, &req.id, tags)?;
    }

    drop(conn);
    get_project_by_id(db, &req.id)
}

pub fn toggle_favorite(db: &Database, id: &str) -> Result<bool, String> {
    let conn = db.lock()?;
    let now = now_timestamp();
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
    let conn = db.lock()?;
    conn.execute("DELETE FROM projects WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_project_tags(db: &Database, project_id: &str) -> Result<Vec<String>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare("SELECT tag FROM project_tags WHERE project_id = ?1 ORDER BY tag")
        .map_err(|e| e.to_string())?;
    let tags = collect_ok(
        stmt.query_map(rusqlite::params![project_id], |row| row.get(0))
            .map_err(|e| e.to_string())?,
    );
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
        let project = import_test_project(&db, "Test Track", "/tmp/wp-test-import", Some("Artist"));

        assert_eq!(project.name, "Test Track");
        assert_eq!(project.artist, Some("Artist".to_string()));
        assert_eq!(project.daw_type, Some("ableton".to_string()));
        assert!(!project.id.is_empty());
        assert!(!project.favorite);
        assert!(project.bpm.is_none());

        let tags = get_project_tags(&db, &project.id).expect("Should get tags");
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"electronic".to_string()));
        assert!(tags.contains(&"ambient".to_string()));
    }

    #[test]
    fn test_duplicate_detection() {
        let db = setup_db();
        import_test_project(&db, "Project A", "/tmp/wp-test-dup-a", None);

        let result = import_project(
            &db,
            ImportProjectRequest {
                name: "Duplicate".to_string(),
                path: "/tmp/wp-test-dup-a".to_string(),
                artist: None,
                daw_type: None,
                tags: None,
                keywords: None,
                notes: None,
            },
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already imported"));
    }

    #[test]
    fn test_path_validation() {
        let db = setup_db();
        let result = import_project(
            &db,
            ImportProjectRequest {
                name: "Ghost".to_string(),
                path: "/tmp/wp-test-nonexistent-99999".to_string(),
                artist: None,
                daw_type: None,
                tags: None,
                keywords: None,
                notes: None,
            },
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_list_projects_default_order() {
        let db = setup_db();
        import_test_project(&db, "B Project", "/tmp/wp-test-order-b", None);
        std::thread::sleep(std::time::Duration::from_millis(10));
        import_test_project(&db, "A Project", "/tmp/wp-test-order-a", None);

        let projects = list_projects(
            &db,
            ListProjectsQuery::default(),
        )
        .expect("Should list");
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].name, "A Project");
    }

    #[test]
    fn test_sort_by_name() {
        let db = setup_db();
        import_test_project(&db, "Beta", "/tmp/wp-test-name-beta", None);
        import_test_project(&db, "Alpha", "/tmp/wp-test-name-alpha", None);

        let projects = list_projects(
            &db,
            ListProjectsQuery {
                sort_by: Some(SortField::Name),
                sort_order: Some(SortOrder::Asc),
                ..Default::default()
            },
        )
        .expect("Should list");
        assert_eq!(projects[0].name, "Alpha");
        assert_eq!(projects[1].name, "Beta");
    }

    #[test]
    fn test_search_by_name() {
        let db = setup_db();
        import_test_project(&db, "My Track", "/tmp/wp-test-search1", None);
        import_test_project(&db, "Another Track", "/tmp/wp-test-search2", None);
        import_test_project(&db, "Something Else", "/tmp/wp-test-search3", None);

        let projects = list_projects(
            &db,
            ListProjectsQuery {
                search: Some("Track".to_string()),
                ..Default::default()
            },
        )
        .expect("Should search");
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_search_by_artist() {
        let db = setup_db();
        import_test_project(&db, "Song 1", "/tmp/wp-test-art1", Some("Artist1"));
        import_test_project(&db, "Song 2", "/tmp/wp-test-art2", Some("Artist2"));

        let projects = list_projects(
            &db,
            ListProjectsQuery {
                artist: Some("Artist1".to_string()),
                ..Default::default()
            },
        )
        .expect("Should search");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Song 1");
    }

    #[test]
    fn test_toggle_favorite() {
        let db = setup_db();
        let project = import_test_project(&db, "Fav Test", "/tmp/wp-test-fav", None);
        assert!(!project.favorite);

        assert!(toggle_favorite(&db, &project.id).expect("Should toggle"));
        let updated = get_project_by_id(&db, &project.id).expect("Should get");
        assert!(updated.favorite);

        assert!(!toggle_favorite(&db, &project.id).expect("Should toggle again"));
    }

    #[test]
    fn test_update_project() {
        let db = setup_db();
        let project = import_test_project(&db, "Original", "/tmp/wp-test-update", None);

        let updated = update_project(
            &db,
            UpdateProjectRequest {
                id: project.id.clone(),
                name: Some("Updated".to_string()),
                artist: None,
                bpm: Some(128.0),
                musical_key: Some("Cm".to_string()),
                root_note: None,
                tags: None,
                keywords: None,
                notes: None,
                description: None,
                favorite: None,
                daw_type: None,
            },
        )
        .expect("Should update");
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.bpm, Some(128.0));
        assert_eq!(updated.musical_key, Some("Cm".to_string()));
    }

    #[test]
    fn test_delete_project() {
        let db = setup_db();
        let project = import_test_project(&db, "Delete Me", "/tmp/wp-test-delete", None);
        delete_project(&db, &project.id).expect("Should delete");
        let projects = list_projects(&db, ListProjectsQuery::default()).expect("Should list");
        assert!(projects.is_empty());
    }

    #[test]
    fn test_favorite_only_filter() {
        let db = setup_db();
        let p1 = import_test_project(&db, "Fav 1", "/tmp/wp-test-favonly1", None);
        import_test_project(&db, "Not Fav", "/tmp/wp-test-favonly2", None);
        toggle_favorite(&db, &p1.id).expect("Should toggle");

        let projects = list_projects(
            &db,
            ListProjectsQuery {
                favorite_only: Some(true),
                ..Default::default()
            },
        )
        .expect("Should filter");
        assert_eq!(projects.len(), 1);
        assert!(projects[0].favorite);
    }

    #[test]
    fn test_reimport_with_canonical_path() {
        let db = setup_db();
        import_test_project(&db, "First", "/tmp", None);

        let result = import_project(
            &db,
            ImportProjectRequest {
                name: "Second".to_string(),
                path: "/tmp".to_string(),
                artist: None,
                daw_type: None,
                tags: None,
                keywords: None,
                notes: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_update_partial_fields_only() {
        let db = setup_db();
        let project = import_test_project(&db, "Partial", "/tmp/wp-test-partial", Some("Artist A"));

        let updated = update_project(
            &db,
            UpdateProjectRequest {
                id: project.id.clone(),
                name: None,
                artist: Some("Artist B".to_string()),
                bpm: None,
                musical_key: None,
                root_note: None,
                tags: None,
                keywords: None,
                notes: None,
                description: None,
                favorite: None,
                daw_type: None,
            },
        )
        .expect("Should update artist only");
        assert_eq!(updated.name, "Partial");
        assert_eq!(updated.artist, Some("Artist B".to_string()));
    }

    #[test]
    fn test_update_no_fields() {
        let db = setup_db();
        let project = import_test_project(&db, "No-op", "/tmp/wp-test-noop", None);

        let result = update_project(
            &db,
            UpdateProjectRequest {
                id: project.id,
                name: None,
                artist: None,
                bpm: None,
                musical_key: None,
                root_note: None,
                tags: None,
                keywords: None,
                notes: None,
                description: None,
                favorite: None,
                daw_type: None,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_by_keywords() {
        let db = setup_db();
        import_test_project(&db, "Study Track", "/tmp/wp-test-kw1", None);

        let projects = list_projects(
            &db,
            ListProjectsQuery {
                keywords: Some("chill".to_string()),
                ..Default::default()
            },
        )
        .expect("Should search by keywords");
        assert_eq!(projects.len(), 1);
    }
}
