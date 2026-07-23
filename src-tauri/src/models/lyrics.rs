use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct LyricResponse {
    pub id: String,
    pub project_id: String,
    pub title: Option<String>,
    pub content: String,
    pub section: Option<String>,
    pub language: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLyricRequest {
    pub project_id: String,
    pub title: Option<String>,
    pub content: String,
    pub section: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateLyricRequest {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub section: Option<String>,
    pub language: Option<String>,
}
