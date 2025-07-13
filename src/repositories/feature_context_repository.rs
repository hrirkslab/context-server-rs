use crate::models::context::FeatureContext;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

#[async_trait]
pub trait FeatureContextRepository: Send + Sync {
    async fn create(&self, feature_context: &FeatureContext) -> Result<FeatureContext, McpError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<FeatureContext>, McpError>;
    async fn update(&self, feature_context: &FeatureContext) -> Result<FeatureContext, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<FeatureContext>, McpError>;
    async fn get_by_feature_name(
        &self,
        project_id: &str,
        feature_name: &str,
    ) -> Result<Option<FeatureContext>, McpError>;
    async fn bulk_create(
        &self,
        feature_contexts: &[FeatureContext],
    ) -> Result<Vec<FeatureContext>, McpError>;
    async fn bulk_update(
        &self,
        feature_contexts: &[FeatureContext],
    ) -> Result<Vec<FeatureContext>, McpError>;
    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError>;
}
