use crate::models::context::ArchitecturalDecision;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

/// Repository interface for Architectural Decision operations (DIP - Dependency Inversion)
#[async_trait]
pub trait ArchitecturalDecisionRepository: Send + Sync {
    async fn create(
        &self,
        decision: &ArchitecturalDecision,
    ) -> Result<ArchitecturalDecision, McpError>;
    async fn find_by_project_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<ArchitecturalDecision>, McpError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<ArchitecturalDecision>, McpError>;
    async fn update(
        &self,
        decision: &ArchitecturalDecision,
    ) -> Result<ArchitecturalDecision, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
}
