use crate::db::Database;
use crate::db::schema::ReleaseChecklist;
use crate::models::releases::{
    AddChecklistItemRequest, CreateChecklistRequest, RemoveChecklistItemRequest,
    UpdateChecklistItemRequest,
};
use crate::services::releases_service;
use tauri::State;

#[tauri::command]
pub fn create_checklist(
    db: State<Database>,
    req: CreateChecklistRequest,
) -> Result<ReleaseChecklist, String> {
    log::info!("Creating checklist for project: {}", req.project_id);
    releases_service::create_checklist(&db, req)
}

#[tauri::command]
pub fn get_checklist(db: State<Database>, id: String) -> Result<ReleaseChecklist, String> {
    releases_service::get_checklist(&db, &id)
}

#[tauri::command]
pub fn get_checklist_for_project(
    db: State<Database>,
    project_id: String,
) -> Result<Option<ReleaseChecklist>, String> {
    releases_service::get_checklist_for_project(&db, &project_id)
}

#[tauri::command]
pub fn list_checklists(db: State<Database>) -> Result<Vec<ReleaseChecklist>, String> {
    releases_service::list_checklists(&db)
}

#[tauri::command]
pub fn toggle_checklist_item(
    db: State<Database>,
    req: UpdateChecklistItemRequest,
) -> Result<ReleaseChecklist, String> {
    log::info!("Toggling checklist item: {}", req.item_id);
    releases_service::toggle_checklist_item(&db, req)
}

#[tauri::command]
pub fn add_checklist_item(
    db: State<Database>,
    req: AddChecklistItemRequest,
) -> Result<ReleaseChecklist, String> {
    log::info!("Adding checklist item to: {}", req.checklist_id);
    releases_service::add_checklist_item(&db, req)
}

#[tauri::command]
pub fn remove_checklist_item(
    db: State<Database>,
    req: RemoveChecklistItemRequest,
) -> Result<ReleaseChecklist, String> {
    log::info!("Removing checklist item: {}", req.item_id);
    releases_service::remove_checklist_item(&db, req)
}

#[tauri::command]
pub fn delete_checklist(db: State<Database>, id: String) -> Result<(), String> {
    log::info!("Deleting checklist: {}", id);
    releases_service::delete_checklist(&db, &id)
}
