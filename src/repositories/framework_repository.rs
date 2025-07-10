use async_trait::async_trait;
use crate::models::framework::FrameworkComponent;
use rmcp::model::ErrorData as McpError;

/// Repository interface for generic framework/language component operations
#[async_trait]
pub trait FrameworkRepository: Send + Sync {
    async fn create(&self, component: &FrameworkComponent) -> Result<FrameworkComponent, McpError>;
    async fn find_by_project_id(&self, project_id: &str) -> Result<Vec<FrameworkComponent>, McpError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<FrameworkComponent>, McpError>;
    async fn update(&self, component: &FrameworkComponent) -> Result<FrameworkComponent, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn find_by_architecture_layer(&self, project_id: &str, layer: &str) -> Result<Vec<FrameworkComponent>, McpError>;
}
