use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Client identifier for WebSocket connections
pub type ClientId = Uuid;

/// Message identifier for tracking and acknowledgment
pub type MessageId = Uuid;

/// WebSocket message types for real-time synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Client authentication message
    Auth {
        token: Option<String>,
        project_id: String,
        client_info: ClientInfo,
    },
    /// Authentication response
    AuthResponse {
        success: bool,
        client_id: ClientId,
        message: String,
    },
    /// Subscribe to specific context changes
    Subscribe {
        filters: SyncFilters,
    },
    /// Unsubscribe from context changes
    Unsubscribe {
        filters: SyncFilters,
    },
    /// Context change notification
    ContextChange {
        message_id: MessageId,
        change: ContextChange,
        timestamp: DateTime<Utc>,
    },
    /// Acknowledgment of received message
    Ack {
        message_id: MessageId,
    },
    /// Heartbeat/ping message
    Ping {
        timestamp: DateTime<Utc>,
    },
    /// Heartbeat/pong response
    Pong {
        timestamp: DateTime<Utc>,
    },
    /// Error message
    Error {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },
}

/// Client information for connection management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub user_agent: Option<String>,
    pub client_type: ClientType,
    pub version: String,
}

/// Type of client connecting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientType {
    AIAgent,
    IDE,
    WebInterface,
    CLI,
    Other(String),
}

/// Filters for subscribing to specific context changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFilters {
    pub project_ids: Option<Vec<String>>,
    pub entity_types: Option<Vec<String>>,
    pub feature_areas: Option<Vec<String>>,
    pub change_types: Option<Vec<ChangeType>>,
}

/// Types of context changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Create,
    Update,
    Delete,
    Bulk,
}

/// Context change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextChange {
    pub change_id: Uuid,
    pub change_type: ChangeType,
    pub entity_type: String,
    pub entity_id: String,
    pub project_id: String,
    pub feature_area: Option<String>,
    pub delta: Option<serde_json::Value>,
    pub full_entity: Option<serde_json::Value>,
    pub metadata: ChangeMetadata,
}

/// Metadata about the change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeMetadata {
    pub user_id: Option<String>,
    pub client_id: ClientId,
    pub timestamp: DateTime<Utc>,
    pub version: u32,
    pub conflict_resolution: Option<ConflictResolution>,
}

/// Conflict resolution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub strategy: ConflictStrategy,
    pub resolved_by: String,
    pub original_changes: Vec<serde_json::Value>,
}

/// Strategies for resolving conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictStrategy {
    LastWriterWins,
    ManualResolution,
    AutoMerge,
    Reject,
}

/// Connection status for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub client_id: ClientId,
    pub connected_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub project_id: String,
    pub subscriptions: Vec<SyncFilters>,
    pub message_queue_size: usize,
    pub is_healthy: bool,
}

/// Sync status for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub project_id: String,
    pub connected_clients: u32,
    pub pending_changes: u32,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_health: SyncHealth,
}

/// Health status of synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncHealth {
    Healthy,
    Degraded,
    Unhealthy,
}

impl Default for SyncFilters {
    fn default() -> Self {
        Self {
            project_ids: None,
            entity_types: None,
            feature_areas: None,
            change_types: None,
        }
    }
}

impl SyncFilters {
    /// Check if a context change matches these filters
    pub fn matches(&self, change: &ContextChange) -> bool {
        // Check project filter
        if let Some(ref project_ids) = self.project_ids {
            if !project_ids.contains(&change.project_id) {
                return false;
            }
        }

        // Check entity type filter
        if let Some(ref entity_types) = self.entity_types {
            if !entity_types.contains(&change.entity_type) {
                return false;
            }
        }

        // Check feature area filter
        if let Some(ref feature_areas) = self.feature_areas {
            if let Some(ref change_feature_area) = change.feature_area {
                if !feature_areas.contains(change_feature_area) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check change type filter
        if let Some(ref change_types) = self.change_types {
            if !change_types.contains(&change.change_type) {
                return false;
            }
        }

        true
    }
}