use super::change_broadcaster::ChangeBroadcaster;
use super::change_detection_service::ChangeDetectionService;
use super::sync_engine::SyncEngine;
use super::websocket_types::*;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

#[tokio::test]
async fn test_end_to_end_change_broadcasting() {
    // Create sync engine with all components
    let sync_engine = SyncEngine::new();
    let client_id = Uuid::new_v4();
    
    // Subscribe client to changes
    let filters = vec![SyncFilters {
        project_ids: Some(vec!["test-project".to_string()]),
        entity_types: Some(vec!["business_rule".to_string()]),
        feature_areas: None,
        change_types: Some(vec![ChangeType::Create, ChangeType::Update]),
    }];
    
    let mut stream = sync_engine.subscribe(client_id, filters).await.unwrap();
    
    // Get change detector for triggering changes
    let change_detector = sync_engine.get_change_detector();
    
    // Test entity creation
    let entity_data = json!({
        "id": "rule-1",
        "name": "Test Rule",
        "description": "A test business rule",
        "priority": "high"
    });
    
    // Trigger change in background task
    let change_detector_clone = change_detector.clone();
    let client_id_clone = client_id;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        change_detector_clone
            .notify_entity_created(
                "business_rule",
                "rule-1",
                "test-project",
                entity_data,
                client_id_clone,
                Some("authentication".to_string()),
            )
            .await
            .unwrap();
    });
    
    // Receive the change
    let received_change = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("Timeout waiting for change")
        .expect("Failed to receive change");
    
    assert_eq!(received_change.entity_type, "business_rule");
    assert_eq!(received_change.entity_id, "rule-1");
    assert_eq!(received_change.change_type, ChangeType::Create);
    assert_eq!(received_change.project_id, "test-project");
    assert_eq!(received_change.feature_area, Some("authentication".to_string()));
}

#[tokio::test]
async fn test_change_filtering_across_multiple_clients() {
    let sync_engine = SyncEngine::new();
    
    // Create multiple clients with different filters
    let client1 = Uuid::new_v4();
    let client2 = Uuid::new_v4();
    let client3 = Uuid::new_v4();
    
    // Client 1: Only business rules in project1
    let filters1 = vec![SyncFilters {
        project_ids: Some(vec!["project1".to_string()]),
        entity_types: Some(vec!["business_rule".to_string()]),
        feature_areas: None,
        change_types: None,
    }];
    
    // Client 2: Only architectural decisions in any project
    let filters2 = vec![SyncFilters {
        project_ids: None,
        entity_types: Some(vec!["architectural_decision".to_string()]),
        feature_areas: None,
        change_types: None,
    }];
    
    // Client 3: Only authentication feature area in project1
    let filters3 = vec![SyncFilters {
        project_ids: Some(vec!["project1".to_string()]),
        entity_types: None,
        feature_areas: Some(vec!["authentication".to_string()]),
        change_types: None,
    }];
    
    let mut stream1 = sync_engine.subscribe(client1, filters1).await.unwrap();
    let mut stream2 = sync_engine.subscribe(client2, filters2).await.unwrap();
    let mut stream3 = sync_engine.subscribe(client3, filters3).await.unwrap();
    
    let change_detector = sync_engine.get_change_detector();
    
    // Create a business rule in project1 with authentication feature area
    // This should match client1 and client3, but not client2
    let entity_data = json!({
        "id": "rule-1",
        "name": "Auth Rule",
        "description": "Authentication business rule"
    });
    
    let change_detector_clone = change_detector.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        change_detector_clone
            .notify_entity_created(
                "business_rule",
                "rule-1",
                "project1",
                entity_data,
                client1,
                Some("authentication".to_string()),
            )
            .await
            .unwrap();
    });
    
    // Client 1 should receive the change (business_rule + project1)
    let change1 = timeout(Duration::from_secs(1), stream1.next())
        .await
        .expect("Client 1 should receive change")
        .expect("Failed to receive change for client 1");
    
    assert_eq!(change1.entity_type, "business_rule");
    assert_eq!(change1.project_id, "project1");
    
    // Client 3 should receive the change (authentication + project1)
    let change3 = timeout(Duration::from_secs(1), stream3.next())
        .await
        .expect("Client 3 should receive change")
        .expect("Failed to receive change for client 3");
    
    assert_eq!(change3.entity_type, "business_rule");
    assert_eq!(change3.feature_area, Some("authentication".to_string()));
    
    // Client 2 should NOT receive the change (only wants architectural_decision)
    let result2 = timeout(Duration::from_millis(500), stream2.next()).await;
    assert!(result2.is_err(), "Client 2 should not receive the change");
}

