use async_trait::async_trait;
use crate::models::flutter::{
    PrivacyRule, PrivacyViolation, ArchitectureLayerConfig, ModelContext, CodeTemplate,
    PrivacyRuleType, Severity, ViolationStatus, TemplateType
};
use crate::repositories::{
    PrivacyRuleRepository, PrivacyViolationRepository, ArchitectureLayerRepository,
    ModelContextRepository, CodeTemplateRepository
};
use rmcp::model::ErrorData as McpError;
use uuid::Uuid;

/// Advanced CRUD service for Flutter-specific entities
#[async_trait]
#[allow(dead_code)]
pub trait FlutterAdvancedCrudService: Send + Sync {
    // Privacy Rule CRUD
    async fn create_privacy_rule(&self, project_id: &str, rule_name: &str, rule_type: PrivacyRuleType, pattern: &str) -> Result<PrivacyRule, McpError>;
    async fn get_privacy_rule(&self, id: &str) -> Result<Option<PrivacyRule>, McpError>;
    async fn update_privacy_rule(&self, rule: &PrivacyRule) -> Result<PrivacyRule, McpError>;
    async fn delete_privacy_rule(&self, id: &str) -> Result<bool, McpError>;
    async fn list_privacy_rules(&self, project_id: &str) -> Result<Vec<PrivacyRule>, McpError>;
    async fn list_privacy_rules_by_type(&self, project_id: &str, rule_type: &str) -> Result<Vec<PrivacyRule>, McpError>;

    // Privacy Violation CRUD
    async fn create_privacy_violation(&self, project_id: &str, rule_id: &str, file_path: &str) -> Result<PrivacyViolation, McpError>;
    async fn get_privacy_violation(&self, id: &str) -> Result<Option<PrivacyViolation>, McpError>;
    async fn update_privacy_violation(&self, violation: &PrivacyViolation) -> Result<PrivacyViolation, McpError>;
    async fn delete_privacy_violation(&self, id: &str) -> Result<bool, McpError>;
    async fn list_privacy_violations(&self, project_id: &str) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn list_violations_by_rule(&self, rule_id: &str) -> Result<Vec<PrivacyViolation>, McpError>;
    async fn list_violations_by_status(&self, project_id: &str, status: ViolationStatus) -> Result<Vec<PrivacyViolation>, McpError>;

    // Architecture Layer CRUD
    async fn create_architecture_layer(&self, project_id: &str, layer_name: &str) -> Result<ArchitectureLayerConfig, McpError>;
    async fn get_architecture_layer(&self, id: &str) -> Result<Option<ArchitectureLayerConfig>, McpError>;
    async fn get_architecture_layer_by_name(&self, project_id: &str, layer_name: &str) -> Result<Option<ArchitectureLayerConfig>, McpError>;
    async fn update_architecture_layer(&self, layer: &ArchitectureLayerConfig) -> Result<ArchitectureLayerConfig, McpError>;
    async fn delete_architecture_layer(&self, id: &str) -> Result<bool, McpError>;
    async fn list_architecture_layers(&self, project_id: &str) -> Result<Vec<ArchitectureLayerConfig>, McpError>;

    // Model Context CRUD
    async fn create_model_context(&self, project_id: &str, model_name: &str, model_path: Option<&str>) -> Result<ModelContext, McpError>;
    async fn get_model_context(&self, id: &str) -> Result<Option<ModelContext>, McpError>;
    async fn update_model_context(&self, model_context: &ModelContext) -> Result<ModelContext, McpError>;
    async fn delete_model_context(&self, id: &str) -> Result<bool, McpError>;
    async fn list_model_contexts(&self, project_id: &str) -> Result<Vec<ModelContext>, McpError>;
    async fn get_active_model(&self, project_id: &str) -> Result<Option<ModelContext>, McpError>;
    async fn set_active_model(&self, project_id: &str, model_id: &str) -> Result<bool, McpError>;

