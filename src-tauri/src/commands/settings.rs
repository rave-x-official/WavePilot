use crate::db::Database;
use crate::models::settings::Settings;
use crate::services::settings_service;
use tauri::State;

#[tauri::command]
pub fn get_settings(db: State<Database>) -> Result<Settings, String> {
    settings_service::get_settings(&db)
}

#[tauri::command]
pub fn update_setting(db: State<Database>, key: String, value: String) -> Result<(), String> {
    log::info!("Updating setting: {} = {}", key, value);
    settings_service::update_setting(&db, &key, &value)
}

#[tauri::command]
pub fn reset_settings(db: State<Database>) -> Result<(), String> {
    log::info!("Resetting all settings");
    settings_service::reset_settings(&db)
}
