use async_trait::async_trait;
use crate::models::context::{SecurityPolicy, ProjectConvention, FeatureContext};
use crate::repositories::{SecurityPolicyRepository, ProjectConventionRepository, FeatureContextRepository};
use rmcp::model::ErrorData as McpError;
use uuid::Uuid;

/// Extended CRUD service for security policies, project conventions, and feature contexts
#[async_trait]
pub trait ExtendedContextCrudService: Send + Sync {
    // Security Policy CRUD
    async fn create_security_policy(&self, project_id: &str, policy_name: &str, policy_area: Option<&str>) -> Result<SecurityPolicy, McpError>;
    async fn get_security_policy(&self, id: &str) -> Result<Option<SecurityPolicy>, McpError>;
    async fn update_security_policy(&self, policy: &SecurityPolicy) -> Result<SecurityPolicy, McpError>;
    async fn delete_security_policy(&self, id: &str) -> Result<bool, McpError>;
    async fn list_security_policies(&self, project_id: &str) -> Result<Vec<SecurityPolicy>, McpError>;
    async fn list_security_policies_by_area(&self, project_id: &str, policy_area: &str) -> Result<Vec<SecurityPolicy>, McpError>;

    // Project Convention CRUD
    async fn create_project_convention(&self, project_id: &str, convention_type: Option<&str>, convention_rule: Option<&str>) -> Result<ProjectConvention, McpError>;
    async fn get_project_convention(&self, id: &str) -> Result<Option<ProjectConvention>, McpError>;
    async fn update_project_convention(&self, convention: &ProjectConvention) -> Result<ProjectConvention, McpError>;
    async fn delete_project_convention(&self, id: &str) -> Result<bool, McpError>;
    async fn list_project_conventions(&self, project_id: &str) -> Result<Vec<ProjectConvention>, McpError>;
    async fn list_project_conventions_by_type(&self, project_id: &str, convention_type: &str) -> Result<Vec<ProjectConvention>, McpError>;

    // Feature Context CRUD
    async fn create_feature_context(&self, project_id: &str, feature_name: &str, business_purpose: Option<&str>) -> Result<FeatureContext, McpError>;
    async fn get_feature_context(&self, id: &str) -> Result<Option<FeatureContext>, McpError>;
    async fn get_feature_context_by_name(&self, project_id: &str, feature_name: &str) -> Result<Option<FeatureContext>, McpError>;
    async fn update_feature_context(&self, feature_context: &FeatureContext) -> Result<FeatureContext, McpError>;
    async fn delete_feature_context(&self, id: &str) -> Result<bool, McpError>;
    async fn list_feature_contexts(&self, project_id: &str) -> Result<Vec<FeatureContext>, McpError>;

    // Bulk operations
    async fn bulk_create_security_policies(&self, policies: &[SecurityPolicy]) -> Result<Vec<SecurityPolicy>, McpError>;
    async fn bulk_update_security_policies(&self, policies: &[SecurityPolicy]) -> Result<Vec<SecurityPolicy>, McpError>;
    async fn bulk_delete_security_policies(&self, ids: &[String]) -> Result<usize, McpError>;

    async fn bulk_create_project_conventions(&self, conventions: &[ProjectConvention]) -> Result<Vec<ProjectConvention>, McpError>;
    async fn bulk_update_project_conventions(&self, conventions: &[ProjectConvention]) -> Result<Vec<ProjectConvention>, McpError>;
    async fn bulk_delete_project_conventions(&self, ids: &[String]) -> Result<usize, McpError>;

    async fn bulk_create_feature_contexts(&self, contexts: &[FeatureContext]) -> Result<Vec<FeatureContext>, McpError>;
    async fn bulk_update_feature_contexts(&self, contexts: &[FeatureContext]) -> Result<Vec<FeatureContext>, McpError>;
    async fn bulk_delete_feature_contexts(&self, ids: &[String]) -> Result<usize, McpError>;
}

/// Implementation of ExtendedContextCrudService
pub struct ExtendedContextCrudServiceImpl<SPR, PCR, FCR>
where
    SPR: SecurityPolicyRepository,
    PCR: ProjectConventionRepository,
    FCR: FeatureContextRepository,
{
    security_policy_repository: SPR,
    project_convention_repository: PCR,
    feature_context_repository: FCR,
}

impl<SPR, PCR, FCR> ExtendedContextCrudServiceImpl<SPR, PCR, FCR>
where
    SPR: SecurityPolicyRepository,
    PCR: ProjectConventionRepository,
    FCR: FeatureContextRepository,
{
    pub fn new(
        security_policy_repository: SPR,
        project_convention_repository: PCR,
        feature_context_repository: FCR,
    ) -> Self {
        Self {
            security_policy_repository,
            project_convention_repository,
            feature_context_repository,
        }
    }
}

