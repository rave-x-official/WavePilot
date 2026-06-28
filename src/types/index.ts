export interface Project {
  id: string;
  name: string;
  path: string;
  artist: string | null;
  bpm: number | null;
  musical_key: string | null;
  root_note: string | null;
  tags: string | null;
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
}

export interface UpdateProjectRequest {
  id: string;
  name?: string;
  artist?: string;
  bpm?: number;
  musical_key?: string;
  root_note?: string;
  tags?: string[];
  daw_type?: string;
}

export interface ProjectSearchQuery {
  query?: string;
  artist?: string;
  bpm_min?: number;
  bpm_max?: number;
  musical_key?: string;
  root_note?: string;
  tags?: string[];
  daw_type?: string;
}

export interface Settings {
  theme: string;
  default_backup_count: number;
  projects_directory: string | null;
  analysis_enabled: boolean;
  autosave_interval_seconds: number;
}

export interface Lyric {
  id: string;
  project_id: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export type NavPage =
  | "projects"
  | "search"
  | "analysis"
  | "lyrics"
  | "releases"
  | "settings";
