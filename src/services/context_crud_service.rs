use async_trait::async_trait;
use crate::models::context::{BusinessRule, ArchitecturalDecision, PerformanceRequirement};
use crate::repositories::{BusinessRuleRepository, ArchitecturalDecisionRepository, PerformanceRequirementRepository};
use rmcp::model::ErrorData as McpError;
use uuid::Uuid;

/// Enhanced CRUD service for business rules, architectural decisions, and performance requirements
#[async_trait]
pub trait ContextCrudService: Send + Sync {
    // Business Rules CRUD
    async fn create_business_rule(&self, project_id: &str, rule_name: &str, description: Option<&str>, domain_area: Option<&str>) -> Result<BusinessRule, McpError>;
    async fn get_business_rule(&self, id: &str) -> Result<Option<BusinessRule>, McpError>;
    async fn update_business_rule(&self, rule: &BusinessRule) -> Result<BusinessRule, McpError>;
    async fn delete_business_rule(&self, id: &str) -> Result<bool, McpError>;
    async fn list_business_rules(&self, project_id: &str) -> Result<Vec<BusinessRule>, McpError>;
    async fn list_business_rules_by_domain(&self, project_id: &str, domain_area: &str) -> Result<Vec<BusinessRule>, McpError>;

    // Architectural Decisions CRUD
    async fn create_architectural_decision(&self, project_id: &str, decision_title: &str, context: Option<&str>, decision: Option<&str>) -> Result<ArchitecturalDecision, McpError>;
    async fn get_architectural_decision(&self, id: &str) -> Result<Option<ArchitecturalDecision>, McpError>;
    async fn update_architectural_decision(&self, decision: &ArchitecturalDecision) -> Result<ArchitecturalDecision, McpError>;
    async fn delete_architectural_decision(&self, id: &str) -> Result<bool, McpError>;
    async fn list_architectural_decisions(&self, project_id: &str) -> Result<Vec<ArchitecturalDecision>, McpError>;

    // Performance Requirements CRUD
    async fn create_performance_requirement(&self, project_id: &str, component_area: Option<&str>, requirement_type: Option<&str>, target_value: Option<&str>) -> Result<PerformanceRequirement, McpError>;
    async fn get_performance_requirement(&self, id: &str) -> Result<Option<PerformanceRequirement>, McpError>;
    async fn update_performance_requirement(&self, requirement: &PerformanceRequirement) -> Result<PerformanceRequirement, McpError>;
    async fn delete_performance_requirement(&self, id: &str) -> Result<bool, McpError>;
    async fn list_performance_requirements(&self, project_id: &str) -> Result<Vec<PerformanceRequirement>, McpError>;

    // Bulk operations
    async fn bulk_create_business_rules(&self, rules: &[BusinessRule]) -> Result<Vec<BusinessRule>, McpError>;
    async fn bulk_update_business_rules(&self, rules: &[BusinessRule]) -> Result<Vec<BusinessRule>, McpError>;
    async fn bulk_delete_business_rules(&self, ids: &[String]) -> Result<usize, McpError>;
}

/// Implementation of ContextCrudService
pub struct ContextCrudServiceImpl<BR, ADR, PR> 
where 
    BR: BusinessRuleRepository,
    ADR: ArchitecturalDecisionRepository,
    PR: PerformanceRequirementRepository,
{
    business_rule_repository: BR,
    architectural_decision_repository: ADR,
    performance_requirement_repository: PR,
}

