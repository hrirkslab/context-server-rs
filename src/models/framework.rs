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

// Generic server metadata structures (framework-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilitiesInfo {
    pub server_info: ServerMetadata,
    pub features: Vec<FeatureInfo>,
    pub database_tables: Vec<TableInfo>,
    pub mcp_tools: Vec<ToolInfo>,
    pub usage_examples: Vec<UsageExample>,
    pub recommended_workflow: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub config_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    pub name: String,
    pub description: String,
    pub status: FeatureStatus,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureStatus {
    Implemented,
    Framework,    // Database structure exists, tools being added
    Planned,      // Not yet implemented
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub description: String,
    pub primary_fields: Vec<String>,
    pub example_use: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub required_params: Vec<String>,
    pub example_use: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageExample {
    pub scenario: String,
    pub steps: Vec<String>,
}

// Display trait implementations
impl std::fmt::Display for FeatureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureStatus::Implemented => write!(f, "implemented"),
            FeatureStatus::Framework => write!(f, "framework"),
            FeatureStatus::Planned => write!(f, "planned"),
        }
    }
}
