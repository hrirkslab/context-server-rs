use async_trait::async_trait;
use crate::models::flutter::{PrivacyRule, PrivacyViolation, ArchitectureLayerConfig, ModelContext, CodeTemplate};
use rmcp::model::ErrorData as McpError;

#[async_trait]
pub trait PrivacyRuleRepository: Send + Sync {
    async fn create(&self, privacy_rule: &PrivacyRule) -> Result<PrivacyRule, McpError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<PrivacyRule>, McpError>;
    async fn update(&self, privacy_rule: &PrivacyRule) -> Result<PrivacyRule, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<PrivacyRule>, McpError>;
    async fn list_by_rule_type(&self, project_id: &str, rule_type: &str) -> Result<Vec<PrivacyRule>, McpError>;
    async fn bulk_create(&self, privacy_rules: &[PrivacyRule]) -> Result<Vec<PrivacyRule>, McpError>;
    async fn bulk_update(&self, privacy_rules: &[PrivacyRule]) -> Result<Vec<PrivacyRule>, McpError>;
    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError>;
}

#[async_trait]
pub trait PrivacyViolationRepository: Send + Sync {
    async fn create(&self, violation: &PrivacyViolation) -> Result<PrivacyViolation, McpError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<PrivacyViolation>, McpError>;
    async fn update(&self, violation: &PrivacyViolation) -> Result<PrivacyViolation, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn list_by_rule(&self, rule_id: &str) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn list_by_status(&self, project_id: &str, status: &str) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn bulk_create(&self, violations: &[PrivacyViolation]) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn bulk_update(&self, violations: &[PrivacyViolation]) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError>;
}

#[async_trait]
pub trait ArchitectureLayerRepository: Send + Sync {
    async fn create(&self, layer_config: &ArchitectureLayerConfig) -> Result<ArchitectureLayerConfig, McpError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<ArchitectureLayerConfig>, McpError>;
    async fn update(&self, layer_config: &ArchitectureLayerConfig) -> Result<ArchitectureLayerConfig, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<ArchitectureLayerConfig>, McpError>;
    async fn get_by_layer_name(&self, project_id: &str, layer_name: &str) -> Result<Option<ArchitectureLayerConfig>, McpError>;
    async fn bulk_create(&self, layer_configs: &[ArchitectureLayerConfig]) -> Result<Vec<ArchitectureLayerConfig>, McpError>;
    async fn bulk_update(&self, layer_configs: &[ArchitectureLayerConfig]) -> Result<Vec<ArchitectureLayerConfig>, McpError>;
    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError>;
}

#[async_trait]
pub trait ModelContextRepository: Send + Sync {
    async fn create(&self, model_context: &ModelContext) -> Result<ModelContext, McpError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<ModelContext>, McpError>;
    async fn update(&self, model_context: &ModelContext) -> Result<ModelContext, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<ModelContext>, McpError>;
    async fn get_active_model(&self, project_id: &str) -> Result<Option<ModelContext>, McpError>;
    async fn set_active_model(&self, project_id: &str, model_id: &str) -> Result<bool, McpError>;
    async fn bulk_create(&self, model_contexts: &[ModelContext]) -> Result<Vec<ModelContext>, McpError>;
    async fn bulk_update(&self, model_contexts: &[ModelContext]) -> Result<Vec<ModelContext>, McpError>;
    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError>;
}

#[async_trait]
pub trait CodeTemplateRepository: Send + Sync {
    async fn create(&self, template: &CodeTemplate) -> Result<CodeTemplate, McpError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<CodeTemplate>, McpError>;
    async fn update(&self, template: &CodeTemplate) -> Result<CodeTemplate, McpError>;
    async fn delete(&self, id: &str) -> Result<bool, McpError>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<CodeTemplate>, McpError>;
    async fn list_by_template_type(&self, project_id: &str, template_type: &str) -> Result<Vec<CodeTemplate>, McpError>;
    async fn get_by_name(&self, project_id: &str, template_name: &str) -> Result<Option<CodeTemplate>, McpError>;
    async fn bulk_create(&self, templates: &[CodeTemplate]) -> Result<Vec<CodeTemplate>, McpError>;
    async fn bulk_update(&self, templates: &[CodeTemplate]) -> Result<Vec<CodeTemplate>, McpError>;
    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError>;
}
