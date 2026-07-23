use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupDirectory {
    pub id: String,
    pub path: String,
    pub label: Option<String>,
    pub recursive: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddBackupDirectoryRequest {
    pub path: String,
    pub label: Option<String>,
    pub recursive: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupFileEntry {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub modified: String,
    pub parent_project: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct BackupScanProgress {
    pub current_file: String,
    pub files_found: u64,
    pub phase: ScanPhase,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ScanPhase {
    Scanning,
    Analyzing,
    Complete,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupScanResult {
    pub directory_id: String,
    pub files: Vec<BackupFileEntry>,
    pub total_files: u64,
    pub total_size_bytes: u64,
    pub skipped_count: u64,
    pub skipped_log: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CleanupPreview {
    pub directory_id: String,
    pub files_to_delete: Vec<BackupFileEntry>,
    pub total_files: u64,
    pub total_size_bytes: u64,
    pub kept_files: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecuteCleanupRequest {
    pub directory_id: String,
    pub file_paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CleanupResult {
    pub files_deleted: u64,
    pub files_failed: u64,
    pub space_freed_bytes: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupHistoryEntry {
    pub id: String,
    pub directory_id: String,
    pub directory_path: String,
    pub scanned_at: String,
    pub total_files: u64,
    pub files_deleted: u64,
    pub space_freed_bytes: u64,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupSettings {
    pub backups_to_keep: u32,
    pub min_file_age_days: u32,
    pub recursive_scan: bool,
    pub confirm_before_delete: bool,
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            backups_to_keep: 5,
            min_file_age_days: 0,
            recursive_scan: true,
            confirm_before_delete: true,
        }
    }
}
