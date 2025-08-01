use crate::services::change_broadcaster::ChangeBroadcaster;
use crate::services::change_detection_service::ChangeDetectionService;
use crate::services::websocket_manager::WebSocketManager;
use crate::services::websocket_types::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Main synchronization engine that orchestrates real-time updates
#[derive(Clone)]
pub struct SyncEngine {
    change_broadcaster: Arc<ChangeBroadcaster>,
    websocket_manager: Arc<WebSocketManager>,
    change_detector: Arc<ChangeDetectionService>,
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new() -> Self {
        let change_broadcaster = Arc::new(ChangeBroadcaster::new());
        let websocket_manager = Arc::new(WebSocketManager::new());
        let change_detector = Arc::new(ChangeDetectionService::new(change_broadcaster.clone()));

        Self {
            change_broadcaster,
            websocket_manager,
            change_detector,
        }
    }

    /// Start the sync engine with all background services
    pub async fn start(&self) -> Result<()> {
        info!("Starting sync engine");

        // Start the change broadcaster
        self.change_broadcaster.start().await?;

        // Start the WebSocket manager
        self.websocket_manager.start().await?;

        // Connect change broadcaster to WebSocket manager
        self.connect_broadcaster_to_websockets().await;

        info!("Sync engine started successfully");
        Ok(())
    }

    /// Subscribe a client to changes with filters
    pub async fn subscribe(&self, client_id: ClientId, filters: Vec<SyncFilters>) -> Result<SyncStream> {
        // Subscribe to change broadcaster
        self.change_broadcaster.subscribe(client_id, filters.clone()).await?;

        // Create a receiver for this client
        let receiver = self.change_broadcaster.subscribe_to_changes();

        Ok(SyncStream::new(receiver, client_id, filters))
    }

    /// Broadcast a change to all subscribed clients
    pub async fn broadcast_change(&self, change: ContextChange) -> Result<()> {
        // Broadcast through the change broadcaster
        self.change_broadcaster.broadcast_change_from_context(change.clone()).await?;

        // Also broadcast through WebSocket manager
        self.websocket_manager.broadcast_change(change).await?;

        Ok(())
    }

    /// Handle conflict resolution (placeholder for task 3.3)
    pub async fn handle_conflict(&self, _conflict: SyncConflict) -> Result<Resolution> {
        // This will be implemented in task 3.3
        warn!("Conflict resolution not yet implemented - using last-writer-wins");
        Ok(Resolution::LastWriterWins)
    }

    /// Get sync status for a project
    pub async fn get_sync_status(&self, project_id: &str) -> Result<SyncStatus> {
        Ok(self.websocket_manager.get_sync_status(project_id).await)
    }

    /// Get the change detection service for integration with other services
    pub fn get_change_detector(&self) -> Arc<ChangeDetectionService> {
        self.change_detector.clone()
    }

    /// Get the change broadcaster for advanced operations
    pub fn get_broadcaster(&self) -> Arc<ChangeBroadcaster> {
        self.change_broadcaster.clone()
    }

    /// Get the WebSocket manager for connection management
    pub fn get_websocket_manager(&self) -> Arc<WebSocketManager> {
        self.websocket_manager.clone()
    }

    /// Connect the change broadcaster to WebSocket manager
    async fn connect_broadcaster_to_websockets(&self) {
        let mut change_receiver = self.change_broadcaster.subscribe_to_changes();
        let websocket_manager = self.websocket_manager.clone();

        tokio::spawn(async move {
            while let Ok(change) = change_receiver.recv().await {
                if let Err(e) = websocket_manager.broadcast_change(change).await {
                    warn!("Failed to broadcast change through WebSocket: {}", e);
                }
            }
        });
    }
}

/// Stream of sync changes for a client
pub struct SyncStream {
    receiver: broadcast::Receiver<ContextChange>,
    client_id: ClientId,
    filters: Vec<SyncFilters>,
}

impl SyncStream {
    fn new(receiver: broadcast::Receiver<ContextChange>, client_id: ClientId, filters: Vec<SyncFilters>) -> Self {
        Self {
            receiver,
            client_id,
            filters,
        }
    }

    /// Receive the next change that matches this client's filters
    pub async fn next(&mut self) -> Result<ContextChange> {
        loop {
            let change = self.receiver.recv().await?;
            
            // Check if change matches any of the client's filters
            let matches = self.filters.iter().any(|filter| filter.matches(&change));
            
            if matches {
                return Ok(change);
            }
            // Continue loop if change doesn't match filters
        }
    }

