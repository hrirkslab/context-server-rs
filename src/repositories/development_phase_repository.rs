use crate::models::development::DevelopmentPhase;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

/// Repository interface for Development Phase operations (DIP - Dependency Inversion)
#[async_trait]
pub trait DevelopmentPhaseRepository: Send + Sync {
    async fn create(&self, phase: &DevelopmentPhase) -> Result<DevelopmentPhase, McpError>;
    async fn find_by_project_id(&self, project_id: &str)
        -> Result<Vec<DevelopmentPhase>, McpError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<DevelopmentPhase>, McpError>;
    async fn update(&self, phase: &DevelopmentPhase) -> Result<DevelopmentPhase, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
}
