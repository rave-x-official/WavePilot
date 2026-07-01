use crate::db::Database;
use crate::db::schema::Project;
use crate::models::project::{ImportProjectRequest, ListProjectsQuery, UpdateProjectRequest};
use crate::services::project_service;
use tauri::State;

#[tauri::command]
pub fn import_project(db: State<Database>, req: ImportProjectRequest) -> Result<Project, String> {
    log::info!("Importing project: {}", req.name);
    project_service::import_project(&db, req)
}

#[tauri::command]
pub fn list_projects(
    db: State<Database>,
    query: ListProjectsQuery,
) -> Result<Vec<Project>, String> {
    project_service::list_projects(&db, query)
}

#[tauri::command]
pub fn get_project(db: State<Database>, id: String) -> Result<Project, String> {
    project_service::get_project_by_id(&db, &id)
}

#[tauri::command]
pub fn update_project(db: State<Database>, req: UpdateProjectRequest) -> Result<Project, String> {
    log::info!("Updating project: {}", req.id);
    project_service::update_project(&db, req)
}

#[tauri::command]
pub fn toggle_favorite(db: State<Database>, id: String) -> Result<bool, String> {
    log::info!("Toggling favorite for project: {}", id);
    project_service::toggle_favorite(&db, &id)
}

#[tauri::command]
pub fn delete_project(db: State<Database>, id: String) -> Result<(), String> {
    log::info!("Deleting project: {}", id);
    project_service::delete_project(&db, &id)
}

#[tauri::command]
pub fn check_project_exists(db: State<Database>, path: String) -> Result<bool, String> {
    project_service::project_exists_by_path(&db, &path)
}

#[tauri::command]
pub fn get_project_tags(db: State<Database>, project_id: String) -> Result<Vec<String>, String> {
    project_service::get_project_tags(&db, &project_id)
}
