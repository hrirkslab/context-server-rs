use crate::models::context::SecurityPolicy;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

#[async_trait]
pub trait SecurityPolicyRepository: Send + Sync {
    async fn create(&self, security_policy: &SecurityPolicy) -> Result<SecurityPolicy, McpError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<SecurityPolicy>, McpError>;
    async fn update(&self, security_policy: &SecurityPolicy) -> Result<SecurityPolicy, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<SecurityPolicy>, McpError>;
    async fn list_by_policy_area(
        &self,
        project_id: &str,
        policy_area: &str,
    ) -> Result<Vec<SecurityPolicy>, McpError>;
    async fn bulk_create(
        &self,
        security_policies: &[SecurityPolicy],
    ) -> Result<Vec<SecurityPolicy>, McpError>;
    async fn bulk_update(
        &self,
        security_policies: &[SecurityPolicy],
    ) -> Result<Vec<SecurityPolicy>, McpError>;
    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError>;
}
