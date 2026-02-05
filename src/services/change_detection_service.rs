use crate::services::change_broadcaster::{ChangeBroadcaster, ChangeEvent};
use crate::services::websocket_types::{ChangeType, ClientId};
use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;

/// Service for detecting and broadcasting context changes
#[derive(Clone)]
pub struct ChangeDetectionService {
    broadcaster: Arc<ChangeBroadcaster>,
}

impl ChangeDetectionService {
    /// Create a new change detection service
    pub fn new(broadcaster: Arc<ChangeBroadcaster>) -> Self {
        Self { broadcaster }
    }

    /// Notify about a context entity creation
    pub async fn notify_entity_created(
        &self,
        entity_type: &str,
        entity_id: &str,
        project_id: &str,
        entity_data: Value,
        client_id: ClientId,
        feature_area: Option<String>,
    ) -> Result<()> {
        debug!("Notifying entity created: {}/{}", entity_type, entity_id);

        let change_event = ChangeEvent {
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            project_id: project_id.to_string(),
            change_type: ChangeType::Create,
            old_value: None,
            new_value: Some(entity_data),
            client_id,
            feature_area,
        };

        self.broadcaster.broadcast_change(change_event).await?;
        Ok(())
    }

    /// Notify about a context entity update
    pub async fn notify_entity_updated(
        &self,
        entity_type: &str,
        entity_id: &str,
        project_id: &str,
        old_data: Value,
        new_data: Value,
        client_id: ClientId,
        feature_area: Option<String>,
    ) -> Result<()> {
        debug!("Notifying entity updated: {}/{}", entity_type, entity_id);

        let change_event = ChangeEvent {
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            project_id: project_id.to_string(),
            change_type: ChangeType::Update,
            old_value: Some(old_data),
            new_value: Some(new_data),
            client_id,
            feature_area,
        };

        self.broadcaster.broadcast_change(change_event).await?;
        Ok(())
    }

    /// Notify about a context entity deletion
    pub async fn notify_entity_deleted(
        &self,
        entity_type: &str,
        entity_id: &str,
        project_id: &str,
        old_data: Value,
        client_id: ClientId,
        feature_area: Option<String>,
    ) -> Result<()> {
        debug!("Notifying entity deleted: {}/{}", entity_type, entity_id);

        let change_event = ChangeEvent {
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            project_id: project_id.to_string(),
            change_type: ChangeType::Delete,
            old_value: Some(old_data),
            new_value: None,
            client_id,
            feature_area,
        };

        self.broadcaster.broadcast_change(change_event).await?;
        Ok(())
    }

    /// Notify about bulk operations
    pub async fn notify_bulk_operation(
        &self,
        entity_type: &str,
        project_id: &str,
        operation_summary: Value,
        client_id: ClientId,
        feature_area: Option<String>,
    ) -> Result<()> {
        debug!("Notifying bulk operation: {} in project {}", entity_type, project_id);

        let change_event = ChangeEvent {
            entity_type: entity_type.to_string(),
            entity_id: format!("bulk_{}", Uuid::new_v4()),
            project_id: project_id.to_string(),
            change_type: ChangeType::Bulk,
            old_value: None,
            new_value: Some(operation_summary),
            client_id,
            feature_area,
        };

        self.broadcaster.broadcast_change(change_event).await?;
        Ok(())
    }

    /// Get the underlying broadcaster for advanced operations
    pub fn get_broadcaster(&self) -> Arc<ChangeBroadcaster> {
        self.broadcaster.clone()
    }
}

/// Trait for services that can emit change events
pub trait ChangeEmitter {
    /// Get the change detection service
    fn get_change_detector(&self) -> Option<&ChangeDetectionService>;

    /// Emit a creation event
    async fn emit_created(
        &self,
        entity_type: &str,
        entity_id: &str,
        project_id: &str,
        entity_data: Value,
        client_id: ClientId,
        feature_area: Option<String>,
    ) -> Result<()> {
        if let Some(detector) = self.get_change_detector() {
            detector
                .notify_entity_created(entity_type, entity_id, project_id, entity_data, client_id, feature_area)
                .await
                .map_err(|e| {
                    error!("Failed to emit created event: {}", e);
                    e
                })?;
        }
        Ok(())
    }

