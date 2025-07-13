use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentPhase {
    pub id: String,
    pub project_id: String,
    pub phase_name: String,
    pub phase_order: i32,
    pub status: PhaseStatus,
    pub description: Option<String>,
    pub completion_criteria: Vec<String>,
    pub dependencies: Vec<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhaseStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

impl fmt::Display for PhaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhaseStatus::Pending => write!(f, "pending"),
            PhaseStatus::InProgress => write!(f, "in_progress"),
            PhaseStatus::Completed => write!(f, "completed"),
            PhaseStatus::Blocked => write!(f, "blocked"),
        }
    }
}