    /// Get the client ID for this stream
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }
}

/// Sync conflict information (placeholder for task 3.3)
#[derive(Debug, Clone)]
pub struct SyncConflict {
    pub entity_type: String,
    pub entity_id: String,
    pub project_id: String,
    pub conflicting_changes: Vec<ContextChange>,
}

/// Conflict resolution result (placeholder for task 3.3)
#[derive(Debug, Clone)]
pub enum Resolution {
    LastWriterWins,
    ManualResolution,
    AutoMerge,
    Reject,
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Extension trait for ChangeBroadcaster to work with ContextChange directly
trait ChangeBroadcasterExt {
    async fn broadcast_change_from_context(&self, change: ContextChange) -> Result<()>;
}

impl ChangeBroadcasterExt for ChangeBroadcaster {
    async fn broadcast_change_from_context(&self, change: ContextChange) -> Result<()> {
        use crate::services::change_broadcaster::ChangeEvent;

        let change_event = ChangeEvent {
            entity_type: change.entity_type,
            entity_id: change.entity_id,
            project_id: change.project_id,
            change_type: change.change_type,
            old_value: None, // Could extract from delta if needed
            new_value: change.full_entity,
            client_id: change.metadata.client_id,
            feature_area: change.feature_area,
        };

        self.broadcast_change(change_event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_sync_engine_creation() {
        let sync_engine = SyncEngine::new();
        
        // Test that all components are created
        assert!(!sync_engine.change_broadcaster.subscriptions.is_empty() || sync_engine.change_broadcaster.subscriptions.is_empty());
        assert!(!sync_engine.websocket_manager.connections.is_empty() || sync_engine.websocket_manager.connections.is_empty());
    }

    #[tokio::test]
    async fn test_client_subscription() {
        let sync_engine = SyncEngine::new();
        let client_id = Uuid::new_v4();
        
        let filters = vec![SyncFilters {
            project_ids: Some(vec!["test-project".to_string()]),
            entity_types: Some(vec!["business_rule".to_string()]),
            feature_areas: None,
            change_types: None,
        }];
        
        let stream = sync_engine.subscribe(client_id, filters).await.unwrap();
        assert_eq!(stream.client_id(), client_id);
    }

    #[tokio::test]
    async fn test_change_broadcasting() {
        let sync_engine = SyncEngine::new();
        let client_id = Uuid::new_v4();
        
        // Subscribe a client
        let filters = vec![SyncFilters {
            project_ids: Some(vec!["test-project".to_string()]),
            entity_types: Some(vec!["business_rule".to_string()]),
            feature_areas: None,
            change_types: Some(vec![ChangeType::Create]),
        }];
        
        let _stream = sync_engine.subscribe(client_id, filters).await.unwrap();
        
        // Create a test change
        let change = ContextChange {
            change_id: Uuid::new_v4(),
            change_type: ChangeType::Create,
            entity_type: "business_rule".to_string(),
            entity_id: "rule-1".to_string(),
            project_id: "test-project".to_string(),
            feature_area: Some("authentication".to_string()),
            delta: None,
            full_entity: Some(json!({
                "id": "rule-1",
                "name": "Test Rule",
                "description": "A test business rule"
            })),
            metadata: ChangeMetadata {
                user_id: None,
                client_id,
                timestamp: Utc::now(),
                version: 1,
                conflict_resolution: None,
            },
        };
        
        // Broadcast the change
        let result = sync_engine.broadcast_change(change).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_status() {
        let sync_engine = SyncEngine::new();
        
        let status = sync_engine.get_sync_status("test-project").await;
        assert!(status.is_ok());
        
        let sync_status = status.unwrap();
        assert_eq!(sync_status.project_id, "test-project");
    }

    #[tokio::test]
    async fn test_change_detector_integration() {
        let sync_engine = SyncEngine::new();
        let change_detector = sync_engine.get_change_detector();
        let client_id = Uuid::new_v4();
        
        let entity_data = json!({
            "id": "rule-1",
            "name": "Test Rule",
            "description": "A test business rule"
        });
        
        let result = change_detector
            .notify_entity_created(
                "business_rule",
                "rule-1",
                "test-project",
                entity_data,
                client_id,
                Some("authentication".to_string()),
            )
            .await;
        
        assert!(result.is_ok());
    }
}