/// Tagging system for categorizing context entries
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextTag {
    pub id: String,
    pub project_id: String,
    pub tag_name: String,
    pub category: String, // e.g., "team", "priority", "component", "status"
    pub color: Option<String>, // hex color for UI
    pub description: Option<String>,
    pub created_at: String,
}

impl ContextTag {
    pub fn new(project_id: String, tag_name: String, category: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            tag_name,
            category,
            color: None,
            description: None,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedEntity {
    pub id: String,
    pub project_id: String,
    pub entity_id: String,
    pub entity_type: String, // e.g., "constraint", "business_rule", "service"
    pub tag_id: String,
    pub tagged_at: String,
}

impl TaggedEntity {
    pub fn new(
        project_id: String,
        entity_id: String,
        entity_type: String,
        tag_id: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            entity_id,
            entity_type,
            tag_id,
            tagged_at: now,
        }
    }
}
