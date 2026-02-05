use crate::services::conflict_resolution_engine::{ConflictResolutionEngine, ConflictType, ManualResolutionRequest};
use crate::services::conflict_resolution_ui::{ConflictResolutionUI, StartResolutionRequest, ConflictResolutionStep, UpdateUIStateRequest};
use crate::services::sync_engine::SyncEngine;
use crate::services::websocket_types::{ContextChange, ChangeType, ChangeMetadata, ConflictStrategy, ClientId};
use crate::models::enhanced_context::EnhancedContextItem;
use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Integration test for the complete conflict resolution workflow
#[tokio::test]
async fn test_complete_conflict_resolution_workflow() -> Result<()> {
    // Setup
    let sync_engine = SyncEngine::new();
    let mut conflict_ui = ConflictResolutionUI::new();
    
    let client1 = Uuid::new_v4();
    let client2 = Uuid::new_v4();
    let now = Utc::now();

    // Create an existing entity
    let existing_entity = EnhancedContextItem {
        id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        version: 2,
        ..Default::default()
    };

    // Create a conflicting change (version conflict)
    let incoming_change = ContextChange {
        change_id: Uuid::new_v4(),
        change_type: ChangeType::Update,
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        feature_area: Some("authentication".to_string()),
        delta: None,
        full_entity: Some(json!({
            "id": "rule-1",
            "name": "Updated Rule",
            "description": "This is an updated business rule"
        })),
        metadata: ChangeMetadata {
            user_id: Some("user1".to_string()),
            client_id: client1,
            timestamp: now,
            version: 1, // Lower version than existing entity
            conflict_resolution: None,
        },
    };

    // Step 1: Detect conflict using sync engine
    let conflict_info = sync_engine
        .detect_and_handle_conflict(&incoming_change, Some(&existing_entity), &[])
        .await?;

    assert!(conflict_info.is_some());
    let conflict_info = conflict_info.unwrap();
    assert_eq!(conflict_info.conflict_type, ConflictType::VersionConflict);
    assert_eq!(conflict_info.conflicting_changes.len(), 1);

    // Step 2: Start UI resolution session
    let start_request = StartResolutionRequest {
        conflict_id: conflict_info.conflict_id.clone(),
        user_id: "test-user".to_string(),
        client_id: client2,
        preferred_strategy: None,
        timeout_seconds: Some(600),
    };

    let start_response = conflict_ui
        .start_resolution_session(start_request, conflict_info.clone())
        .await?;

    assert!(!start_response.session_id.is_empty());
    assert_eq!(start_response.recommended_strategy, ConflictStrategy::LastWriterWins);
    assert!(!start_response.available_strategies.is_empty());

    // Step 3: Update UI state to select strategy
    let mut user_selections = HashMap::new();
    user_selections.insert("strategy".to_string(), json!("LastWriterWins"));

    let update_request = UpdateUIStateRequest {
        session_id: start_response.session_id.clone(),
        step: ConflictResolutionStep::StrategySelection,
        user_selections,
        selected_strategy: Some(ConflictStrategy::LastWriterWins),
    };

    let update_response = conflict_ui.update_ui_state(update_request).await?;
    assert!(update_response.success);
    assert!(update_response.can_proceed);

    // Step 4: Complete resolution
    let manual_request = conflict_ui
        .complete_resolution(
            &start_response.session_id,
            Some("Resolved using last writer wins strategy".to_string()),
        )
        .await?;

    assert_eq!(manual_request.resolution_strategy, ConflictStrategy::LastWriterWins);
    assert_eq!(manual_request.resolved_by, "test-user");

    // Step 5: Apply resolution using sync engine
    let resolution_result = sync_engine
        .resolve_conflict_manually(manual_request)
        .await?;

    assert_eq!(resolution_result.strategy_used, ConflictStrategy::LastWriterWins);
    assert!(resolution_result.resolved_entity.is_some());

    // Step 6: Verify conflict is resolved
    let resolved_conflict = sync_engine
        .get_conflict_info(&conflict_info.conflict_id)
        .await;

    assert!(resolved_conflict.is_some());
    let resolved_conflict = resolved_conflict.unwrap();
    assert!(resolved_conflict.resolved_at.is_some());
    assert_eq!(resolved_conflict.resolved_by, Some("test-user".to_string()));

    Ok(())
}

