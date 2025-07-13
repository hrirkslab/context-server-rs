use crate::models::context::Project;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

/// Repository interface for Project operations (DIP - Dependency Inversion)
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, project: &Project) -> Result<Project, McpError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Project>, McpError>;
    async fn find_all(&self) -> Result<Vec<Project>, McpError>;
    async fn update(&self, project: &Project) -> Result<Project, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
}
