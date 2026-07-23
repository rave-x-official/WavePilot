use crate::db::Database;
use crate::db::schema::Lyric;
use crate::models::lyrics::{CreateLyricRequest, UpdateLyricRequest};
use crate::services::lyrics_service;
use tauri::State;

#[tauri::command]
pub fn create_lyric(db: State<Database>, req: CreateLyricRequest) -> Result<Lyric, String> {
    log::info!("Creating lyric for project: {}", req.project_id);
    lyrics_service::create_lyric(&db, req)
}

#[tauri::command]
pub fn get_lyric(db: State<Database>, id: String) -> Result<Lyric, String> {
    lyrics_service::get_lyric(&db, &id)
}

#[tauri::command]
pub fn list_lyrics(db: State<Database>, project_id: String) -> Result<Vec<Lyric>, String> {
    lyrics_service::list_lyrics_for_project(&db, &project_id)
}

#[tauri::command]
pub fn update_lyric(db: State<Database>, req: UpdateLyricRequest) -> Result<Lyric, String> {
    log::info!("Updating lyric: {}", req.id);
    lyrics_service::update_lyric(&db, req)
}

#[tauri::command]
pub fn delete_lyric(db: State<Database>, id: String) -> Result<(), String> {
    log::info!("Deleting lyric: {}", id);
    lyrics_service::delete_lyric(&db, &id)
}

#[tauri::command]
pub fn search_lyrics(db: State<Database>, query: String) -> Result<Vec<Lyric>, String> {
    lyrics_service::search_lyrics(&db, &query)
}
