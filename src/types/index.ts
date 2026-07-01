export interface Project {
  id: string;
  name: string;
  path: string;
  artist: string | null;
  bpm: number | null;
  musical_key: string | null;
  root_note: string | null;
  tags: string | null;
  keywords: string | null;
  notes: string | null;
  favorite: boolean;
  daw_type: string | null;
  last_opened: string | null;
  created_at: string;
  updated_at: string;
}

export interface ImportProjectRequest {
  name: string;
  path: string;
  artist?: string;
  daw_type?: string;
  tags?: string[];
  keywords?: string;
  notes?: string;
}

export interface UpdateProjectRequest {
  id: string;
  name?: string;
  artist?: string;
  bpm?: number;
  musical_key?: string;
  root_note?: string;
  tags?: string[];
  keywords?: string;
  notes?: string;
  favorite?: boolean;
  daw_type?: string;
}

export type SortField = "Name" | "DateAdded" | "LastOpened" | "Bpm" | "Artist";
export type SortOrder = "Asc" | "Desc";
export type ViewMode = "Grid" | "List";

export interface ListProjectsQuery {
  search?: string;
  artist?: string;
  bpm_min?: number;
  bpm_max?: number;
  musical_key?: string;
  root_note?: string;
  tags?: string[];
  keywords?: string;
  favorite_only?: boolean;
  sort_by?: SortField;
  sort_order?: SortOrder;
  view?: ViewMode;
}

export interface Settings {
  theme: string;
  default_backup_count: number;
  projects_directory: string | null;
  analysis_enabled: boolean;
  autosave_interval_seconds: number;
}

// --- Backup Cleaner ---

export interface BackupDirectory {
  id: string;
  path: string;
  label: string | null;
  recursive: boolean;
  created_at: string;
  updated_at: string;
}

export interface AddBackupDirectoryRequest {
  path: string;
  label?: string;
  recursive?: boolean;
}

export interface BackupFileEntry {
  path: string;
  name: string;
  size_bytes: number;
  modified: string;
  parent_project: string | null;
}

export interface BackupScanResult {
  directory_id: string;
  files: BackupFileEntry[];
  total_files: number;
  total_size_bytes: number;
  skipped_count: number;
  skipped_log: string[];
}

export interface CleanupPreview {
  directory_id: string;
  files_to_delete: BackupFileEntry[];
  total_files: number;
  total_size_bytes: number;
  kept_files: number;
}

export interface ExecuteCleanupRequest {
  directory_id: string;
  file_paths: string[];
}

export interface CleanupResult {
  files_deleted: number;
  files_failed: number;
  space_freed_bytes: number;
  errors: string[];
}

export interface BackupHistoryEntry {
  id: string;
  directory_id: string;
  directory_path: string;
  scanned_at: string;
  total_files: number;
  files_deleted: number;
  space_freed_bytes: number;
  status: string;
  error: string | null;
}

export interface BackupSettings {
  backups_to_keep: number;
  min_file_age_days: number;
  recursive_scan: boolean;
  confirm_before_delete: boolean;
}

// --- Navigation ---

export type NavPage =
  | "projects"
  | "search"
  | "analysis"
  | "lyrics"
  | "releases"
  | "backup"
  | "settings";
