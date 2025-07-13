use crate::models::flutter::{ArchitectureLayerConfig, ModelContext, PrivacyViolation};
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

// Privacy rule repository removed - not currently implemented or used

#[async_trait]
#[allow(dead_code)]
pub trait PrivacyViolationRepository: Send + Sync {
    async fn create(&self, violation: &PrivacyViolation) -> Result<PrivacyViolation, McpError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<PrivacyViolation>, McpError>;
    async fn update(&self, violation: &PrivacyViolation) -> Result<PrivacyViolation, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn list_by_rule(&self, rule_id: &str) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn list_by_status(
        &self,
        project_id: &str,
        status: &str,
    ) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn bulk_create(
        &self,
        violations: &[PrivacyViolation],
    ) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn bulk_update(
        &self,
        violations: &[PrivacyViolation],
    ) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError>;
}

#[async_trait]
#[allow(dead_code)]
pub trait ArchitectureLayerRepository: Send + Sync {
    async fn create(
        &self,
        layer_config: &ArchitectureLayerConfig,
    ) -> Result<ArchitectureLayerConfig, McpError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<ArchitectureLayerConfig>, McpError>;
    async fn update(
        &self,
        layer_config: &ArchitectureLayerConfig,
    ) -> Result<ArchitectureLayerConfig, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn list_by_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ArchitectureLayerConfig>, McpError>;
    async fn get_by_layer_name(
        &self,
        project_id: &str,
        layer_name: &str,
    ) -> Result<Option<ArchitectureLayerConfig>, McpError>;
    async fn bulk_create(
        &self,
        layer_configs: &[ArchitectureLayerConfig],
    ) -> Result<Vec<ArchitectureLayerConfig>, McpError>;
    async fn bulk_update(
        &self,
        layer_configs: &[ArchitectureLayerConfig],
    ) -> Result<Vec<ArchitectureLayerConfig>, McpError>;
    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError>;
}

#[async_trait]
#[allow(dead_code)]
pub trait ModelContextRepository: Send + Sync {
    async fn create(&self, model_context: &ModelContext) -> Result<ModelContext, McpError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<ModelContext>, McpError>;
    async fn update(&self, model_context: &ModelContext) -> Result<ModelContext, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<ModelContext>, McpError>;
    async fn get_active_model(&self, project_id: &str) -> Result<Option<ModelContext>, McpError>;
    async fn set_active_model(&self, project_id: &str, model_id: &str) -> Result<bool, McpError>;
    async fn bulk_create(
        &self,
        model_contexts: &[ModelContext],
    ) -> Result<Vec<ModelContext>, McpError>;
    async fn bulk_update(
        &self,
        model_contexts: &[ModelContext],
    ) -> Result<Vec<ModelContext>, McpError>;
    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError>;
}

// Code template repository removed - not currently implemented or used
// This was Flutter-specific functionality that's not needed for the framework-agnostic server
