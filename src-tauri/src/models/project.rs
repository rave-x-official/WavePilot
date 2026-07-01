use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportProjectRequest {
    pub name: String,
    pub path: String,
    pub artist: Option<String>,
    pub daw_type: Option<String>,
    pub tags: Option<Vec<String>>,
    pub keywords: Option<String>,
    pub notes: Option<String>,
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
    pub keywords: Option<String>,
    pub notes: Option<String>,
    pub favorite: Option<bool>,
    pub daw_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListProjectsQuery {
    pub search: Option<String>,
    pub artist: Option<String>,
    pub bpm_min: Option<f64>,
    pub bpm_max: Option<f64>,
    pub musical_key: Option<String>,
    pub root_note: Option<String>,
    pub tags: Option<Vec<String>>,
    pub keywords: Option<String>,
    pub favorite_only: Option<bool>,
    pub sort_by: Option<SortField>,
    pub sort_order: Option<SortOrder>,
    pub view: Option<ViewMode>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SortField {
    Name,
    DateAdded,
    LastOpened,
    Bpm,
    Artist,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ViewMode {
    Grid,
    List,
}
