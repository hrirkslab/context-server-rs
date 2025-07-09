use async_trait::async_trait;
use crate::models::flutter::FlutterComponent;
use rmcp::model::ErrorData as McpError;

/// Repository interface for Flutter component operations (DIP - Dependency Inversion)
#[async_trait]
pub trait FlutterRepository: Send + Sync {
    async fn create(&self, component: &FlutterComponent) -> Result<FlutterComponent, McpError>;
    async fn find_by_project_id(&self, project_id: &str) -> Result<Vec<FlutterComponent>, McpError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<FlutterComponent>, McpError>;
    async fn update(&self, component: &FlutterComponent) -> Result<FlutterComponent, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn find_by_architecture_layer(&self, project_id: &str, layer: &str) -> Result<Vec<FlutterComponent>, McpError>;
}
