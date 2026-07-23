use crate::db::Database;
use crate::models::backup::{
    AddBackupDirectoryRequest, BackupDirectory, BackupFileEntry, BackupHistoryEntry,
    BackupScanResult, CleanupPreview, CleanupResult, ExecuteCleanupRequest,
};
use crate::utils::{collect_ok, new_id, now_timestamp, resolve_canonical_path};
use std::collections::HashMap;

const BACKUP_EXTENSIONS: &[&str] = &["bak", "backup", "old", "rpp-bak"];
const BACKUP_NAME_MARKS: &[&str] = &[
    "_backup", "-backup", " backup", "_Backup", "-Backup", "_bak", ".bak.",
];
const BACKUP_DIR_NAMES: &[&str] = &[
    "Backup", "backup", "Backups", "backups", "_Backup", "Project Backup",
];

// --- Row mapping ---

fn row_to_backup_directory(row: &rusqlite::Row) -> rusqlite::Result<BackupDirectory> {
    Ok(BackupDirectory {
        id: row.get(0)?,
        path: row.get(1)?,
        label: row.get(2)?,
        recursive: row.get::<_, i32>(3)? != 0,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn row_to_history_entry(row: &rusqlite::Row) -> rusqlite::Result<BackupHistoryEntry> {
    Ok(BackupHistoryEntry {
        id: row.get(0)?,
        directory_id: row.get(1)?,
        directory_path: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        scanned_at: row.get(3)?,
        total_files: row.get::<_, i64>(4)? as u64,
        files_deleted: row.get::<_, i64>(5)? as u64,
        space_freed_bytes: row.get::<_, i64>(6)? as u64,
        status: row.get(7)?,
        error: row.get(8)?,
    })
}

// --- Directory management ---

pub fn add_backup_directory(
    db: &Database,
    req: AddBackupDirectoryRequest,
) -> Result<BackupDirectory, String> {
    let canonical = resolve_canonical_path(&req.path)?;

    // Verify it's a directory
    if !std::path::Path::new(&canonical).is_dir() {
        return Err(format!("Path is not a directory: {}", req.path));
    }

    let conn = db.lock()?;
    let id = new_id();
    let now = now_timestamp();
    let recursive = req.recursive.unwrap_or(true);

    conn.execute(
        "INSERT INTO backup_directories (id, path, label, recursive, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, canonical, req.label, recursive as i32, now, now],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            "Directory already added".to_string()
        } else {
            e.to_string()
        }
    })?;

    Ok(BackupDirectory {
        id,
        path: canonical,
        label: req.label,
        recursive,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn list_backup_directories(db: &Database) -> Result<Vec<BackupDirectory>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare("SELECT id, path, label, recursive, created_at, updated_at FROM backup_directories ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let dirs = collect_ok(
        stmt.query_map([], row_to_backup_directory)
            .map_err(|e| e.to_string())?,
    );
    Ok(dirs)
}

pub fn remove_backup_directory(db: &Database, id: &str) -> Result<(), String> {
    let conn = db.lock()?;
    conn.execute(
        "DELETE FROM backup_directories WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// --- Scanning ---

fn is_backup_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    BACKUP_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(&format!(".{}", ext)))
        || BACKUP_NAME_MARKS.iter().any(|mark| lower.contains(mark))
}

fn is_backup_directory(name: &str) -> bool {
    BACKUP_DIR_NAMES.iter().any(|&d| name == d)
}

fn is_hidden(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

fn detect_parent_project(path: &std::path::Path) -> Option<String> {
    let parent = path.parent()?;
    let parent_name = parent.file_name()?.to_str()?;

    if is_backup_directory(parent_name) {
        return parent
            .parent()
            .and_then(|gp| gp.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());
    }

    Some(parent_name.to_string())
}

fn collect_single_file(
    path: &std::path::Path,
    min_file_age_days: u32,
    files: &mut Vec<BackupFileEntry>,
    skipped: &mut u64,
    skipped_log: &mut Vec<String>,
) {
    let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if !is_backup_file(fname) {
        return;
    }

    if min_file_age_days > 0 {
        if let Ok(metadata) = path.metadata() {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = std::time::SystemTime::now().duration_since(modified) {
                    let age_days = duration.as_secs() / 86400;
                    if (age_days as u32) < min_file_age_days {
                        return;
                    }
                }
            }
        }
    }

    let metadata = match path.metadata() {
        Ok(m) => m,
        Err(e) => {
            skipped_log.push(format!("Skipped (metadata): {} - {}", path.display(), e));
            *skipped += 1;
            return;
        }
    };

    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| {
            let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
            let secs = duration.as_secs() as i64;
            let naive = chrono::DateTime::from_timestamp(secs, 0)?;
            Some(naive.to_rfc3339())
        })
        .unwrap_or_default();

    let parent_project = detect_parent_project(path);

    files.push(BackupFileEntry {
        path: path.to_string_lossy().to_string(),
        name: fname.to_string(),
        size_bytes: metadata.len(),
        modified,
        parent_project,
    });
}

fn collect_backup_files(
    dir: &std::path::Path,
    recursive: bool,
    min_file_age_days: u32,
    skipped_log: &mut Vec<String>,
) -> (Vec<BackupFileEntry>, u64) {
    let mut files = Vec::new();
    let mut skipped = 0u64;

    if recursive {
        for entry in walkdir::WalkDir::new(dir).into_iter().flatten() {
            let path = entry.path();
            if path.is_dir() {
                if is_hidden(path) {
                    continue;
                }
            } else if path.is_file() && !is_hidden(path) {
                collect_single_file(path, min_file_age_days, &mut files, &mut skipped, skipped_log);
            }
        }
    } else if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && !is_hidden(&path) {
                collect_single_file(&path, min_file_age_days, &mut files, &mut skipped, skipped_log);
            }
        }
    }

    (files, skipped)
}

