use crate::models::context::ProjectConvention;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

#[async_trait]
pub trait ProjectConventionRepository: Send + Sync {
    async fn create(&self, convention: &ProjectConvention) -> Result<ProjectConvention, McpError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<ProjectConvention>, McpError>;
    async fn update(&self, convention: &ProjectConvention) -> Result<ProjectConvention, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<ProjectConvention>, McpError>;
    async fn list_by_convention_type(
        &self,
        project_id: &str,
        convention_type: &str,
    ) -> Result<Vec<ProjectConvention>, McpError>;
    async fn bulk_create(
        &self,
        conventions: &[ProjectConvention],
    ) -> Result<Vec<ProjectConvention>, McpError>;
    async fn bulk_update(
        &self,
        conventions: &[ProjectConvention],
    ) -> Result<Vec<ProjectConvention>, McpError>;
    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError>;
}
