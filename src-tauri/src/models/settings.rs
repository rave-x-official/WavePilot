use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub theme: String,
    pub default_backup_count: i32,
    pub projects_directory: Option<String>,
    pub analysis_enabled: bool,
    pub autosave_interval_seconds: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            default_backup_count: 5,
            projects_directory: None,
            analysis_enabled: true,
            autosave_interval_seconds: 30,
        }
    }
}
