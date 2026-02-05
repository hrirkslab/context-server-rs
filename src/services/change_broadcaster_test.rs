use super::change_broadcaster::*;
use super::websocket_types::*;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_change_broadcaster_creation() {
    let broadcaster = ChangeBroadcaster::new();
    
    // Test that broadcaster is created successfully
    assert!(broadcaster.subscriptions.is_empty());
    assert!(broadcaster.change_queue.is_empty());
    assert!(broadcaster.change_history.is_empty());
}

#[tokio::test]
async fn test_client_subscription() {
    let broadcaster = ChangeBroadcaster::new();
    let client_id = Uuid::new_v4();
    
    let filters = vec![SyncFilters {
        project_ids: Some(vec!["test-project".to_string()]),
        entity_types: Some(vec!["business_rule".to_string()]),
        feature_areas: None,
        change_types: None,
    }];
    
    // Test subscription
    broadcaster.subscribe(client_id, filters.clone()).await.unwrap();
    
    assert!(broadcaster.subscriptions.contains_key(&client_id));
    assert!(broadcaster.change_queue.contains_key(&client_id));
    
    let stored_filters = broadcaster.subscriptions.get(&client_id).unwrap();
    assert_eq!(stored_filters.len(), 1);
    
    // Test unsubscription
    broadcaster.unsubscribe(client_id).await.unwrap();
    
    assert!(!broadcaster.subscriptions.contains_key(&client_id));
    assert!(!broadcaster.change_queue.contains_key(&client_id));
}

