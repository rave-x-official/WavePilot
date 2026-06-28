use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportProjectRequest {
    pub name: String,
    pub path: String,
    pub artist: Option<String>,
    pub daw_type: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateProjectRequest {
    pub id: String,
    pub name: Option<String>,
    pub artist: Option<String>,
    pub bpm: Option<f64>,
    pub musical_key: Option<String>,
    pub root_note: Option<String>,
    pub tags: Option<Vec<String>>,
    pub daw_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSearchQuery {
    pub query: Option<String>,
    pub artist: Option<String>,
    pub bpm_min: Option<f64>,
    pub bpm_max: Option<f64>,
    pub musical_key: Option<String>,
    pub root_note: Option<String>,
    pub tags: Option<Vec<String>>,
    pub daw_type: Option<String>,
}