impl<BR, ADR, PR> ContextCrudServiceImpl<BR, ADR, PR>
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
impl<BR, ADR, PR> ContextCrudService for ContextCrudServiceImpl<BR, ADR, PR>
where 
    BR: BusinessRuleRepository,
    ADR: ArchitecturalDecisionRepository,
    PR: PerformanceRequirementRepository,
{
    // Business Rules CRUD Implementation
    async fn create_business_rule(&self, project_id: &str, rule_name: &str, description: Option<&str>, domain_area: Option<&str>) -> Result<BusinessRule, McpError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        let rule = BusinessRule {
            id,
            project_id: project_id.to_string(),
            rule_name: rule_name.to_string(),
            description: description.map(|s| s.to_string()),
            domain_area: domain_area.map(|s| s.to_string()),
            implementation_pattern: None,
            constraints: None,
            examples: None,
            created_at: Some(now),
        };

        self.business_rule_repository.create(&rule).await
    }

    async fn get_business_rule(&self, id: &str) -> Result<Option<BusinessRule>, McpError> {
        self.business_rule_repository.find_by_id(id).await
    }

    async fn update_business_rule(&self, rule: &BusinessRule) -> Result<BusinessRule, McpError> {
        self.business_rule_repository.update(rule).await
    }

    async fn delete_business_rule(&self, id: &str) -> Result<bool, McpError> {
        self.business_rule_repository.delete(id).await
    }

    async fn list_business_rules(&self, project_id: &str) -> Result<Vec<BusinessRule>, McpError> {
        self.business_rule_repository.find_by_project_id(project_id).await
    }

    async fn list_business_rules_by_domain(&self, project_id: &str, domain_area: &str) -> Result<Vec<BusinessRule>, McpError> {
        self.business_rule_repository.find_by_domain_area(project_id, domain_area).await
    }

    // Architectural Decisions CRUD Implementation
    async fn create_architectural_decision(&self, project_id: &str, decision_title: &str, context: Option<&str>, decision: Option<&str>) -> Result<ArchitecturalDecision, McpError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        let arch_decision = ArchitecturalDecision {
            id,
            project_id: project_id.to_string(),
            decision_title: decision_title.to_string(),
            context: context.map(|s| s.to_string()),
            decision: decision.map(|s| s.to_string()),
            consequences: None,
            alternatives_considered: None,
            status: Some("proposed".to_string()),
            created_at: Some(now),
        };

        self.architectural_decision_repository.create(&arch_decision).await
    }

    async fn get_architectural_decision(&self, id: &str) -> Result<Option<ArchitecturalDecision>, McpError> {
        self.architectural_decision_repository.find_by_id(id).await
    }

    async fn update_architectural_decision(&self, decision: &ArchitecturalDecision) -> Result<ArchitecturalDecision, McpError> {
        self.architectural_decision_repository.update(decision).await
    }

    async fn delete_architectural_decision(&self, id: &str) -> Result<bool, McpError> {
        self.architectural_decision_repository.delete(id).await
    }

    async fn list_architectural_decisions(&self, project_id: &str) -> Result<Vec<ArchitecturalDecision>, McpError> {
        self.architectural_decision_repository.find_by_project_id(project_id).await
    }

    // Performance Requirements CRUD Implementation
    async fn create_performance_requirement(&self, project_id: &str, component_area: Option<&str>, requirement_type: Option<&str>, target_value: Option<&str>) -> Result<PerformanceRequirement, McpError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        let requirement = PerformanceRequirement {
            id,
            project_id: project_id.to_string(),
            component_area: component_area.map(|s| s.to_string()),
            requirement_type: requirement_type.map(|s| s.to_string()),
            target_value: target_value.map(|s| s.to_string()),
            optimization_patterns: None,
            avoid_patterns: None,
            created_at: Some(now),
        };

        self.performance_requirement_repository.create(&requirement).await
    }

    async fn get_performance_requirement(&self, id: &str) -> Result<Option<PerformanceRequirement>, McpError> {
        self.performance_requirement_repository.find_by_id(id).await
    }

    async fn update_performance_requirement(&self, requirement: &PerformanceRequirement) -> Result<PerformanceRequirement, McpError> {
        self.performance_requirement_repository.update(requirement).await
    }

    async fn delete_performance_requirement(&self, id: &str) -> Result<bool, McpError> {
        self.performance_requirement_repository.delete(id).await
    }

    async fn list_performance_requirements(&self, project_id: &str) -> Result<Vec<PerformanceRequirement>, McpError> {
        self.performance_requirement_repository.find_by_project_id(project_id).await
    }

    // Bulk Operations Implementation
    async fn bulk_create_business_rules(&self, rules: &[BusinessRule]) -> Result<Vec<BusinessRule>, McpError> {
        let mut created_rules = Vec::new();
        for rule in rules {
            let created = self.business_rule_repository.create(rule).await?;
            created_rules.push(created);
        }
        Ok(created_rules)
    }

    async fn bulk_update_business_rules(&self, rules: &[BusinessRule]) -> Result<Vec<BusinessRule>, McpError> {
        let mut updated_rules = Vec::new();
        for rule in rules {
            let updated = self.business_rule_repository.update(rule).await?;
            updated_rules.push(updated);
        }
        Ok(updated_rules)
    }

    async fn bulk_delete_business_rules(&self, ids: &[String]) -> Result<usize, McpError> {
        let mut deleted_count = 0;
        for id in ids {
            if self.business_rule_repository.delete(id).await? {
                deleted_count += 1;
            }
        }
        Ok(deleted_count)
    }
}