/// Test content conflict resolution with auto-merge
#[tokio::test]
async fn test_content_conflict_auto_merge() -> Result<()> {
    let sync_engine = SyncEngine::new();
    let mut conflict_ui = ConflictResolutionUI::new();
    
    let client1 = Uuid::new_v4();
    let client2 = Uuid::new_v4();
    let now = Utc::now();

    // Create two concurrent changes
    let change1 = ContextChange {
        change_id: Uuid::new_v4(),
        change_type: ChangeType::Update,
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        feature_area: Some("auth".to_string()),
        delta: None,
        full_entity: Some(json!({
            "id": "rule-1",
            "name": "Rule from Client 1",
            "description": "Original description"
        })),
        metadata: ChangeMetadata {
            user_id: Some("user1".to_string()),
            client_id: client1,
            timestamp: now,
            version: 1,
            conflict_resolution: None,
        },
    };

    let change2 = ContextChange {
        change_id: Uuid::new_v4(),
        change_type: ChangeType::Update,
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        feature_area: Some("auth".to_string()),
        delta: None,
        full_entity: Some(json!({
            "id": "rule-1",
            "name": "Rule from Client 2",
            "priority": "high"
        })),
        metadata: ChangeMetadata {
            user_id: Some("user2".to_string()),
            client_id: client2,
            timestamp: now + chrono::Duration::seconds(10),
            version: 1,
            conflict_resolution: None,
        },
    };

    // Detect content conflict
    let conflict_info = sync_engine
        .detect_and_handle_conflict(&change2, None, &[change1])
        .await?;

    assert!(conflict_info.is_some());
    let conflict_info = conflict_info.unwrap();
    assert_eq!(conflict_info.conflict_type, ConflictType::ContentConflict);

    // Resolve using auto-merge strategy
    let resolution_result = sync_engine
        .resolve_conflict(&conflict_info.conflict_id, ConflictStrategy::AutoMerge, Some("auto-resolver".to_string()))
        .await?;

    assert_eq!(resolution_result.strategy_used, ConflictStrategy::AutoMerge);
    assert!(resolution_result.resolved_entity.is_some());
    assert!(resolution_result.merge_details.is_some());

    let merge_details = resolution_result.merge_details.unwrap();
    assert_eq!(merge_details.merge_algorithm, "simple_field_merge");
    assert!(merge_details.confidence_score > 0.0);

    Ok(())
}

/// Test manual conflict resolution workflow
#[tokio::test]
async fn test_manual_conflict_resolution() -> Result<()> {
    let sync_engine = SyncEngine::new();
    let mut conflict_ui = ConflictResolutionUI::new();
    
    let client1 = Uuid::new_v4();
    let client2 = Uuid::new_v4();
    let now = Utc::now();

    // Create a complex conflict that requires manual resolution
    let change1 = ContextChange {
        change_id: Uuid::new_v4(),
        change_type: ChangeType::Update,
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        feature_area: Some("auth".to_string()),
        delta: None,
        full_entity: Some(json!({
            "id": "rule-1",
            "name": "Complex Rule 1",
            "conditions": ["condition_a", "condition_b"]
        })),
        metadata: ChangeMetadata {
            user_id: Some("user1".to_string()),
            client_id: client1,
            timestamp: now,
            version: 1,
            conflict_resolution: None,
        },
    };

    let change2 = ContextChange {
        change_id: Uuid::new_v4(),
        change_type: ChangeType::Update,
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        feature_area: Some("auth".to_string()),
        delta: None,
        full_entity: Some(json!({
            "id": "rule-1",
            "name": "Complex Rule 2",
            "conditions": ["condition_c", "condition_d"]
        })),
        metadata: ChangeMetadata {
            user_id: Some("user2".to_string()),
            client_id: client2,
            timestamp: now + chrono::Duration::seconds(5),
            version: 1,
            conflict_resolution: None,
        },
    };

    // Detect conflict
    let conflict_info = sync_engine
        .detect_and_handle_conflict(&change2, None, &[change1])
        .await?;

    assert!(conflict_info.is_some());
    let conflict_info = conflict_info.unwrap();

    // Start manual resolution session
    let start_request = StartResolutionRequest {
        conflict_id: conflict_info.conflict_id.clone(),
        user_id: "expert-user".to_string(),
        client_id: Uuid::new_v4(),
        preferred_strategy: Some(ConflictStrategy::ManualResolution),
        timeout_seconds: Some(1800), // 30 minutes for complex resolution
    };

    let start_response = conflict_ui
        .start_resolution_session(start_request, conflict_info.clone())
        .await?;

    // Select manual resolution strategy
    let mut user_selections = HashMap::new();
    user_selections.insert("strategy".to_string(), json!("ManualResolution"));

    let update_request = UpdateUIStateRequest {
        session_id: start_response.session_id.clone(),
        step: ConflictResolutionStep::StrategySelection,
        user_selections,
        selected_strategy: Some(ConflictStrategy::ManualResolution),
    };

    let _update_response = conflict_ui.update_ui_state(update_request).await?;

    // Perform manual resolution
    let mut manual_selections = HashMap::new();
    manual_selections.insert("resolved_entity".to_string(), json!({
        "id": "rule-1",
        "name": "Manually Resolved Complex Rule",
        "conditions": ["condition_a", "condition_b", "condition_c", "condition_d"],
        "resolution_notes": "Combined all conditions from both changes"
    }));

    let manual_update_request = UpdateUIStateRequest {
        session_id: start_response.session_id.clone(),
        step: ConflictResolutionStep::ManualResolution,
        user_selections: manual_selections,
        selected_strategy: Some(ConflictStrategy::ManualResolution),
    };

    let manual_update_response = conflict_ui.update_ui_state(manual_update_request).await?;
    assert!(manual_update_response.success);

    // Move to preview step to generate the preview entity
    let preview_request = UpdateUIStateRequest {
        session_id: start_response.session_id.clone(),
        step: ConflictResolutionStep::PreviewConfirmation,
        user_selections: HashMap::new(),
        selected_strategy: Some(ConflictStrategy::ManualResolution),
    };

    let _preview_response = conflict_ui.update_ui_state(preview_request).await?;

    // Complete resolution
    let manual_request = conflict_ui
        .complete_resolution(
            &start_response.session_id,
            Some("Manually resolved by combining all conditions".to_string()),
        )
        .await?;

    // Apply resolution
    let resolution_result = sync_engine
        .resolve_conflict_manually(manual_request)
        .await?;

    assert_eq!(resolution_result.strategy_used, ConflictStrategy::ManualResolution);
    assert!(resolution_result.resolved_entity.is_some());

    let resolved_entity = resolution_result.resolved_entity.unwrap();
    assert_eq!(resolved_entity["name"], "Manually Resolved Complex Rule");
    assert_eq!(resolved_entity["conditions"].as_array().unwrap().len(), 4);

    Ok(())
}