pub fn scan_directory(
    db: &Database,
    directory_id: &str,
    min_file_age_days: u32,
) -> Result<BackupScanResult, String> {
    let dirs = list_backup_directories(db)?;
    let dir = dirs
        .into_iter()
        .find(|d| d.id == directory_id)
        .ok_or_else(|| "Backup directory not found".to_string())?;

    let path = std::path::Path::new(&dir.path);
    if !path.exists() {
        return Err(format!("Directory no longer exists: {}", dir.path));
    }

    let mut skipped_log = Vec::new();
    let (files, skipped) =
        collect_backup_files(path, dir.recursive, min_file_age_days, &mut skipped_log);

    let total_size: u64 = files.iter().map(|f| f.size_bytes).sum();

    log::info!(
        "Backup scan complete: {} files found, {} skipped, {} bytes",
        files.len(),
        skipped,
        total_size
    );

    Ok(BackupScanResult {
        directory_id: directory_id.to_string(),
        total_files: files.len() as u64,
        total_size_bytes: total_size,
        files,
        skipped_count: skipped,
        skipped_log,
    })
}

// --- Cleanup preview ---

pub fn preview_cleanup(
    scan_result: &BackupScanResult,
    backups_to_keep: u32,
) -> CleanupPreview {
    let mut grouped: HashMap<String, Vec<BackupFileEntry>> = HashMap::new();

    for file in &scan_result.files {
        let project = file
            .parent_project
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        grouped.entry(project).or_default().push(file.clone());
    }

    let mut to_delete = Vec::new();
    let mut kept = 0u64;

    for (_project, mut files) in grouped {
        files.sort_by(|a, b| b.modified.cmp(&a.modified));
        let to_keep = backups_to_keep as usize;
        if files.len() > to_keep {
            kept += to_keep as u64;
            to_delete.extend(files.into_iter().skip(to_keep));
        } else {
            kept += files.len() as u64;
        }
    }

    CleanupPreview {
        directory_id: scan_result.directory_id.clone(),
        total_files: to_delete.len() as u64,
        total_size_bytes: to_delete.iter().map(|f| f.size_bytes).sum(),
        files_to_delete: to_delete,
        kept_files: kept,
    }
}

// --- Cleanup execution ---

pub fn execute_cleanup(
    db: &Database,
    directory_id: &str,
    req: ExecuteCleanupRequest,
) -> Result<CleanupResult, String> {
    let mut deleted = 0u64;
    let mut failed = 0u64;
    let mut freed_bytes = 0u64;
    let mut errors = Vec::new();

    for file_path in &req.file_paths {
        let path = std::path::Path::new(file_path);
        let size = path.metadata().ok().map(|m| m.len()).unwrap_or(0);

        match std::fs::remove_file(path) {
            Ok(()) => {
                deleted += 1;
                freed_bytes += size;
                log::info!("Deleted backup file: {}", file_path);
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: {}", file_path, e));
                log::warn!("Failed to delete {}: {}", file_path, e);
            }
        }
    }

    // Save history
    let conn = db.lock()?;
    let id = new_id();
    let now = now_timestamp();
    conn.execute(
        "INSERT INTO backup_scan_history (id, directory_id, scanned_at, total_files, files_deleted, space_freed_bytes, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, directory_id, now, deleted + failed, deleted, freed_bytes, if failed > 0 { "partial" } else { "completed" }],
    )
    .map_err(|e| e.to_string())?;

    log::info!("Cleanup done: {} deleted, {} failed, {} bytes freed", deleted, failed, freed_bytes);

    Ok(CleanupResult {
        files_deleted: deleted,
        files_failed: failed,
        space_freed_bytes: freed_bytes,
        errors,
    })
}

