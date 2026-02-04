/// Audit log model for tracking all changes made to context entities
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    Created,
    Updated,
    Deleted,
    QueryExecuted,
    ConstraintApplied,
}

impl AuditEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditEventType::Created => "created",
            AuditEventType::Updated => "updated",
            AuditEventType::Deleted => "deleted",
            AuditEventType::QueryExecuted => "query_executed",
            AuditEventType::ConstraintApplied => "constraint_applied",
        }
    }
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub entity_type: String,
    pub entity_id: String,
    pub initiator: String, // e.g., "openclaw", "user:john", "system"
    pub previous_state: Option<serde_json::Value>,
    pub new_state: Option<serde_json::Value>,
    pub change_summary: String,
    pub project_id: Option<String>,
    pub metadata: Option<serde_json::Value>, // For additional context
}

impl AuditTrail {
    pub fn new(
        event_type: AuditEventType,
        entity_type: String,
        entity_id: String,
        initiator: String,
        change_summary: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            entity_type,
            entity_id,
            initiator,
            previous_state: None,
            new_state: None,
            change_summary,
            project_id: None,
            metadata: None,
        }
    }

    pub fn with_states(
        mut self,
        previous: Option<serde_json::Value>,
        new: Option<serde_json::Value>,
    ) -> Self {
        self.previous_state = previous;
        self.new_state = new;
        self
    }

    pub fn with_project_id(mut self, project_id: String) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}
