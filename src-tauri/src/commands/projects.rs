use crate::db::Database;
use crate::db::schema::Project;
use crate::models::project::{ImportProjectRequest, ProjectSearchQuery, UpdateProjectRequest};
use crate::services::project_service;
use tauri::State;

#[tauri::command]
pub fn import_project(db: State<Database>, req: ImportProjectRequest) -> Result<Project, String> {
    log::info!("Importing project: {:?}", req.name);
    project_service::import_project(&db, req)
}

#[tauri::command]
pub fn list_projects(db: State<Database>) -> Result<Vec<Project>, String> {
    project_service::list_projects(&db)
}

#[tauri::command]
pub fn search_projects(
    db: State<Database>,
    query: ProjectSearchQuery,
) -> Result<Vec<Project>, String> {
    project_service::search_projects(&db, query)
}

#[tauri::command]
pub fn update_project(db: State<Database>, req: UpdateProjectRequest) -> Result<Project, String> {
    log::info!("Updating project: {:?}", req.id);
    project_service::update_project(&db, req)
}

#[tauri::command]
pub fn delete_project(db: State<Database>, id: String) -> Result<(), String> {
    log::info!("Deleting project: {:?}", id);
    project_service::delete_project(&db, &id)
}