// --- History ---

pub fn get_cleanup_history(db: &Database) -> Result<Vec<BackupHistoryEntry>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare(
            "SELECT h.id, h.directory_id, d.path, h.scanned_at, h.total_files, h.files_deleted, h.space_freed_bytes, h.status, h.error
             FROM backup_scan_history h
             LEFT JOIN backup_directories d ON h.directory_id = d.id
             ORDER BY h.scanned_at DESC
             LIMIT 100",
        )
        .map_err(|e| e.to_string())?;

    let entries = collect_ok(
        stmt.query_map([], row_to_history_entry)
            .map_err(|e| e.to_string())?,
    );
    Ok(entries)
}

// --- Excluded paths (reserved for future UI) ---

#[allow(dead_code)]
pub fn add_excluded_path(db: &Database, directory_id: &str, pattern: &str) -> Result<(), String> {
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO backup_excluded_paths (id, directory_id, pattern, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![new_id(), directory_id, pattern, now_timestamp()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(dead_code)]
pub fn list_excluded_paths(db: &Database, directory_id: &str) -> Result<Vec<String>, String> {
    let conn = db.lock()?;
    let mut stmt = conn
        .prepare("SELECT pattern FROM backup_excluded_paths WHERE directory_id = ?1 ORDER BY created_at")
        .map_err(|e| e.to_string())?;
    let patterns = collect_ok(
        stmt.query_map(rusqlite::params![directory_id], |row| row.get(0))
            .map_err(|e| e.to_string())?,
    );
    Ok(patterns)
}

#[allow(dead_code)]
pub fn remove_excluded_path(db: &Database, directory_id: &str, pattern: &str) -> Result<(), String> {
    let conn = db.lock()?;
    conn.execute(
        "DELETE FROM backup_excluded_paths WHERE directory_id = ?1 AND pattern = ?2",
        rusqlite::params![directory_id, pattern],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn setup_db() -> Database {
        Database::new_in_memory().expect("Failed to create in-memory database")
    }

    fn create_dir(path: &str) -> std::path::PathBuf {
        let dir = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    fn touch(path: &std::path::Path) {
        std::fs::write(path, b"test").ok();
    }

    #[test]
    fn test_is_backup_file() {
        assert!(is_backup_file("project.bak"));
        assert!(is_backup_file("song.backup"));
        assert!(is_backup_file("track_Backup.als"));
        assert!(is_backup_file("song.rpp-bak"));
        assert!(!is_backup_file("project.wav"));
        assert!(!is_backup_file("track.als"));
    }

    #[test]
    fn test_add_and_list_directories() {
        let db = setup_db();
        let dir = create_dir("/tmp/wp-test-bu-add");
        let added = add_backup_directory(
            &db,
            AddBackupDirectoryRequest {
                path: dir.to_string_lossy().to_string(),
                label: Some("Test Dir".to_string()),
                recursive: Some(true),
            },
        )
        .expect("Should add");
        assert_eq!(added.label, Some("Test Dir".to_string()));

        let list = list_backup_directories(&db).expect("Should list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, added.id);
    }

    #[test]
    fn test_add_directory_not_exists() {
        let db = setup_db();
        let result = add_backup_directory(
            &db,
            AddBackupDirectoryRequest {
                path: "/tmp/wp-test-nonexistent-xxx".to_string(),
                label: None,
                recursive: Some(true),
            },
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_remove_directory() {
        let db = setup_db();
        let dir = create_dir("/tmp/wp-test-bu-remove");
        let added = add_backup_directory(
            &db,
            AddBackupDirectoryRequest {
                path: dir.to_string_lossy().to_string(),
                label: None,
                recursive: Some(true),
            },
        )
        .expect("Should add");
        remove_backup_directory(&db, &added.id).expect("Should remove");
        assert!(list_backup_directories(&db).unwrap().is_empty());
    }

    #[test]
    fn test_collect_backup_files() {
        let dir = create_dir("/tmp/wp-test-scan");
        touch(&dir.join("project.wav"));
        touch(&dir.join("project.bak"));
        touch(&dir.join("song.backup"));
        touch(&dir.join("track_Backup.als"));
        touch(&dir.join(".hidden.bak"));

        let sub = dir.join("Backup");
        std::fs::create_dir_all(&sub).ok();
        touch(&sub.join("old_version.bak"));

        let (files, _skipped) = collect_backup_files(&dir, true, 0, &mut Vec::new());
        assert_eq!(files.len(), 4);
        assert!(files.iter().any(|f| f.name == "project.bak"));
        assert!(files.iter().any(|f| f.name == "song.backup"));
        assert!(files.iter().any(|f| f.name == "old_version.bak"));

        // Non-recursive should not find files in subdirectory
        let (flat, _) = collect_backup_files(&dir, false, 0, &mut Vec::new());
        assert_eq!(flat.len(), 3); // project.bak, song.backup, track_Backup.als
    }

    #[test]
    fn test_scan_directory() {
        let db = setup_db();
        let dir = create_dir("/tmp/wp-test-scan-dir");
        touch(&dir.join("backup1.bak"));
        touch(&dir.join("backup2.bak"));
        touch(&dir.join("project.flp"));

        let added = add_backup_directory(
            &db,
            AddBackupDirectoryRequest {
                path: dir.to_string_lossy().to_string(),
                label: None,
                recursive: Some(true),
            },
        )
        .expect("Should add");

        let result = scan_directory(&db, &added.id, 0).expect("Should scan");
        assert_eq!(result.total_files, 2);
    }

    #[test]
    fn test_scan_with_min_age() {
        let db = setup_db();
        let dir = create_dir("/tmp/wp-test-scan-age");

        // Create an old file (modify mtime to the past)
        let old = dir.join("old.bak");
        touch(&old);
        let old_time = std::time::SystemTime::now()
            - std::time::Duration::from_secs(86400 * 10); // 10 days ago
        let f = std::fs::File::open(&old).ok();
        if let Some(file) = f {
            file.set_modified(old_time).ok();
        }

        let recent = dir.join("recent.bak");
        touch(&recent);

        let added = add_backup_directory(
            &db,
            AddBackupDirectoryRequest {
                path: dir.to_string_lossy().to_string(),
                label: None,
                recursive: Some(true),
            },
        )
        .expect("Should add");

        // Min age 5 days — old file (10 days) should be found, recent (0 days) excluded
        let result = scan_directory(&db, &added.id, 5).expect("Should scan");
        assert_eq!(result.total_files, 1);
    }

    #[test]
    fn test_preview_cleanup_keeps_latest() {
        let mut files = Vec::new();
        for i in 0..10 {
            let modified = format!("2024-01-{:02}T00:00:00+00:00", i + 1);
            files.push(BackupFileEntry {
                path: format!("/tmp/b{}.bak", i),
                name: format!("b{}.bak", i),
                size_bytes: 100,
                modified,
                parent_project: Some("TestProject".to_string()),
            });
        }

        let scan = BackupScanResult {
            directory_id: "test".to_string(),
            total_files: 10,
            total_size_bytes: 1000,
            files,
            skipped_count: 0,
            skipped_log: vec![],
        };

        let preview = preview_cleanup(&scan, 3);
        assert_eq!(preview.total_files, 7);
        assert_eq!(preview.kept_files, 3);
        assert_eq!(preview.total_size_bytes, 700);
    }

    #[test]
    fn test_preview_cleanup_below_threshold() {
        let files: Vec<_> = (0..3)
            .map(|i| BackupFileEntry {
                path: format!("/tmp/f{}.bak", i),
                name: format!("f{}.bak", i),
                size_bytes: 100,
                modified: format!("2024-01-{:02}T00:00:00+00:00", i + 1),
                parent_project: Some("Proj".to_string()),
            })
            .collect();

        let scan = BackupScanResult {
            directory_id: "test".to_string(),
            total_files: 3,
            total_size_bytes: 300,
            files,
            skipped_count: 0,
            skipped_log: vec![],
        };

        let preview = preview_cleanup(&scan, 5);
        assert_eq!(preview.total_files, 0);
        assert_eq!(preview.kept_files, 3);
    }

    #[test]
    fn test_preview_multiple_projects() {
        let files = vec![
            BackupFileEntry {
                path: "/tmp/proj1/old.bak".to_string(),
                name: "old.bak".to_string(),
                size_bytes: 100,
                modified: "2024-01-01T00:00:00+00:00".to_string(),
                parent_project: Some("proj1".to_string()),
            },
            BackupFileEntry {
                path: "/tmp/proj1/newer.bak".to_string(),
                name: "newer.bak".to_string(),
                size_bytes: 100,
                modified: "2024-01-02T00:00:00+00:00".to_string(),
                parent_project: Some("proj1".to_string()),
            },
            BackupFileEntry {
                path: "/tmp/proj2/only.bak".to_string(),
                name: "only.bak".to_string(),
                size_bytes: 100,
                modified: "2024-01-01T00:00:00+00:00".to_string(),
                parent_project: Some("proj2".to_string()),
            },
        ];

        let scan = BackupScanResult {
            directory_id: "test".to_string(),
            total_files: 3,
            total_size_bytes: 300,
            files,
            skipped_count: 0,
            skipped_log: vec![],
        };

        // Keep 1 per project: proj1 has 2 (1 stays, 1 deleted), proj2 has 1 (stays)
        let preview = preview_cleanup(&scan, 1);
        assert_eq!(preview.total_files, 1);
        assert_eq!(preview.kept_files, 2);
    }

    #[test]
    fn test_execute_cleanup() {
        let db = setup_db();
        let dir = create_dir("/tmp/wp-test-exec");
        let f1 = dir.join("del.bak");
        let f2 = dir.join("keep.bak");
        touch(&f1);
        touch(&f2);

        let added = add_backup_directory(
            &db,
            AddBackupDirectoryRequest {
                path: dir.to_string_lossy().to_string(),
                label: None,
                recursive: Some(true),
            },
        )
        .expect("Should add");

        let result = execute_cleanup(
            &db,
            &added.id,
            ExecuteCleanupRequest {
                directory_id: added.id.clone(),
                file_paths: vec![f1.to_string_lossy().to_string()],
            },
        )
        .expect("Should execute");
        assert_eq!(result.files_deleted, 1);
        assert_eq!(result.files_failed, 0);
        assert!(!f1.exists());
        assert!(f2.exists());

        let history = get_cleanup_history(&db).expect("Should get history");
        assert!(!history.is_empty());
    }

    #[test]
    fn test_cleanup_nonexistent_file() {
        let db = setup_db();
        let dir = create_dir("/tmp/wp-test-exec-nonexist");
        let added = add_backup_directory(
            &db,
            AddBackupDirectoryRequest {
                path: dir.to_string_lossy().to_string(),
                label: None,
                recursive: Some(true),
            },
        )
        .expect("Should add");

        let result = execute_cleanup(
            &db,
            &added.id,
            ExecuteCleanupRequest {
                directory_id: added.id.clone(),
                file_paths: vec!["/tmp/wp-test-nonexistent-file.bak".to_string()],
            },
        )
        .expect("Should handle gracefully");
        assert_eq!(result.files_deleted, 0);
        assert_eq!(result.files_failed, 1);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_history_after_cleanup() {
        let db = setup_db();
        let dir = create_dir("/tmp/wp-test-history");
        let added = add_backup_directory(
            &db,
            AddBackupDirectoryRequest {
                path: dir.to_string_lossy().to_string(),
                label: None,
                recursive: Some(true),
            },
        )
        .expect("Should add");

        let f = dir.join("old.bak");
        touch(&f);

        execute_cleanup(
            &db,
            &added.id,
            ExecuteCleanupRequest {
                directory_id: added.id.clone(),
                file_paths: vec![f.to_string_lossy().to_string()],
            },
        )
        .expect("Should execute");

        let history = get_cleanup_history(&db).expect("Should get history");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].files_deleted, 1);
    }

    #[test]
    fn test_history_empty_when_no_cleanups() {
        let db = setup_db();
        let history = get_cleanup_history(&db).expect("Should get history");
        assert!(history.is_empty());
    }

    #[test]
    fn test_add_duplicate_directory() {
        let db = setup_db();
        let dir = create_dir("/tmp/wp-test-dup-dir");
        let req = AddBackupDirectoryRequest {
            path: dir.to_string_lossy().to_string(),
            label: None,
            recursive: Some(true),
        };
        add_backup_directory(&db, req.clone()).expect("First add");
        let result = add_backup_directory(&db, req);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already added"));
    }

    #[test]
    fn test_parent_project_detection() {
        // File in Backup subfolder
        let path = std::path::Path::new("/Projects/MyTrack/Backup/old.bak");
        assert_eq!(
            detect_parent_project(path),
            Some("MyTrack".to_string())
        );

        // File directly in project folder
        let path2 = std::path::Path::new("/Projects/MyTrack/old.bak");
        assert_eq!(
            detect_parent_project(path2),
            Some("MyTrack".to_string())
        );
    }
}
