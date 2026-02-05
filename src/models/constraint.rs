/// Constraint model for operational constraints and guardrails
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    ResourceLimit,
    SafetyGuard,
    RollbackProcedure,
    ApprovalRequired,
    PerformanceTarget,
    SecurityRequirement,
}

impl ConstraintType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConstraintType::ResourceLimit => "resource_limit",
            ConstraintType::SafetyGuard => "safety_guard",
            ConstraintType::RollbackProcedure => "rollback_procedure",
            ConstraintType::ApprovalRequired => "approval_required",
            ConstraintType::PerformanceTarget => "performance_target",
            ConstraintType::SecurityRequirement => "security_requirement",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub project_id: String,
    pub constraint_type: ConstraintType,
    pub name: String,
    pub description: String,
    pub target: String, // e.g., "service:api-server", "component:database"
    pub value: String,  // e.g., "max_connections:100", "rollback:k8s_rollout_undo"
    pub severity: String, // "critical", "high", "medium", "low"
    pub enabled: bool,
    pub created_at: String,
    pub last_modified_at: String,
    pub tags: Vec<String>,
    pub enforcement_action: Option<String>, // What to do if violated
}

impl Constraint {
    pub fn new(
        project_id: String,
        constraint_type: ConstraintType,
        name: String,
        description: String,
        target: String,
        value: String,
        severity: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            constraint_type,
            name,
            description,
            target,
            value,
            severity,
            enabled: true,
            created_at: now.clone(),
            last_modified_at: now,
            tags: Vec::new(),
            enforcement_action: None,
        }
    }
}

// Dependencies/Relationships between components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Requires,
    RequiredBy,
    DependsOn,
    Blocks,
    Triggers,
    Communicates,
}

impl DependencyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DependencyType::Requires => "requires",
            DependencyType::RequiredBy => "required_by",
            DependencyType::DependsOn => "depends_on",
            DependencyType::Blocks => "blocks",
            DependencyType::Triggers => "triggers",
            DependencyType::Communicates => "communicates",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDependency {
    pub id: String,
    pub project_id: String,
    pub source_component: String,
    pub source_type: String,
    pub target_component: String,
    pub target_type: String,
    pub dependency_type: DependencyType,
    pub description: String,
    pub criticality: String, // "critical", "high", "medium", "low"
    pub impact_on_failure: Option<String>,
    pub created_at: String,
}

impl ComponentDependency {
    pub fn new(
        project_id: String,
        source_component: String,
        source_type: String,
        target_component: String,
        target_type: String,
        dependency_type: DependencyType,
        description: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            source_component,
            source_type,
            target_component,
            target_type,
            dependency_type,
            description,
            criticality: "high".to_string(),
            impact_on_failure: None,
            created_at: now,
        }
    }
}