#[tokio::test]
async fn test_subscription_update() {
    let broadcaster = ChangeBroadcaster::new();
    let client_id = Uuid::new_v4();
    
    let initial_filters = vec![SyncFilters {
        project_ids: Some(vec!["project1".to_string()]),
        entity_types: None,
        feature_areas: None,
        change_types: None,
    }];
    
    let updated_filters = vec![
        SyncFilters {
            project_ids: Some(vec!["project1".to_string(), "project2".to_string()]),
            entity_types: Some(vec!["business_rule".to_string()]),
            feature_areas: None,
            change_types: None,
        },
        SyncFilters {
            project_ids: None,
            entity_types: None,
            feature_areas: Some(vec!["authentication".to_string()]),
            change_types: Some(vec![ChangeType::Create, ChangeType::Update]),
        },
    ];
    
    // Subscribe with initial filters
    broadcaster.subscribe(client_id, initial_filters).await.unwrap();
    
    // Update subscription
    broadcaster.update_subscription(client_id, updated_filters.clone()).await.unwrap();
    
    let stored_filters = broadcaster.subscriptions.get(&client_id).unwrap();
    assert_eq!(stored_filters.len(), 2);
    
    // Test updating non-existent client
    let non_existent_client = Uuid::new_v4();
    let result = broadcaster.update_subscription(non_existent_client, updated_filters).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_change_broadcasting() {
    let broadcaster = ChangeBroadcaster::new();
    let client_id = Uuid::new_v4();
    
    // Subscribe client
    let filters = vec![SyncFilters {
        project_ids: Some(vec!["test-project".to_string()]),
        entity_types: Some(vec!["business_rule".to_string()]),
        feature_areas: None,
        change_types: Some(vec![ChangeType::Create]),
    }];
    
    broadcaster.subscribe(client_id, filters).await.unwrap();
    
    // Create a change event
    let change_event = ChangeEvent {
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        change_type: ChangeType::Create,
        old_value: None,
        new_value: Some(json!({
            "id": "rule-1",
            "name": "Test Rule",
            "description": "A test business rule"
        })),
        client_id,
        feature_area: Some("authentication".to_string()),
    };
    
    // Broadcast the change
    broadcaster.broadcast_change(change_event).await.unwrap();
    
    // Check metrics
    let metrics = broadcaster.get_metrics();
    assert_eq!(metrics.total_changes_broadcast.load(std::sync::atomic::Ordering::Relaxed), 1);
    assert_eq!(metrics.total_clients_notified.load(std::sync::atomic::Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_delta_calculation() {
    let broadcaster = ChangeBroadcaster::new();
    let client_id = Uuid::new_v4();
    
    // Create an update change event with old and new values
    let change_event = ChangeEvent {
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        change_type: ChangeType::Update,
        old_value: Some(json!({
            "id": "rule-1",
            "name": "Old Rule Name",
            "description": "Old description",
            "priority": "high"
        })),
        new_value: Some(json!({
            "id": "rule-1",
            "name": "New Rule Name",
            "description": "New description",
            "status": "active"
        })),
        client_id,
        feature_area: Some("authentication".to_string()),
    };
    
    // Calculate delta
    let delta = broadcaster.calculate_delta(&change_event).await.unwrap();
    
    assert!(delta.is_some());
    let delta_value = delta.unwrap();
    
    // Check that delta contains old and new values
    assert!(delta_value.get("old").is_some());
    assert!(delta_value.get("new").is_some());
    assert!(delta_value.get("changed_fields").is_some());
    
    // Check changed fields
    let changed_fields = delta_value.get("changed_fields").unwrap().as_array().unwrap();
    assert!(changed_fields.contains(&json!("name")));
    assert!(changed_fields.contains(&json!("description")));
    assert!(changed_fields.contains(&json!("status")));
    assert!(changed_fields.contains(&json!("removed_priority")));
}

#[tokio::test]
async fn test_change_filtering() {
    let broadcaster = ChangeBroadcaster::new();
    let client1 = Uuid::new_v4();
    let client2 = Uuid::new_v4();
    let client3 = Uuid::new_v4();
    
    // Subscribe clients with different filters
    broadcaster.subscribe(client1, vec![SyncFilters {
        project_ids: Some(vec!["project1".to_string()]),
        entity_types: None,
        feature_areas: None,
        change_types: None,
    }]).await.unwrap();
    
    broadcaster.subscribe(client2, vec![SyncFilters {
        project_ids: None,
        entity_types: Some(vec!["business_rule".to_string()]),
        feature_areas: None,
        change_types: None,
    }]).await.unwrap();
    
    broadcaster.subscribe(client3, vec![SyncFilters {
        project_ids: Some(vec!["project2".to_string()]),
        entity_types: Some(vec!["architectural_decision".to_string()]),
        feature_areas: None,
        change_types: None,
    }]).await.unwrap();
    
    // Create a change that should match client1 and client2
    let change = ContextChange {
        change_id: Uuid::new_v4(),
        change_type: ChangeType::Create,
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "project1".to_string(),
        feature_area: Some("authentication".to_string()),
        delta: None,
        full_entity: Some(json!({"name": "test rule"})),
        metadata: ChangeMetadata {
            user_id: None,
            client_id: client1,
            timestamp: Utc::now(),
            version: 1,
            conflict_resolution: None,
        },
    };
    
    let matching_clients = broadcaster.find_matching_clients(&change).await;
    
    // Should match client1 (project1) and client2 (business_rule), but not client3
    assert_eq!(matching_clients.len(), 2);
    assert!(matching_clients.contains(&client1));
    assert!(matching_clients.contains(&client2));
    assert!(!matching_clients.contains(&client3));
}

#[tokio::test]
async fn test_change_queue_management() {
    let broadcaster = ChangeBroadcaster::new();
    let client_id = Uuid::new_v4();
    
    broadcaster.subscribe(client_id, vec![SyncFilters::default()]).await.unwrap();
    
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
    
    // Queue the change
    broadcaster.queue_change(&change, &[client_id]).await.unwrap();
    
    // Check that change is queued
    let queued_changes = broadcaster.get_queued_changes(client_id).await;
    assert_eq!(queued_changes.len(), 1);
    assert_eq!(queued_changes[0].change_id, change.change_id);
    
    // Acknowledge the change
    broadcaster.acknowledge_change(client_id, change.change_id).await.unwrap();
    
    // Check that change is removed from queue
    let queued_changes = broadcaster.get_queued_changes(client_id).await;
    assert_eq!(queued_changes.len(), 0);
}

#[tokio::test]
async fn test_broadcast_metrics() {
    let broadcaster = ChangeBroadcaster::new();
    let client_id = Uuid::new_v4();
    
    broadcaster.subscribe(client_id, vec![SyncFilters::default()]).await.unwrap();
    
    // Initial metrics should be zero
    let initial_metrics = broadcaster.get_metrics();
    assert_eq!(initial_metrics.total_changes_broadcast.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert_eq!(initial_metrics.total_clients_notified.load(std::sync::atomic::Ordering::Relaxed), 0);
    
    // Broadcast a change
    let change_event = ChangeEvent {
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        change_type: ChangeType::Create,
        old_value: None,
        new_value: Some(json!({"name": "test"})),
        client_id,
        feature_area: None,
    };
    
    broadcaster.broadcast_change(change_event).await.unwrap();
    
    // Check updated metrics
    let updated_metrics = broadcaster.get_metrics();
    assert_eq!(updated_metrics.total_changes_broadcast.load(std::sync::atomic::Ordering::Relaxed), 1);
    assert_eq!(updated_metrics.total_clients_notified.load(std::sync::atomic::Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_change_receiver() {
    let broadcaster = ChangeBroadcaster::new();
    let mut receiver = broadcaster.subscribe_to_changes();
    
    let client_id = Uuid::new_v4();
    broadcaster.subscribe(client_id, vec![SyncFilters::default()]).await.unwrap();
    
    // Broadcast a change
    let change_event = ChangeEvent {
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        change_type: ChangeType::Create,
        old_value: None,
        new_value: Some(json!({"name": "test"})),
        client_id,
        feature_area: None,
    };
    
    // Start broadcasting in a separate task
    let broadcaster_clone = broadcaster.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        broadcaster_clone.broadcast_change(change_event).await.unwrap();
    });
    
    // Receive the change
    let received_change = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        receiver.recv()
    ).await.unwrap().unwrap();
    
    assert_eq!(received_change.entity_type, "business_rule");
    assert_eq!(received_change.entity_id, "rule-1");
    assert_eq!(received_change.change_type, ChangeType::Create);
}