#[tokio::test]
async fn test_delta_calculation_in_updates() {
    let sync_engine = SyncEngine::new();
    let client_id = Uuid::new_v4();
    
    let filters = vec![SyncFilters {
        project_ids: Some(vec!["test-project".to_string()]),
        entity_types: Some(vec!["business_rule".to_string()]),
        feature_areas: None,
        change_types: Some(vec![ChangeType::Update]),
    }];
    
    let mut stream = sync_engine.subscribe(client_id, filters).await.unwrap();
    let change_detector = sync_engine.get_change_detector();
    
    // Test entity update with delta calculation
    let old_data = json!({
        "id": "rule-1",
        "name": "Old Rule Name",
        "description": "Old description",
        "priority": "high",
        "status": "draft"
    });
    
    let new_data = json!({
        "id": "rule-1",
        "name": "New Rule Name",
        "description": "New description",
        "priority": "medium",
        "category": "security"
    });
    
    let change_detector_clone = change_detector.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        change_detector_clone
            .notify_entity_updated(
                "business_rule",
                "rule-1",
                "test-project",
                old_data,
                new_data,
                client_id,
                Some("security".to_string()),
            )
            .await
            .unwrap();
    });
    
    let received_change = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("Timeout waiting for update change")
        .expect("Failed to receive update change");
    
    assert_eq!(received_change.change_type, ChangeType::Update);
    assert!(received_change.delta.is_some(), "Delta should be calculated for updates");
    
    let delta = received_change.delta.unwrap();
    assert!(delta.get("old").is_some());
    assert!(delta.get("new").is_some());
    assert!(delta.get("changed_fields").is_some());
    
    let changed_fields = delta.get("changed_fields").unwrap().as_array().unwrap();
    assert!(changed_fields.contains(&json!("name")));
    assert!(changed_fields.contains(&json!("description")));
    assert!(changed_fields.contains(&json!("priority")));
    assert!(changed_fields.contains(&json!("category")));
    assert!(changed_fields.contains(&json!("removed_status")));
}

#[tokio::test]
async fn test_bulk_operation_broadcasting() {
    let sync_engine = SyncEngine::new();
    let client_id = Uuid::new_v4();
    
    let filters = vec![SyncFilters {
        project_ids: Some(vec!["test-project".to_string()]),
        entity_types: Some(vec!["business_rule".to_string()]),
        feature_areas: None,
        change_types: Some(vec![ChangeType::Bulk]),
    }];
    
    let mut stream = sync_engine.subscribe(client_id, filters).await.unwrap();
    let change_detector = sync_engine.get_change_detector();
    
    let operation_summary = json!({
        "operation": "bulk_create",
        "entity_count": 5,
        "entities": ["rule-1", "rule-2", "rule-3", "rule-4", "rule-5"],
        "feature_area": "authentication",
        "timestamp": Utc::now().to_rfc3339()
    });
    
    let change_detector_clone = change_detector.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        change_detector_clone
            .notify_bulk_operation(
                "business_rule",
                "test-project",
                operation_summary,
                client_id,
                Some("authentication".to_string()),
            )
            .await
            .unwrap();
    });
    
    let received_change = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("Timeout waiting for bulk change")
        .expect("Failed to receive bulk change");
    
    assert_eq!(received_change.change_type, ChangeType::Bulk);
    assert_eq!(received_change.entity_type, "business_rule");
    assert_eq!(received_change.project_id, "test-project");
    assert!(received_change.entity_id.starts_with("bulk_"));
    
    let operation_data = received_change.full_entity.unwrap();
    assert_eq!(operation_data.get("operation").unwrap(), "bulk_create");
    assert_eq!(operation_data.get("entity_count").unwrap(), 5);
}

