use super::websocket_manager::WebSocketManager;
use super::websocket_types::*;
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_websocket_manager_creation() {
    let manager = WebSocketManager::new();
    
    // Test that manager is created successfully
    assert!(manager.connections.is_empty());
    assert!(manager.message_queue.is_empty());
    assert!(manager.health_monitor.is_empty());
}

#[tokio::test]
async fn test_sync_filters_matching() {
    let project_id = "test-project".to_string();
    let entity_type = "business_rule".to_string();
    let feature_area = "authentication".to_string();
    
    // Create a test context change
    let change = ContextChange {
        change_id: Uuid::new_v4(),
        change_type: ChangeType::Create,
        entity_type: entity_type.clone(),
        entity_id: "rule-1".to_string(),
        project_id: project_id.clone(),
        feature_area: Some(feature_area.clone()),
        delta: None,
        full_entity: Some(serde_json::json!({"name": "test rule"})),
        metadata: ChangeMetadata {
            user_id: None,
            client_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: 1,
            conflict_resolution: None,
        },
    };

    // Test filter matching
    let filter_match_all = SyncFilters::default();
    assert!(filter_match_all.matches(&change));

    let filter_project = SyncFilters {
        project_ids: Some(vec![project_id.clone()]),
        entity_types: None,
        feature_areas: None,
        change_types: None,
    };
    assert!(filter_project.matches(&change));

    let filter_wrong_project = SyncFilters {
        project_ids: Some(vec!["wrong-project".to_string()]),
        entity_types: None,
        feature_areas: None,
        change_types: None,
    };
    assert!(!filter_wrong_project.matches(&change));

    let filter_entity_type = SyncFilters {
        project_ids: None,
        entity_types: Some(vec![entity_type.clone()]),
        feature_areas: None,
        change_types: None,
    };
    assert!(filter_entity_type.matches(&change));

    let filter_feature_area = SyncFilters {
        project_ids: None,
        entity_types: None,
        feature_areas: Some(vec![feature_area.clone()]),
        change_types: None,
    };
    assert!(filter_feature_area.matches(&change));

    let filter_change_type = SyncFilters {
        project_ids: None,
        entity_types: None,
        feature_areas: None,
        change_types: Some(vec![ChangeType::Create]),
    };
    assert!(filter_change_type.matches(&change));

    let filter_wrong_change_type = SyncFilters {
        project_ids: None,
        entity_types: None,
        feature_areas: None,
        change_types: Some(vec![ChangeType::Delete]),
    };
    assert!(!filter_wrong_change_type.matches(&change));
}

#[tokio::test]
async fn test_websocket_message_serialization() {
    // Test authentication message
    let auth_msg = WebSocketMessage::Auth {
        token: Some("test-token".to_string()),
        project_id: "test-project".to_string(),
        client_info: ClientInfo {
            user_agent: Some("test-agent".to_string()),
            client_type: ClientType::AIAgent,
            version: "1.0.0".to_string(),
        },
    };

    let serialized = serde_json::to_string(&auth_msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::Auth { token, project_id, client_info } => {
            assert_eq!(token, Some("test-token".to_string()));
            assert_eq!(project_id, "test-project");
            assert_eq!(client_info.version, "1.0.0");
            match client_info.client_type {
                ClientType::AIAgent => {},
                _ => panic!("Wrong client type"),
            }
        }
        _ => panic!("Wrong message type"),
    }

    // Test context change message
    let change = ContextChange {
        change_id: Uuid::new_v4(),
        change_type: ChangeType::Update,
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        feature_area: Some("authentication".to_string()),
        delta: Some(serde_json::json!({"name": "updated rule"})),
        full_entity: Some(serde_json::json!({"id": "rule-1", "name": "updated rule"})),
        metadata: ChangeMetadata {
            user_id: Some("user-1".to_string()),
            client_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: 2,
            conflict_resolution: None,
        },
    };

    let change_msg = WebSocketMessage::ContextChange {
        message_id: Uuid::new_v4(),
        change: change.clone(),
        timestamp: Utc::now(),
    };

    let serialized = serde_json::to_string(&change_msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::ContextChange { message_id: _, change: deserialized_change, timestamp: _ } => {
            assert_eq!(deserialized_change.entity_type, "business_rule");
            assert_eq!(deserialized_change.change_type, ChangeType::Update);
            assert_eq!(deserialized_change.metadata.version, 2);
        }
        _ => panic!("Wrong message type"),
    }
}

#[tokio::test]
async fn test_sync_status_creation() {
    let manager = WebSocketManager::new();
    let project_id = "test-project";
    
    let status = manager.get_sync_status(project_id).await;
    
    assert_eq!(status.project_id, project_id);
    assert_eq!(status.connected_clients, 0);
    assert_eq!(status.pending_changes, 0);
    assert!(matches!(status.sync_health, SyncHealth::Unhealthy)); // No clients connected
}

#[tokio::test]
async fn test_conflict_resolution_types() {
    let resolution = ConflictResolution {
        strategy: ConflictStrategy::LastWriterWins,
        resolved_by: "system".to_string(),
        original_changes: vec![
            serde_json::json!({"field": "old_value"}),
            serde_json::json!({"field": "new_value"}),
        ],
    };

    // Test serialization
    let serialized = serde_json::to_string(&resolution).unwrap();
    let deserialized: ConflictResolution = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.resolved_by, "system");
    assert!(matches!(deserialized.strategy, ConflictStrategy::LastWriterWins));
    assert_eq!(deserialized.original_changes.len(), 2);
}

#[tokio::test]
async fn test_client_info_types() {
    let client_types = vec![
        ClientType::AIAgent,
        ClientType::IDE,
        ClientType::WebInterface,
        ClientType::CLI,
        ClientType::Other("custom".to_string()),
    ];

    for client_type in client_types {
        let client_info = ClientInfo {
            user_agent: Some("test-agent".to_string()),
            client_type: client_type.clone(),
            version: "1.0.0".to_string(),
        };

        // Test serialization
        let serialized = serde_json::to_string(&client_info).unwrap();
        let deserialized: ClientInfo = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.version, "1.0.0");
        match (&client_type, &deserialized.client_type) {
            (ClientType::AIAgent, ClientType::AIAgent) => {},
            (ClientType::IDE, ClientType::IDE) => {},
            (ClientType::WebInterface, ClientType::WebInterface) => {},
            (ClientType::CLI, ClientType::CLI) => {},
            (ClientType::Other(a), ClientType::Other(b)) => assert_eq!(a, b),
            _ => panic!("Client type mismatch"),
        }
    }
}