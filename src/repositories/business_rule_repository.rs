use async_trait::async_trait;
use crate::models::context::BusinessRule;
use rmcp::model::ErrorData as McpError;

/// Repository interface for Business Rule operations (DIP - Dependency Inversion)
#[async_trait]
pub trait BusinessRuleRepository: Send + Sync {
    async fn create(&self, rule: &BusinessRule) -> Result<BusinessRule, McpError>;
    async fn find_by_project_id(&self, project_id: &str) -> Result<Vec<BusinessRule>, McpError>;
    async fn find_by_domain_area(&self, project_id: &str, domain_area: &str) -> Result<Vec<BusinessRule>, McpError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<BusinessRule>, McpError>;
    async fn update(&self, rule: &BusinessRule) -> Result<BusinessRule, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
}