    // Code Template CRUD
    async fn create_code_template(&self, project_id: &str, template_name: &str, template_type: TemplateType, template_content: &str) -> Result<CodeTemplate, McpError>;
    async fn get_code_template(&self, id: &str) -> Result<Option<CodeTemplate>, McpError>;
    async fn get_code_template_by_name(&self, project_id: &str, template_name: &str) -> Result<Option<CodeTemplate>, McpError>;
    async fn update_code_template(&self, template: &CodeTemplate) -> Result<CodeTemplate, McpError>;
    async fn delete_code_template(&self, id: &str) -> Result<bool, McpError>;
    async fn list_code_templates(&self, project_id: &str) -> Result<Vec<CodeTemplate>, McpError>;
    async fn list_code_templates_by_type(&self, project_id: &str, template_type: TemplateType) -> Result<Vec<CodeTemplate>, McpError>;

    // Bulk operations
    async fn bulk_create_privacy_rules(&self, rules: &[PrivacyRule]) -> Result<Vec<PrivacyRule>, McpError>;
    async fn bulk_update_privacy_rules(&self, rules: &[PrivacyRule]) -> Result<Vec<PrivacyRule>, McpError>;
    async fn bulk_delete_privacy_rules(&self, ids: &[String]) -> Result<usize, McpError>;

    async fn bulk_create_code_templates(&self, templates: &[CodeTemplate]) -> Result<Vec<CodeTemplate>, McpError>;
    async fn bulk_update_code_templates(&self, templates: &[CodeTemplate]) -> Result<Vec<CodeTemplate>, McpError>;
    async fn bulk_delete_code_templates(&self, ids: &[String]) -> Result<usize, McpError>;
}

/// Implementation of FlutterAdvancedCrudService
pub struct FlutterAdvancedCrudServiceImpl<PRR, PVR, ALR, MCR, CTR>
where
    PRR: PrivacyRuleRepository,
    PVR: PrivacyViolationRepository,
    ALR: ArchitectureLayerRepository,
    MCR: ModelContextRepository,
    CTR: CodeTemplateRepository,
{
    privacy_rule_repository: PRR,
    privacy_violation_repository: PVR,
    architecture_layer_repository: ALR,
    model_context_repository: MCR,
    code_template_repository: CTR,
}

impl<PRR, PVR, ALR, MCR, CTR> FlutterAdvancedCrudServiceImpl<PRR, PVR, ALR, MCR, CTR>
where
    PRR: PrivacyRuleRepository,
    PVR: PrivacyViolationRepository,
    ALR: ArchitectureLayerRepository,
    MCR: ModelContextRepository,
    CTR: CodeTemplateRepository,
{
    #[allow(dead_code)]
    pub fn new(
        privacy_rule_repository: PRR,
        privacy_violation_repository: PVR,
        architecture_layer_repository: ALR,
        model_context_repository: MCR,
        code_template_repository: CTR,
    ) -> Self {
        Self {
            privacy_rule_repository,
            privacy_violation_repository,
            architecture_layer_repository,
            model_context_repository,
            code_template_repository,
        }
    }
}