/// Test conflict cleanup and session management
#[tokio::test]
async fn test_conflict_cleanup_and_session_management() -> Result<()> {
    let sync_engine = SyncEngine::new();
    let mut conflict_ui = ConflictResolutionUI::new();
    
    let client_id = Uuid::new_v4();
    let now = Utc::now();

    // Create a simple conflict
    let change = ContextChange {
        change_id: Uuid::new_v4(),
        change_type: ChangeType::Update,
        entity_type: "business_rule".to_string(),
        entity_id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        feature_area: Some("auth".to_string()),
        delta: None,
        full_entity: Some(json!({"id": "rule-1", "name": "Test Rule"})),
        metadata: ChangeMetadata {
            user_id: Some("user1".to_string()),
            client_id,
            timestamp: now,
            version: 1,
            conflict_resolution: None,
        },
    };

    let existing_entity = EnhancedContextItem {
        id: "rule-1".to_string(),
        project_id: "test-project".to_string(),
        version: 2,
        ..Default::default()
    };

    // Detect conflict
    let conflict_info = sync_engine
        .detect_and_handle_conflict(&change, Some(&existing_entity), &[])
        .await?;

    assert!(conflict_info.is_some());
    let conflict_info = conflict_info.unwrap();

    // Start resolution session
    let start_request = StartResolutionRequest {
        conflict_id: conflict_info.conflict_id.clone(),
        user_id: "test-user".to_string(),
        client_id,
        preferred_strategy: None,
        timeout_seconds: Some(60), // Short timeout for testing
    };

    let start_response = conflict_ui
        .start_resolution_session(start_request, conflict_info.clone())
        .await?;

    // Verify session exists
    let session = conflict_ui.get_session(&start_response.session_id);
    assert!(session.is_some());

    // Test session cancellation
    conflict_ui.cancel_resolution(&start_response.session_id).await?;

    let cancelled_session = conflict_ui.get_session(&start_response.session_id);
    assert!(cancelled_session.is_some());
    assert_eq!(cancelled_session.unwrap().ui_state.current_step, ConflictResolutionStep::Cancelled);

    // Test cleanup of expired sessions
    conflict_ui.cleanup_expired_sessions();

    // Verify active and resolved conflicts
    let active_conflicts = sync_engine.get_active_conflicts("test-project").await;
    assert_eq!(active_conflicts.len(), 1);

    // Resolve the conflict
    let _resolution_result = sync_engine
        .resolve_conflict(&conflict_info.conflict_id, ConflictStrategy::LastWriterWins, Some("test-resolver".to_string()))
        .await?;

    let resolved_conflicts = sync_engine.get_resolved_conflicts("test-project").await;
    assert_eq!(resolved_conflicts.len(), 1);

    let active_conflicts_after = sync_engine.get_active_conflicts("test-project").await;
    assert_eq!(active_conflicts_after.len(), 0);

    Ok(())
}