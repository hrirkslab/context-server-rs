use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkComponent {
    pub id: String,
    pub project_id: String,
    pub component_name: String,
    pub component_type: String, // Now a string to support any language/framework
    pub architecture_layer: String, // Now a string to support any architecture
    pub file_path: Option<String>,
    pub dependencies: Vec<String>,
    pub metadata: Option<serde_json::Value>, // For framework/language-specific fields
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