#[async_trait]
impl<PRR, PVR, ALR, MCR, CTR> FlutterAdvancedCrudService for FlutterAdvancedCrudServiceImpl<PRR, PVR, ALR, MCR, CTR>
where
    PRR: PrivacyRuleRepository,
    PVR: PrivacyViolationRepository,
    ALR: ArchitectureLayerRepository,
    MCR: ModelContextRepository,
    CTR: CodeTemplateRepository,
{
    // Privacy Rule CRUD Implementation
    async fn create_privacy_rule(&self, project_id: &str, rule_name: &str, rule_type: PrivacyRuleType, pattern: &str) -> Result<PrivacyRule, McpError> {
        let rule = PrivacyRule {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            rule_name: rule_name.to_string(),
            rule_type,
            pattern: pattern.to_string(),
            description: None,
            severity: Severity::Error, // Default to Error
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.privacy_rule_repository.create(&rule).await
    }

    async fn get_privacy_rule(&self, id: &str) -> Result<Option<PrivacyRule>, McpError> {
        self.privacy_rule_repository.get_by_id(id).await
    }

    async fn update_privacy_rule(&self, rule: &PrivacyRule) -> Result<PrivacyRule, McpError> {
        self.privacy_rule_repository.update(rule).await
    }

    async fn delete_privacy_rule(&self, id: &str) -> Result<bool, McpError> {
        self.privacy_rule_repository.delete(id).await
    }

    async fn list_privacy_rules(&self, project_id: &str) -> Result<Vec<PrivacyRule>, McpError> {
        self.privacy_rule_repository.list_by_project(project_id).await
    }

    async fn list_privacy_rules_by_type(&self, project_id: &str, rule_type: &str) -> Result<Vec<PrivacyRule>, McpError> {
        self.privacy_rule_repository.list_by_rule_type(project_id, rule_type).await
    }

    // Privacy Violation CRUD Implementation
    async fn create_privacy_violation(&self, project_id: &str, rule_id: &str, file_path: &str) -> Result<PrivacyViolation, McpError> {
        let violation = PrivacyViolation {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            rule_id: rule_id.to_string(),
            file_path: file_path.to_string(),
            line_number: None,
            violation_text: None,
            status: ViolationStatus::Open,
            detected_at: Some(chrono::Utc::now().to_rfc3339()),
            resolved_at: None,
        };
        self.privacy_violation_repository.create(&violation).await
    }

    async fn get_privacy_violation(&self, id: &str) -> Result<Option<PrivacyViolation>, McpError> {
        self.privacy_violation_repository.get_by_id(id).await
    }

    async fn update_privacy_violation(&self, violation: &PrivacyViolation) -> Result<PrivacyViolation, McpError> {
        self.privacy_violation_repository.update(violation).await
    }

    async fn delete_privacy_violation(&self, id: &str) -> Result<bool, McpError> {
        self.privacy_violation_repository.delete(id).await
    }

    async fn list_privacy_violations(&self, project_id: &str) -> Result<Vec<PrivacyViolation>, McpError> {
        self.privacy_violation_repository.list_by_project(project_id).await
    }

    async fn list_violations_by_rule(&self, rule_id: &str) -> Result<Vec<PrivacyViolation>, McpError> {
        self.privacy_violation_repository.list_by_rule(rule_id).await
    }

    async fn list_violations_by_status(&self, project_id: &str, status: ViolationStatus) -> Result<Vec<PrivacyViolation>, McpError> {
        let status_str = match status {
            ViolationStatus::Open => "open",
            ViolationStatus::Resolved => "resolved",
            ViolationStatus::Suppressed => "suppressed",
        };
        self.privacy_violation_repository.list_by_status(project_id, status_str).await
    }

    // Architecture Layer CRUD Implementation
    async fn create_architecture_layer(&self, project_id: &str, layer_name: &str) -> Result<ArchitectureLayerConfig, McpError> {
        let layer = ArchitectureLayerConfig {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            layer_name: layer_name.to_string(),
            allowed_dependencies: Vec::new(),
            forbidden_imports: Vec::new(),
            description: None,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.architecture_layer_repository.create(&layer).await
    }

    async fn get_architecture_layer(&self, id: &str) -> Result<Option<ArchitectureLayerConfig>, McpError> {
        self.architecture_layer_repository.get_by_id(id).await
    }

    async fn get_architecture_layer_by_name(&self, project_id: &str, layer_name: &str) -> Result<Option<ArchitectureLayerConfig>, McpError> {
        self.architecture_layer_repository.get_by_layer_name(project_id, layer_name).await
    }

    async fn update_architecture_layer(&self, layer: &ArchitectureLayerConfig) -> Result<ArchitectureLayerConfig, McpError> {
        self.architecture_layer_repository.update(layer).await
    }

    async fn delete_architecture_layer(&self, id: &str) -> Result<bool, McpError> {
        self.architecture_layer_repository.delete(id).await
    }

    async fn list_architecture_layers(&self, project_id: &str) -> Result<Vec<ArchitectureLayerConfig>, McpError> {
        self.architecture_layer_repository.list_by_project(project_id).await
    }

    // Model Context CRUD Implementation
    async fn create_model_context(&self, project_id: &str, model_name: &str, model_path: Option<&str>) -> Result<ModelContext, McpError> {
        let model_context = ModelContext {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            model_name: model_name.to_string(),
            model_path: model_path.map(|s| s.to_string()),
            model_type: None,
            model_size: None,
            performance_metrics: None,
            configuration: None,
            is_active: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.model_context_repository.create(&model_context).await
    }

    async fn get_model_context(&self, id: &str) -> Result<Option<ModelContext>, McpError> {
        self.model_context_repository.get_by_id(id).await
    }

    async fn update_model_context(&self, model_context: &ModelContext) -> Result<ModelContext, McpError> {
        self.model_context_repository.update(model_context).await
    }

    async fn delete_model_context(&self, id: &str) -> Result<bool, McpError> {
        self.model_context_repository.delete(id).await
    }

    async fn list_model_contexts(&self, project_id: &str) -> Result<Vec<ModelContext>, McpError> {
        self.model_context_repository.list_by_project(project_id).await
    }

    async fn get_active_model(&self, project_id: &str) -> Result<Option<ModelContext>, McpError> {
        self.model_context_repository.get_active_model(project_id).await
    }

    async fn set_active_model(&self, project_id: &str, model_id: &str) -> Result<bool, McpError> {
        self.model_context_repository.set_active_model(project_id, model_id).await
    }

    // Code Template CRUD Implementation
    async fn create_code_template(&self, project_id: &str, template_name: &str, template_type: TemplateType, template_content: &str) -> Result<CodeTemplate, McpError> {
        let template = CodeTemplate {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            template_name: template_name.to_string(),
            template_type,
            template_content: template_content.to_string(),
            variables: Vec::new(),
            description: None,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.code_template_repository.create(&template).await
    }

    async fn get_code_template(&self, id: &str) -> Result<Option<CodeTemplate>, McpError> {
        self.code_template_repository.get_by_id(id).await
    }

    async fn get_code_template_by_name(&self, project_id: &str, template_name: &str) -> Result<Option<CodeTemplate>, McpError> {
        self.code_template_repository.get_by_name(project_id, template_name).await
    }

    async fn update_code_template(&self, template: &CodeTemplate) -> Result<CodeTemplate, McpError> {
        self.code_template_repository.update(template).await
    }

    async fn delete_code_template(&self, id: &str) -> Result<bool, McpError> {
        self.code_template_repository.delete(id).await
    }

    async fn list_code_templates(&self, project_id: &str) -> Result<Vec<CodeTemplate>, McpError> {
        self.code_template_repository.list_by_project(project_id).await
    }

    async fn list_code_templates_by_type(&self, project_id: &str, template_type: TemplateType) -> Result<Vec<CodeTemplate>, McpError> {
        let template_type_str = match template_type {
            TemplateType::Widget => "widget",
            TemplateType::Provider => "provider",
            TemplateType::Repository => "repository",
            TemplateType::Test => "test",
            TemplateType::Service => "service",
            TemplateType::Model => "model",
        };
        self.code_template_repository.list_by_template_type(project_id, template_type_str).await
    }

    // Bulk Operations Implementation
    async fn bulk_create_privacy_rules(&self, rules: &[PrivacyRule]) -> Result<Vec<PrivacyRule>, McpError> {
        self.privacy_rule_repository.bulk_create(rules).await
    }

    async fn bulk_update_privacy_rules(&self, rules: &[PrivacyRule]) -> Result<Vec<PrivacyRule>, McpError> {
        self.privacy_rule_repository.bulk_update(rules).await
    }

    async fn bulk_delete_privacy_rules(&self, ids: &[String]) -> Result<usize, McpError> {
        self.privacy_rule_repository.bulk_delete(ids).await
    }

    async fn bulk_create_code_templates(&self, templates: &[CodeTemplate]) -> Result<Vec<CodeTemplate>, McpError> {
        self.code_template_repository.bulk_create(templates).await
    }

    async fn bulk_update_code_templates(&self, templates: &[CodeTemplate]) -> Result<Vec<CodeTemplate>, McpError> {
        self.code_template_repository.bulk_update(templates).await
    }

    async fn bulk_delete_code_templates(&self, ids: &[String]) -> Result<usize, McpError> {
        self.code_template_repository.bulk_delete(ids).await
    }
}
