use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextQuery {
    pub project_id: String,
    pub feature_area: String,
    pub task_type: String, // 'implement', 'fix', 'optimize'
    pub components: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextResponse {
    pub business_rules: Vec<super::context::BusinessRule>,
    pub architectural_guidance: Vec<super::context::ArchitecturalDecision>,
    pub performance_requirements: Vec<super::context::PerformanceRequirement>,
    pub security_policies: Vec<super::context::SecurityPolicy>,
    pub conventions: Vec<super::context::ProjectConvention>,
}
