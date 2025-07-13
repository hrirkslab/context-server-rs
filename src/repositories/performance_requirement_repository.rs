use crate::models::context::PerformanceRequirement;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

/// Repository interface for Performance Requirement operations (DIP - Dependency Inversion)
#[async_trait]
pub trait PerformanceRequirementRepository: Send + Sync {
    async fn create(
        &self,
        requirement: &PerformanceRequirement,
    ) -> Result<PerformanceRequirement, McpError>;
    async fn find_by_project_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<PerformanceRequirement>, McpError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<PerformanceRequirement>, McpError>;
    async fn update(
        &self,
        requirement: &PerformanceRequirement,
    ) -> Result<PerformanceRequirement, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
}
