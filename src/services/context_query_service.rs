use crate::models::context::{
    ArchitecturalDecision, BusinessRule, PerformanceRequirement, ProjectConvention, SecurityPolicy,
};
use crate::repositories::{
    ArchitecturalDecisionRepository, BusinessRuleRepository, PerformanceRequirementRepository,
};
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

/// Result of context query
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextQueryResult {
    pub business_rules: Vec<BusinessRule>,
    pub architectural_decisions: Vec<ArchitecturalDecision>,
    pub performance_requirements: Vec<PerformanceRequirement>,
    pub security_policies: Vec<SecurityPolicy>,
    pub project_conventions: Vec<ProjectConvention>,
}

/// Service for querying project context following Single Responsibility Principle
#[async_trait]
pub trait ContextQueryService: Send + Sync {
    async fn query_context(
        &self,
        project_id: &str,
        feature_area: &str,
        task_type: &str,
        components: &[String],
    ) -> Result<ContextQueryResult, McpError>;
}

/// Implementation of ContextQueryService
pub struct ContextQueryServiceImpl<BR, ADR, PR>
where
    BR: BusinessRuleRepository,
    ADR: ArchitecturalDecisionRepository,
    PR: PerformanceRequirementRepository,
{
    business_rule_repository: BR,
    architectural_decision_repository: ADR,
    performance_requirement_repository: PR,
}

impl<BR, ADR, PR> ContextQueryServiceImpl<BR, ADR, PR>
where
    BR: BusinessRuleRepository,
    ADR: ArchitecturalDecisionRepository,
    PR: PerformanceRequirementRepository,
{
    pub fn new(
        business_rule_repository: BR,
        architectural_decision_repository: ADR,
        performance_requirement_repository: PR,
    ) -> Self {
        Self {
            business_rule_repository,
            architectural_decision_repository,
            performance_requirement_repository,
        }
    }
}

#[async_trait]
impl<BR, ADR, PR> ContextQueryService for ContextQueryServiceImpl<BR, ADR, PR>
where
    BR: BusinessRuleRepository,
    ADR: ArchitecturalDecisionRepository,
    PR: PerformanceRequirementRepository,
{
    async fn query_context(
        &self,
        project_id: &str,
        feature_area: &str,
        _task_type: &str,
        _components: &[String],
    ) -> Result<ContextQueryResult, McpError> {
        // Query business rules for the feature area
        let business_rules = self
            .business_rule_repository
            .find_by_domain_area(project_id, feature_area)
            .await?;

        // Query architectural decisions
        let architectural_decisions = self
            .architectural_decision_repository
            .find_by_project_id(project_id)
            .await?;

        // Query performance requirements
        let performance_requirements = self
            .performance_requirement_repository
            .find_by_project_id(project_id)
            .await?;

        Ok(ContextQueryResult {
            business_rules,
            architectural_decisions,
            performance_requirements,
            security_policies: Vec::new(), // TODO: Implement when security repository is available
            project_conventions: Vec::new(), // TODO: Implement when convention repository is available
        })
    }
}
