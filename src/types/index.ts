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

export type NavPage =
  | "projects"
  | "search"
  | "analysis"
  | "lyrics"
  | "releases"
  | "settings";
