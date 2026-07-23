use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChecklistItem {
    pub id: String,
    pub label: String,
    pub done: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ReleaseChecklistResponse {
    pub id: String,
    pub project_id: String,
    pub items: Vec<ChecklistItem>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateChecklistRequest {
    pub project_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateChecklistItemRequest {
    pub checklist_id: String,
    pub item_id: String,
    pub done: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddChecklistItemRequest {
    pub checklist_id: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveChecklistItemRequest {
    pub checklist_id: String,
    pub item_id: String,
}
