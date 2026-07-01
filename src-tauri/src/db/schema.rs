use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub artist: Option<String>,
    pub bpm: Option<f64>,
    pub musical_key: Option<String>,
    pub root_note: Option<String>,
    pub tags: Option<String>,
    pub keywords: Option<String>,
    pub notes: Option<String>,
    pub favorite: bool,
    pub daw_type: Option<String>,
    pub last_opened: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lyric {
    pub id: String,
    pub project_id: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupRule {
    pub id: String,
    pub project_id: Option<String>,
    pub max_backups: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReleaseChecklist {
    pub id: String,
    pub project_id: String,
    pub items: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisCache {
    pub id: String,
    pub project_id: String,
    pub analysis_type: String,
    pub result: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectTag {
    pub id: String,
    pub project_id: String,
    pub tag: String,
}
