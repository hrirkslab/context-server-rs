use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BusinessRule {
    pub id: String,
    pub project_id: String,
    pub rule_name: String,
    pub description: Option<String>,
    pub domain_area: Option<String>,
    pub implementation_pattern: Option<String>,
    pub constraints: Option<String>, // JSON array
    pub examples: Option<String>,    // JSON array
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchitecturalDecision {
    pub id: String,
    pub project_id: String,
    pub decision_title: String,
    pub context: Option<String>,
    pub decision: Option<String>,
    pub consequences: Option<String>,
    pub alternatives_considered: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceRequirement {
    pub id: String,
    pub project_id: String,
    pub component_area: Option<String>,
    pub requirement_type: Option<String>,
    pub target_value: Option<String>,
    pub optimization_patterns: Option<String>, // JSON array
    pub avoid_patterns: Option<String>,        // JSON array
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub id: String,
    pub project_id: String,
    pub policy_name: String,
    pub policy_area: Option<String>,
    pub requirements: Option<String>,
    pub implementation_pattern: Option<String>,
    pub forbidden_patterns: Option<String>, // JSON array
    pub compliance_notes: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConvention {
    pub id: String,
    pub project_id: String,
    pub convention_type: Option<String>,
    pub convention_rule: Option<String>,
    pub good_examples: Option<String>, // JSON array
    pub bad_examples: Option<String>,  // JSON array
    pub rationale: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureContext {
    pub id: String,
    pub project_id: String,
    pub feature_name: String,
    pub business_purpose: Option<String>,
    pub user_personas: Option<String>,      // JSON array
    pub key_workflows: Option<String>,      // JSON array
    pub integration_points: Option<String>, // JSON array
    pub edge_cases: Option<String>,         // JSON array
    pub created_at: Option<String>,
}
