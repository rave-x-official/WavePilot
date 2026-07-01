use crate::db::Database;
use crate::models::backup::{
    AddBackupDirectoryRequest, BackupDirectory, BackupHistoryEntry, BackupScanResult,
    BackupSettings, CleanupPreview, CleanupResult, ExecuteCleanupRequest,
};
use crate::services::backup_service;
use tauri::State;

#[tauri::command]
pub fn add_backup_directory(
    db: State<Database>,
    req: AddBackupDirectoryRequest,
) -> Result<BackupDirectory, String> {
    log::info!("Adding backup directory: {}", req.path);
    backup_service::add_backup_directory(&db, req)
}

#[tauri::command]
pub fn list_backup_directories(
    db: State<Database>,
) -> Result<Vec<BackupDirectory>, String> {
    backup_service::list_backup_directories(&db)
}

#[tauri::command]
pub fn remove_backup_directory(db: State<Database>, id: String) -> Result<(), String> {
    log::info!("Removing backup directory: {}", id);
    backup_service::remove_backup_directory(&db, &id)
}

#[tauri::command]
pub fn scan_backup_directory(
    db: State<Database>,
    directory_id: String,
    min_file_age_days: u32,
) -> Result<BackupScanResult, String> {
    log::info!("Scanning backup directory: {}", directory_id);
    backup_service::scan_directory(&db, &directory_id, min_file_age_days)
}

#[tauri::command]
pub fn preview_backup_cleanup(
    scan_result: BackupScanResult,
    backups_to_keep: u32,
) -> CleanupPreview {
    backup_service::preview_cleanup(&scan_result, backups_to_keep)
}

#[tauri::command]
pub fn execute_backup_cleanup(
    db: State<Database>,
    directory_id: String,
    req: ExecuteCleanupRequest,
) -> Result<CleanupResult, String> {
    log::info!("Executing backup cleanup in directory: {}", directory_id);
    backup_service::execute_cleanup(&db, &directory_id, req)
}

#[tauri::command]
pub fn get_backup_cleanup_history(
    db: State<Database>,
) -> Result<Vec<BackupHistoryEntry>, String> {
    backup_service::get_cleanup_history(&db)
}

#[tauri::command]
pub fn get_backup_settings() -> BackupSettings {
    BackupSettings::default()
}