    /// Emit an update event
    async fn emit_updated(
        &self,
        entity_type: &str,
        entity_id: &str,
        project_id: &str,
        old_data: Value,
        new_data: Value,
        client_id: ClientId,
        feature_area: Option<String>,
    ) -> Result<()> {
        if let Some(detector) = self.get_change_detector() {
            detector
                .notify_entity_updated(entity_type, entity_id, project_id, old_data, new_data, client_id, feature_area)
                .await
                .map_err(|e| {
                    error!("Failed to emit updated event: {}", e);
                    e
                })?;
        }
        Ok(())
    }

    /// Emit a deletion event
    async fn emit_deleted(
        &self,
        entity_type: &str,
        entity_id: &str,
        project_id: &str,
        old_data: Value,
        client_id: ClientId,
        feature_area: Option<String>,
    ) -> Result<()> {
        if let Some(detector) = self.get_change_detector() {
            detector
                .notify_entity_deleted(entity_type, entity_id, project_id, old_data, client_id, feature_area)
                .await
                .map_err(|e| {
                    error!("Failed to emit deleted event: {}", e);
                    e
                })?;
        }
        Ok(())
    }

    /// Emit a bulk operation event
    async fn emit_bulk_operation(
        &self,
        entity_type: &str,
        project_id: &str,
        operation_summary: Value,
        client_id: ClientId,
        feature_area: Option<String>,
    ) -> Result<()> {
        if let Some(detector) = self.get_change_detector() {
            detector
                .notify_bulk_operation(entity_type, project_id, operation_summary, client_id, feature_area)
                .await
                .map_err(|e| {
                    error!("Failed to emit bulk operation event: {}", e);
                    e
                })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::change_broadcaster::ChangeBroadcaster;
    use serde_json::json;

    #[tokio::test]
    async fn test_change_detection_service_creation() {
        let broadcaster = Arc::new(ChangeBroadcaster::new());
        let detector = ChangeDetectionService::new(broadcaster.clone());
        
        assert!(Arc::ptr_eq(&detector.get_broadcaster(), &broadcaster));
    }

    #[tokio::test]
    async fn test_notify_entity_created() {
        let broadcaster = Arc::new(ChangeBroadcaster::new());
        let detector = ChangeDetectionService::new(broadcaster.clone());
        let client_id = Uuid::new_v4();

        let entity_data = json!({
            "id": "rule-1",
            "name": "Test Rule",
            "description": "A test business rule"
        });

        let result = detector
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

    #[tokio::test]
    async fn test_notify_entity_updated() {
        let broadcaster = Arc::new(ChangeBroadcaster::new());
        let detector = ChangeDetectionService::new(broadcaster.clone());
        let client_id = Uuid::new_v4();

        let old_data = json!({
            "id": "rule-1",
            "name": "Old Rule",
            "description": "Old description"
        });

        let new_data = json!({
            "id": "rule-1",
            "name": "New Rule",
            "description": "New description"
        });

        let result = detector
            .notify_entity_updated(
                "business_rule",
                "rule-1",
                "test-project",
                old_data,
                new_data,
                client_id,
                Some("authentication".to_string()),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_notify_entity_deleted() {
        let broadcaster = Arc::new(ChangeBroadcaster::new());
        let detector = ChangeDetectionService::new(broadcaster.clone());
        let client_id = Uuid::new_v4();

        let old_data = json!({
            "id": "rule-1",
            "name": "Test Rule",
            "description": "A test business rule"
        });

        let result = detector
            .notify_entity_deleted(
                "business_rule",
                "rule-1",
                "test-project",
                old_data,
                client_id,
                Some("authentication".to_string()),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_notify_bulk_operation() {
        let broadcaster = Arc::new(ChangeBroadcaster::new());
        let detector = ChangeDetectionService::new(broadcaster.clone());
        let client_id = Uuid::new_v4();

        let operation_summary = json!({
            "operation": "bulk_create",
            "entity_count": 5,
            "entities": ["rule-1", "rule-2", "rule-3", "rule-4", "rule-5"]
        });

        let result = detector
            .notify_bulk_operation(
                "business_rule",
                "test-project",
                operation_summary,
                client_id,
                Some("authentication".to_string()),
            )
            .await;

        assert!(result.is_ok());
    }
}