#[async_trait]
impl<SPR, PCR, FCR> ExtendedContextCrudService for ExtendedContextCrudServiceImpl<SPR, PCR, FCR>
where
    SPR: SecurityPolicyRepository,
    PCR: ProjectConventionRepository,
    FCR: FeatureContextRepository,
{
    // Security Policy CRUD Implementation
    async fn create_security_policy(&self, project_id: &str, policy_name: &str, policy_area: Option<&str>) -> Result<SecurityPolicy, McpError> {
        let policy = SecurityPolicy {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            policy_name: policy_name.to_string(),
            policy_area: policy_area.map(|s| s.to_string()),
            requirements: None,
            implementation_pattern: None,
            forbidden_patterns: None,
            compliance_notes: None,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.security_policy_repository.create(&policy).await
    }

    async fn get_security_policy(&self, id: &str) -> Result<Option<SecurityPolicy>, McpError> {
        self.security_policy_repository.get_by_id(id).await
    }

    async fn update_security_policy(&self, policy: &SecurityPolicy) -> Result<SecurityPolicy, McpError> {
        self.security_policy_repository.update(policy).await
    }

    async fn delete_security_policy(&self, id: &str) -> Result<bool, McpError> {
        self.security_policy_repository.delete(id).await
    }

    async fn list_security_policies(&self, project_id: &str) -> Result<Vec<SecurityPolicy>, McpError> {
        self.security_policy_repository.list_by_project(project_id).await
    }

    async fn list_security_policies_by_area(&self, project_id: &str, policy_area: &str) -> Result<Vec<SecurityPolicy>, McpError> {
        self.security_policy_repository.list_by_policy_area(project_id, policy_area).await
    }

    // Project Convention CRUD Implementation
    async fn create_project_convention(&self, project_id: &str, convention_type: Option<&str>, convention_rule: Option<&str>) -> Result<ProjectConvention, McpError> {
        let convention = ProjectConvention {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            convention_type: convention_type.map(|s| s.to_string()),
            convention_rule: convention_rule.map(|s| s.to_string()),
            good_examples: None,
            bad_examples: None,
            rationale: None,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.project_convention_repository.create(&convention).await
    }

    async fn get_project_convention(&self, id: &str) -> Result<Option<ProjectConvention>, McpError> {
        self.project_convention_repository.get_by_id(id).await
    }

    async fn update_project_convention(&self, convention: &ProjectConvention) -> Result<ProjectConvention, McpError> {
        self.project_convention_repository.update(convention).await
    }

    async fn delete_project_convention(&self, id: &str) -> Result<bool, McpError> {
        self.project_convention_repository.delete(id).await
    }

    async fn list_project_conventions(&self, project_id: &str) -> Result<Vec<ProjectConvention>, McpError> {
        self.project_convention_repository.list_by_project(project_id).await
    }

    async fn list_project_conventions_by_type(&self, project_id: &str, convention_type: &str) -> Result<Vec<ProjectConvention>, McpError> {
        self.project_convention_repository.list_by_convention_type(project_id, convention_type).await
    }

    // Feature Context CRUD Implementation
    async fn create_feature_context(&self, project_id: &str, feature_name: &str, business_purpose: Option<&str>) -> Result<FeatureContext, McpError> {
        let feature_context = FeatureContext {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            feature_name: feature_name.to_string(),
            business_purpose: business_purpose.map(|s| s.to_string()),
            user_personas: None,
            key_workflows: None,
            integration_points: None,
            edge_cases: None,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.feature_context_repository.create(&feature_context).await
    }

    async fn get_feature_context(&self, id: &str) -> Result<Option<FeatureContext>, McpError> {
        self.feature_context_repository.get_by_id(id).await
    }

    async fn get_feature_context_by_name(&self, project_id: &str, feature_name: &str) -> Result<Option<FeatureContext>, McpError> {
        self.feature_context_repository.get_by_feature_name(project_id, feature_name).await
    }

    async fn update_feature_context(&self, feature_context: &FeatureContext) -> Result<FeatureContext, McpError> {
        self.feature_context_repository.update(feature_context).await
    }

    async fn delete_feature_context(&self, id: &str) -> Result<bool, McpError> {
        self.feature_context_repository.delete(id).await
    }

    async fn list_feature_contexts(&self, project_id: &str) -> Result<Vec<FeatureContext>, McpError> {
        self.feature_context_repository.list_by_project(project_id).await
    }

    // Bulk Operations Implementation
    async fn bulk_create_security_policies(&self, policies: &[SecurityPolicy]) -> Result<Vec<SecurityPolicy>, McpError> {
        self.security_policy_repository.bulk_create(policies).await
    }

    async fn bulk_update_security_policies(&self, policies: &[SecurityPolicy]) -> Result<Vec<SecurityPolicy>, McpError> {
        self.security_policy_repository.bulk_update(policies).await
    }

    async fn bulk_delete_security_policies(&self, ids: &[String]) -> Result<usize, McpError> {
        self.security_policy_repository.bulk_delete(ids).await
    }

    async fn bulk_create_project_conventions(&self, conventions: &[ProjectConvention]) -> Result<Vec<ProjectConvention>, McpError> {
        self.project_convention_repository.bulk_create(conventions).await
    }

    async fn bulk_update_project_conventions(&self, conventions: &[ProjectConvention]) -> Result<Vec<ProjectConvention>, McpError> {
        self.project_convention_repository.bulk_update(conventions).await
    }

    async fn bulk_delete_project_conventions(&self, ids: &[String]) -> Result<usize, McpError> {
        self.project_convention_repository.bulk_delete(ids).await
    }

    async fn bulk_create_feature_contexts(&self, contexts: &[FeatureContext]) -> Result<Vec<FeatureContext>, McpError> {
        self.feature_context_repository.bulk_create(contexts).await
    }

    async fn bulk_update_feature_contexts(&self, contexts: &[FeatureContext]) -> Result<Vec<FeatureContext>, McpError> {
        self.feature_context_repository.bulk_update(contexts).await
    }

    async fn bulk_delete_feature_contexts(&self, ids: &[String]) -> Result<usize, McpError> {
        self.feature_context_repository.bulk_delete(ids).await
    }
}