#[tokio::test]
async fn test_change_queue_and_acknowledgment() {
    let broadcaster = Arc::new(ChangeBroadcaster::new());
    let client_id = Uuid::new_v4();
    
    // Subscribe client
    let filters = vec![SyncFilters::default()];
    broadcaster.subscribe(client_id, filters).await.unwrap();
    
    // Create a test change
    let change = ContextChange {
        change_id: Uuid::new_v4(),
        change_type: ChangeType::Create,
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        feature_area: None,
        delta: None,
        full_entity: Some(json!({"name": "test"})),
        metadata: ChangeMetadata {
            user_id: None,
            client_id,
            timestamp: Utc::now(),
            version: 1,
            conflict_resolution: None,
        },
    };
    
    // Queue the change manually (simulating failed immediate delivery)
    broadcaster.queue_change(&change, &[client_id]).await.unwrap();
    
    // Verify change is queued
    let queued_changes = broadcaster.get_queued_changes(client_id).await;
    assert_eq!(queued_changes.len(), 1);
    assert_eq!(queued_changes[0].change_id, change.change_id);
    
    // Acknowledge the change
    broadcaster.acknowledge_change(client_id, change.change_id).await.unwrap();
    
    // Verify change is removed from queue
    let queued_changes_after_ack = broadcaster.get_queued_changes(client_id).await;
    assert_eq!(queued_changes_after_ack.len(), 0);
}

#[tokio::test]
async fn test_metrics_collection() {
    let sync_engine = SyncEngine::new();
    let client_id = Uuid::new_v4();
    
    // Subscribe client
    let filters = vec![SyncFilters::default()];
    let _stream = sync_engine.subscribe(client_id, filters).await.unwrap();
    
    let broadcaster = sync_engine.get_broadcaster();
    let initial_metrics = broadcaster.get_metrics();
    
    // Trigger some changes
    let change_detector = sync_engine.get_change_detector();
    
    for i in 0..3 {
        let entity_data = json!({
            "id": format!("rule-{}", i),
            "name": format!("Test Rule {}", i)
        });
        
        change_detector
            .notify_entity_created(
                "business_rule",
                &format!("rule-{}", i),
                "test-project",
                entity_data,
                client_id,
                None,
            )
            .await
            .unwrap();
    }
    
    // Allow some time for metrics to update
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let updated_metrics = broadcaster.get_metrics();
    
    // Check that metrics have increased
    let initial_broadcasts = initial_metrics.total_changes_broadcast.load(std::sync::atomic::Ordering::Relaxed);
    let updated_broadcasts = updated_metrics.total_changes_broadcast.load(std::sync::atomic::Ordering::Relaxed);
    
    assert!(updated_broadcasts > initial_broadcasts, 
           "Broadcast count should have increased from {} to {}", 
           initial_broadcasts, updated_broadcasts);
}

#[tokio::test]
async fn test_sync_status_reporting() {
    let sync_engine = SyncEngine::new();
    
    // Get sync status for a project
    let status = sync_engine.get_sync_status("test-project").await.unwrap();
    
    assert_eq!(status.project_id, "test-project");
    assert_eq!(status.connected_clients, 0); // No clients connected yet
    assert_eq!(status.pending_changes, 0); // No pending changes
    
    // The sync health should be unhealthy with no connected clients
    matches!(status.sync_health, SyncHealth::Unhealthy);
}

#[tokio::test]
async fn test_multiple_filter_matching() {
    let sync_engine = SyncEngine::new();
    let client_id = Uuid::new_v4();
    
    // Client with multiple filters - should match if ANY filter matches
    let filters = vec![
        SyncFilters {
            project_ids: Some(vec!["project1".to_string()]),
            entity_types: Some(vec!["business_rule".to_string()]),
            feature_areas: None,
            change_types: None,
        },
        SyncFilters {
            project_ids: Some(vec!["project2".to_string()]),
            entity_types: Some(vec!["architectural_decision".to_string()]),
            feature_areas: None,
            change_types: None,
        },
    ];
    
    let mut stream = sync_engine.subscribe(client_id, filters).await.unwrap();
    let change_detector = sync_engine.get_change_detector();
    
    // Create a change that matches the second filter
    let entity_data = json!({
        "id": "decision-1",
        "title": "Test Decision",
        "description": "A test architectural decision"
    });
    
    let change_detector_clone = change_detector.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        change_detector_clone
            .notify_entity_created(
                "architectural_decision",
                "decision-1",
                "project2",
                entity_data,
                client_id,
                Some("architecture".to_string()),
            )
            .await
            .unwrap();
    });
    
    // Should receive the change because it matches the second filter
    let received_change = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("Should receive change matching second filter")
        .expect("Failed to receive change");
    
    assert_eq!(received_change.entity_type, "architectural_decision");
    assert_eq!(received_change.project_id, "project2");
}