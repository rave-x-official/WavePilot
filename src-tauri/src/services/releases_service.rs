use crate::db::Database;
use crate::db::schema::ReleaseChecklist;
use crate::models::releases::{
    AddChecklistItemRequest, ChecklistItem, CreateChecklistRequest, RemoveChecklistItemRequest,
    UpdateChecklistItemRequest,
};
use crate::utils::{collect_ok, new_id, now_timestamp};

const CHECKLIST_COLUMNS: &str = "id, project_id, items, created_at, updated_at";

fn row_to_checklist(row: &rusqlite::Row) -> rusqlite::Result<ReleaseChecklist> {
    Ok(ReleaseChecklist {
        id: row.get(0)?,
        project_id: row.get(1)?,
        items: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn parse_items(json: &str) -> Vec<ChecklistItem> {
    serde_json::from_str(json).unwrap_or_default()
}

fn serialize_items(items: &[ChecklistItem]) -> String {
    serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string())
}

pub fn create_checklist(db: &Database, req: CreateChecklistRequest) -> Result<ReleaseChecklist, String> {
    let conn = db.lock()?;
    let id = new_id();
    let now = now_timestamp();
    let default_items = vec![
        ChecklistItem { id: new_id(), label: "Mix finished".to_string(), done: false },
        ChecklistItem { id: new_id(), label: "Master exported".to_string(), done: false },
        ChecklistItem { id: new_id(), label: "Cover art ready".to_string(), done: false },
        ChecklistItem { id: new_id(), label: "Metadata completed".to_string(), done: false },
        ChecklistItem { id: new_id(), label: "Uploaded to distributor".to_string(), done: false },
        ChecklistItem { id: new_id(), label: "Scheduled".to_string(), done: false },
        ChecklistItem { id: new_id(), label: "Released".to_string(), done: false },
    ];
    let items_json = serialize_items(&default_items);

    conn.execute(
        "INSERT INTO release_checklists (id, project_id, items, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, req.project_id, items_json, now, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(ReleaseChecklist {
        id,
        project_id: req.project_id,
        items: items_json,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_checklist(db: &Database, id: &str) -> Result<ReleaseChecklist, String> {
    let conn = db.lock()?;
    let sql = format!(
        "SELECT {} FROM release_checklists WHERE id = ?1",
        CHECKLIST_COLUMNS
    );
    conn.query_row(&sql, rusqlite::params![id], row_to_checklist)
        .map_err(|e| e.to_string())
}

pub fn get_checklist_for_project(db: &Database, project_id: &str) -> Result<Option<ReleaseChecklist>, String> {
    let conn = db.lock()?;
    let sql = format!(
        "SELECT {} FROM release_checklists WHERE project_id = ?1 LIMIT 1",
        CHECKLIST_COLUMNS
    );
    let result = conn.query_row(&sql, rusqlite::params![project_id], row_to_checklist);
    match result {
        Ok(c) => Ok(Some(c)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn list_checklists(db: &Database) -> Result<Vec<ReleaseChecklist>, String> {
    let conn = db.lock()?;
    let sql = format!(
        "SELECT {} FROM release_checklists ORDER BY updated_at DESC",
        CHECKLIST_COLUMNS
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let checklists = collect_ok(
        stmt.query_map([], row_to_checklist)
            .map_err(|e| e.to_string())?,
    );
    Ok(checklists)
}

pub fn toggle_checklist_item(
    db: &Database,
    req: UpdateChecklistItemRequest,
) -> Result<ReleaseChecklist, String> {
    let checklist = get_checklist(db, &req.checklist_id)?;
    let mut items = parse_items(&checklist.items);

    if let Some(item) = items.iter_mut().find(|i| i.id == req.item_id) {
        item.done = req.done;
    } else {
        return Err(format!("Item {} not found in checklist", req.item_id));
    }

    let conn = db.lock()?;
    let now = now_timestamp();
    let items_json = serialize_items(&items);

    conn.execute(
        "UPDATE release_checklists SET items = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![items_json, now, req.checklist_id],
    )
    .map_err(|e| e.to_string())?;

    drop(conn);
    get_checklist(db, &req.checklist_id)
}

pub fn add_checklist_item(
    db: &Database,
    req: AddChecklistItemRequest,
) -> Result<ReleaseChecklist, String> {
    let checklist = get_checklist(db, &req.checklist_id)?;
    let mut items = parse_items(&checklist.items);

    items.push(ChecklistItem {
        id: new_id(),
        label: req.label,
        done: false,
    });

    let conn = db.lock()?;
    let now = now_timestamp();
    let items_json = serialize_items(&items);

    conn.execute(
        "UPDATE release_checklists SET items = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![items_json, now, req.checklist_id],
    )
    .map_err(|e| e.to_string())?;

    drop(conn);
    get_checklist(db, &req.checklist_id)
}

pub fn remove_checklist_item(
    db: &Database,
    req: RemoveChecklistItemRequest,
) -> Result<ReleaseChecklist, String> {
    let checklist = get_checklist(db, &req.checklist_id)?;
    let mut items = parse_items(&checklist.items);

    items.retain(|i| i.id != req.item_id);

    let conn = db.lock()?;
    let now = now_timestamp();
    let items_json = serialize_items(&items);

    conn.execute(
        "UPDATE release_checklists SET items = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![items_json, now, req.checklist_id],
    )
    .map_err(|e| e.to_string())?;

    drop(conn);
    get_checklist(db, &req.checklist_id)
}

pub fn delete_checklist(db: &Database, id: &str) -> Result<(), String> {
    let conn = db.lock()?;
    conn.execute(
        "DELETE FROM release_checklists WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::models::project::ImportProjectRequest;
    use crate::services::project_service;

    fn setup_db() -> Database {
        Database::new_in_memory().expect("Failed to create in-memory database")
    }

    fn create_test_project(db: &Database) -> String {
        let req = ImportProjectRequest {
            name: "Test Project".to_string(),
            path: "/tmp/wp-releases-test".to_string(),
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
    fn test_create_checklist() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        let checklist = create_checklist(&db, CreateChecklistRequest { project_id: project_id.clone() })
            .expect("Should create checklist");

        assert_eq!(checklist.project_id, project_id);
        let items = parse_items(&checklist.items);
        assert_eq!(items.len(), 7);
        assert!(!items[0].done);
        assert_eq!(items[0].label, "Mix finished");
    }

    #[test]
    fn test_toggle_item() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        let checklist = create_checklist(&db, CreateChecklistRequest { project_id })
            .expect("Should create");
        let items = parse_items(&checklist.items);
        let first_item_id = items[0].id.clone();

        let updated = toggle_checklist_item(
            &db,
            UpdateChecklistItemRequest {
                checklist_id: checklist.id.clone(),
                item_id: first_item_id,
                done: true,
            },
        )
        .expect("Should toggle");

        let items = parse_items(&updated.items);
        assert!(items[0].done);
    }

    #[test]
    fn test_add_item() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        let checklist = create_checklist(&db, CreateChecklistRequest { project_id })
            .expect("Should create");

        let updated = add_checklist_item(
            &db,
            AddChecklistItemRequest {
                checklist_id: checklist.id.clone(),
                label: "Custom item".to_string(),
            },
        )
        .expect("Should add item");

        let items = parse_items(&updated.items);
        assert_eq!(items.len(), 8);
        assert_eq!(items.last().unwrap().label, "Custom item");
    }

    #[test]
    fn test_remove_item() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        let checklist = create_checklist(&db, CreateChecklistRequest { project_id })
            .expect("Should create");
        let items = parse_items(&checklist.items);
        let first_item_id = items[0].id.clone();

        let updated = remove_checklist_item(
            &db,
            RemoveChecklistItemRequest {
                checklist_id: checklist.id.clone(),
                item_id: first_item_id,
            },
        )
        .expect("Should remove item");

        let items = parse_items(&updated.items);
        assert_eq!(items.len(), 6);
    }

    #[test]
    fn test_get_checklist_for_project() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        create_checklist(&db, CreateChecklistRequest { project_id: project_id.clone() })
            .expect("Should create");

        let found = get_checklist_for_project(&db, &project_id).expect("Should find");
        assert!(found.is_some());

        let not_found = get_checklist_for_project(&db, "nonexistent").expect("Should not find");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_delete_checklist() {
        let db = setup_db();
        let project_id = create_test_project(&db);
        let checklist = create_checklist(&db, CreateChecklistRequest { project_id })
            .expect("Should create");

        delete_checklist(&db, &checklist.id).expect("Should delete");
        let result = get_checklist(&db, &checklist.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_checklists() {
        let db = setup_db();
        let req1 = ImportProjectRequest {
            name: "Project A".to_string(),
            path: "/tmp/wp-releases-list-a".to_string(),
            artist: None,
            daw_type: None,
            tags: None,
            keywords: None,
            notes: Some(String::new()),
        };
        let _ = std::fs::create_dir_all(&req1.path);
        let p1 = project_service::import_project(&db, req1).expect("Import should succeed").id;

        let req2 = ImportProjectRequest {
            name: "Project B".to_string(),
            path: "/tmp/wp-releases-list-b".to_string(),
            artist: None,
            daw_type: None,
            tags: None,
            keywords: None,
            notes: Some(String::new()),
        };
        let _ = std::fs::create_dir_all(&req2.path);
        let p2 = project_service::import_project(&db, req2).expect("Import should succeed").id;

        create_checklist(&db, CreateChecklistRequest { project_id: p1 }).expect("Should create");
        create_checklist(&db, CreateChecklistRequest { project_id: p2 }).expect("Should create");

        let all = list_checklists(&db).expect("Should list");
        assert_eq!(all.len(), 2);
    }
}
