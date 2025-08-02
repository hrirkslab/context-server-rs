use crate::models::enhanced_context::{EnhancedContextItem, ContextId, ProjectId};
use crate::services::websocket_types::{ContextChange, ConflictStrategy, ConflictResolution, ChangeMetadata, ClientId};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn, error};
use uuid::Uuid;

/// Conflict resolution engine for handling concurrent context modifications
#[derive(Clone)]
pub struct ConflictResolutionEngine {
    /// Active conflicts being tracked
    active_conflicts: HashMap<String, ConflictInfo>,
    /// Configuration for conflict resolution strategies
    config: ConflictResolutionConfig,
}

/// Configuration for conflict resolution behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionConfig {
    /// Default strategy to use when no specific strategy is configured
    pub default_strategy: ConflictStrategy,
    /// Maximum time to wait for manual resolution before auto-resolving
    pub manual_resolution_timeout_seconds: u64,
    /// Whether to automatically detect conflicts based on version numbers
    pub auto_detect_version_conflicts: bool,
    /// Whether to automatically detect conflicts based on content changes
    pub auto_detect_content_conflicts: bool,
    /// Minimum time between changes to consider them concurrent (in seconds)
    pub concurrent_change_threshold_seconds: u64,
}

impl Default for ConflictResolutionConfig {
    fn default() -> Self {
        Self {
            default_strategy: ConflictStrategy::LastWriterWins,
            manual_resolution_timeout_seconds: 300, // 5 minutes
            auto_detect_version_conflicts: true,
            auto_detect_content_conflicts: true,
            concurrent_change_threshold_seconds: 30, // 30 seconds
        }
    }
}

/// Information about a detected conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub conflict_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub project_id: String,
    pub conflicting_changes: Vec<ConflictingChange>,
    pub conflict_type: ConflictType,
    pub detected_at: DateTime<Utc>,
    pub resolution_strategy: Option<ConflictStrategy>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub resolution_result: Option<ConflictResolutionResult>,
}

/// A change that conflicts with another change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictingChange {
    pub change_id: Uuid,
    pub change: ContextChange,
    pub base_version: u32,
    pub client_info: ClientInfo,
}

/// Information about the client that made a change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub client_id: ClientId,
    pub user_id: Option<String>,
    pub client_type: String,
    pub timestamp: DateTime<Utc>,
}

/// Types of conflicts that can be detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    /// Version-based conflict (concurrent modifications)
    VersionConflict,
    /// Content-based conflict (overlapping changes)
    ContentConflict,
    /// Semantic conflict (business rule violations)
    SemanticConflict,
    /// Dependency conflict (relationship violations)
    DependencyConflict,
}

/// Result of conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionResult {
    pub strategy_used: ConflictStrategy,
    pub resolved_entity: Option<serde_json::Value>,
    pub discarded_changes: Vec<Uuid>,
    pub merge_details: Option<MergeDetails>,
    pub resolution_notes: Option<String>,
}

/// Details about how changes were merged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeDetails {
    pub merge_algorithm: String,
    pub conflicts_resolved: u32,
    pub manual_interventions: u32,
    pub confidence_score: f64,
}

/// Request for manual conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualResolutionRequest {
    pub conflict_id: String,
    pub resolution_strategy: ConflictStrategy,
    pub resolved_entity: Option<serde_json::Value>,
    pub resolution_notes: Option<String>,
    pub resolved_by: String,
}

impl ConflictResolutionEngine {
    /// Create a new conflict resolution engine
    pub fn new() -> Self {
        Self {
            active_conflicts: HashMap::new(),
            config: ConflictResolutionConfig::default(),
        }
    }

    /// Create a new conflict resolution engine with custom configuration
    pub fn with_config(config: ConflictResolutionConfig) -> Self {
        Self {
            active_conflicts: HashMap::new(),
            config,
        }
    }

    /// Detect conflicts between concurrent changes
    pub async fn detect_conflict(
        &mut self,
        incoming_change: &ContextChange,
        existing_entity: Option<&EnhancedContextItem>,
        recent_changes: &[ContextChange],
    ) -> Result<Option<ConflictInfo>> {
        debug!(
            "Detecting conflicts for change {} on entity {}/{}",
            incoming_change.change_id, incoming_change.entity_type, incoming_change.entity_id
        );

        let mut conflicts = Vec::new();
        let mut detected_conflict_type = ConflictType::ContentConflict; // Default

        // Check for version conflicts first (highest priority)
        if self.config.auto_detect_version_conflicts {
            if let Some(version_conflict) = self.detect_version_conflict(incoming_change, existing_entity)? {
                conflicts.push(version_conflict);
                detected_conflict_type = ConflictType::VersionConflict;
            }
        }

        // Check for content conflicts (only if no version conflict detected)
        if conflicts.is_empty() && self.config.auto_detect_content_conflicts {
            if let Some(content_conflict) = self.detect_content_conflict(incoming_change, recent_changes)? {
                conflicts.push(content_conflict);
                detected_conflict_type = ConflictType::ContentConflict;
            }
        }

        // Check for semantic conflicts
        if let Some(semantic_conflict) = self.detect_semantic_conflict(incoming_change, existing_entity)? {
            conflicts.push(semantic_conflict);
            detected_conflict_type = ConflictType::SemanticConflict;
        }

        if conflicts.is_empty() {
            return Ok(None);
        }

        // Create conflict info
        let conflict_id = Uuid::new_v4().to_string();
        let conflict_info = ConflictInfo {
            conflict_id: conflict_id.clone(),
            entity_type: incoming_change.entity_type.clone(),
            entity_id: incoming_change.entity_id.clone(),
            project_id: incoming_change.project_id.clone(),
            conflicting_changes: conflicts,
            conflict_type: detected_conflict_type,
            detected_at: Utc::now(),
            resolution_strategy: None,
            resolved_at: None,
            resolved_by: None,
            resolution_result: None,
        };

        // Store the conflict
        self.active_conflicts.insert(conflict_id.clone(), conflict_info.clone());

        debug!("Conflict detected: {}", conflict_id);
        Ok(Some(conflict_info))
    }

    /// Resolve a conflict using the specified strategy
    pub async fn resolve_conflict(
        &mut self,
        conflict_id: &str,
        strategy: ConflictStrategy,
        resolver: Option<String>,
    ) -> Result<ConflictResolutionResult> {
        let mut conflict = self.active_conflicts
            .get(conflict_id)
            .ok_or_else(|| anyhow!("Conflict not found: {}", conflict_id))?
            .clone();

        debug!("Resolving conflict {} using strategy {:?}", conflict_id, strategy);

        let resolution_result = match strategy {
            ConflictStrategy::LastWriterWins => {
                self.resolve_last_writer_wins(&conflict).await?
            }
            ConflictStrategy::AutoMerge => {
                self.resolve_auto_merge(&conflict).await?
            }
            ConflictStrategy::ManualResolution => {
                return Err(anyhow!("Manual resolution requires explicit resolution data"));
            }
            ConflictStrategy::Reject => {
                self.resolve_reject(&conflict).await?
            }
        };

        // Update conflict info
        conflict.resolution_strategy = Some(strategy);
        conflict.resolved_at = Some(Utc::now());
        conflict.resolved_by = resolver;
        conflict.resolution_result = Some(resolution_result.clone());

        // Store updated conflict
        self.active_conflicts.insert(conflict_id.to_string(), conflict);

        debug!("Conflict {} resolved successfully", conflict_id);
        Ok(resolution_result)
    }

    /// Resolve a conflict manually with provided resolution data
    pub async fn resolve_conflict_manually(
        &mut self,
        request: ManualResolutionRequest,
    ) -> Result<ConflictResolutionResult> {
        let mut conflict = self.active_conflicts
            .get(&request.conflict_id)
            .ok_or_else(|| anyhow!("Conflict not found: {}", request.conflict_id))?
            .clone();

        debug!("Manually resolving conflict {}", request.conflict_id);

        let resolution_result = ConflictResolutionResult {
            strategy_used: request.resolution_strategy.clone(),
            resolved_entity: request.resolved_entity,
            discarded_changes: conflict.conflicting_changes
                .iter()
                .map(|c| c.change_id)
                .collect(),
            merge_details: None,
            resolution_notes: request.resolution_notes,
        };

        // Update conflict info
        conflict.resolution_strategy = Some(request.resolution_strategy);
        conflict.resolved_at = Some(Utc::now());
        conflict.resolved_by = Some(request.resolved_by);
        conflict.resolution_result = Some(resolution_result.clone());

        // Store updated conflict
        self.active_conflicts.insert(request.conflict_id.clone(), conflict);

        debug!("Conflict {} manually resolved", request.conflict_id);
        Ok(resolution_result)
    }

    /// Get information about an active conflict
    pub fn get_conflict_info(&self, conflict_id: &str) -> Option<&ConflictInfo> {
        self.active_conflicts.get(conflict_id)
    }

    /// Get all active conflicts for a project
    pub fn get_active_conflicts(&self, project_id: &str) -> Vec<&ConflictInfo> {
        self.active_conflicts
            .values()
            .filter(|c| c.project_id == project_id && c.resolved_at.is_none())
            .collect()
    }

    /// Get all resolved conflicts for a project
    pub fn get_resolved_conflicts(&self, project_id: &str) -> Vec<&ConflictInfo> {
        self.active_conflicts
            .values()
            .filter(|c| c.project_id == project_id && c.resolved_at.is_some())
            .collect()
    }

    /// Clean up old resolved conflicts
    pub fn cleanup_resolved_conflicts(&mut self, older_than: DateTime<Utc>) {
        self.active_conflicts.retain(|_, conflict| {
            if let Some(resolved_at) = conflict.resolved_at {
                resolved_at > older_than
            } else {
                true // Keep unresolved conflicts
            }
        });
    }

    /// Detect version-based conflicts
    fn detect_version_conflict(
        &self,
        incoming_change: &ContextChange,
        existing_entity: Option<&EnhancedContextItem>,
    ) -> Result<Option<ConflictingChange>> {
        if let Some(entity) = existing_entity {
            let incoming_version = incoming_change.metadata.version;
            let current_version = entity.version;

            if incoming_version < current_version {
                debug!(
                    "Version conflict detected: incoming version {} < current version {}",
                    incoming_version, current_version
                );

                return Ok(Some(ConflictingChange {
                    change_id: incoming_change.change_id,
                    change: incoming_change.clone(),
                    base_version: incoming_version,
                    client_info: ClientInfo {
                        client_id: incoming_change.metadata.client_id,
                        user_id: incoming_change.metadata.user_id.clone(),
                        client_type: "unknown".to_string(),
                        timestamp: incoming_change.metadata.timestamp,
                    },
                }));
            }
        }

        Ok(None)
    }

    /// Detect content-based conflicts
    fn detect_content_conflict(
        &self,
        incoming_change: &ContextChange,
        recent_changes: &[ContextChange],
    ) -> Result<Option<ConflictingChange>> {
        let threshold = chrono::Duration::seconds(self.config.concurrent_change_threshold_seconds as i64);

        for recent_change in recent_changes {
            if recent_change.entity_id == incoming_change.entity_id
                && recent_change.change_id != incoming_change.change_id
            {
                let time_diff = incoming_change.metadata.timestamp - recent_change.metadata.timestamp;
                
                if time_diff.abs() < threshold {
                    debug!(
                        "Content conflict detected: concurrent changes within {} seconds",
                        self.config.concurrent_change_threshold_seconds
                    );

                    return Ok(Some(ConflictingChange {
                        change_id: recent_change.change_id,
                        change: recent_change.clone(),
                        base_version: recent_change.metadata.version,
                        client_info: ClientInfo {
                            client_id: recent_change.metadata.client_id,
                            user_id: recent_change.metadata.user_id.clone(),
                            client_type: "unknown".to_string(),
                            timestamp: recent_change.metadata.timestamp,
                        },
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Detect semantic conflicts (business rule violations)
    fn detect_semantic_conflict(
        &self,
        _incoming_change: &ContextChange,
        _existing_entity: Option<&EnhancedContextItem>,
    ) -> Result<Option<ConflictingChange>> {
        // TODO: Implement semantic conflict detection
        // This would involve checking business rules, constraints, etc.
        Ok(None)
    }

    /// Determine the primary conflict type from a list of conflicting changes
    fn determine_conflict_type(&self, conflicts: &[ConflictingChange]) -> ConflictType {
        if conflicts.is_empty() {
            return ConflictType::ContentConflict; // Default fallback
        }

        // Check if any conflict was detected by version conflict detection
        // Version conflicts are typically single conflicts where the incoming change has an older version
        if conflicts.len() == 1 {
            let conflict = &conflicts[0];
            // If the change was detected as a version conflict (base_version < current version)
            // we can infer this from the context of how it was detected
            // For now, we'll assume single conflicts from version detection are version conflicts
            // This is a heuristic that works with our current detection logic
            ConflictType::VersionConflict
        } else {
            // Multiple conflicts are typically content conflicts (concurrent changes)
            ConflictType::ContentConflict
        }
    }

    /// Resolve conflict using last-writer-wins strategy
    async fn resolve_last_writer_wins(&self, conflict: &ConflictInfo) -> Result<ConflictResolutionResult> {
        debug!("Resolving conflict using last-writer-wins strategy");

        // Find the most recent change
        let latest_change = conflict.conflicting_changes
            .iter()
            .max_by_key(|c| c.change.metadata.timestamp)
            .ok_or_else(|| anyhow!("No conflicting changes found"))?;

        let discarded_changes: Vec<Uuid> = conflict.conflicting_changes
            .iter()
            .filter(|c| c.change_id != latest_change.change_id)
            .map(|c| c.change_id)
            .collect();

        Ok(ConflictResolutionResult {
            strategy_used: ConflictStrategy::LastWriterWins,
            resolved_entity: latest_change.change.full_entity.clone(),
            discarded_changes,
            merge_details: None,
            resolution_notes: Some("Resolved using last-writer-wins strategy".to_string()),
        })
    }

    /// Resolve conflict using automatic merge strategy
    async fn resolve_auto_merge(&self, conflict: &ConflictInfo) -> Result<ConflictResolutionResult> {
        debug!("Resolving conflict using auto-merge strategy");

        // For now, implement a simple merge that combines non-conflicting fields
        let merged_entity = self.merge_changes(&conflict.conflicting_changes)?;
        
        let merge_details = MergeDetails {
            merge_algorithm: "simple_field_merge".to_string(),
            conflicts_resolved: conflict.conflicting_changes.len() as u32,
            manual_interventions: 0,
            confidence_score: 0.8, // Moderate confidence for auto-merge
        };

        Ok(ConflictResolutionResult {
            strategy_used: ConflictStrategy::AutoMerge,
            resolved_entity: Some(merged_entity),
            discarded_changes: Vec::new(), // No changes discarded in merge
            merge_details: Some(merge_details),
            resolution_notes: Some("Resolved using automatic merge strategy".to_string()),
        })
    }

    /// Resolve conflict by rejecting all changes
    async fn resolve_reject(&self, conflict: &ConflictInfo) -> Result<ConflictResolutionResult> {
        debug!("Resolving conflict by rejecting all changes");

        let discarded_changes: Vec<Uuid> = conflict.conflicting_changes
            .iter()
            .map(|c| c.change_id)
            .collect();

        Ok(ConflictResolutionResult {
            strategy_used: ConflictStrategy::Reject,
            resolved_entity: None,
            discarded_changes,
            merge_details: None,
            resolution_notes: Some("All conflicting changes rejected".to_string()),
        })
    }

    /// Merge multiple changes into a single entity
    fn merge_changes(&self, changes: &[ConflictingChange]) -> Result<serde_json::Value> {
        if changes.is_empty() {
            return Err(anyhow!("No changes to merge"));
        }

        // Start with the first change as base
        let mut merged = changes[0].change.full_entity
            .as_ref()
            .ok_or_else(|| anyhow!("No entity data in first change"))?
            .clone();

        // Merge subsequent changes
        for change in &changes[1..] {
            if let Some(entity_data) = &change.change.full_entity {
                merged = self.merge_json_objects(&merged, entity_data)?;
            }
        }

        Ok(merged)
    }

    /// Merge two JSON objects, with the second taking precedence for conflicts
    fn merge_json_objects(
        &self,
        base: &serde_json::Value,
        overlay: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        match (base, overlay) {
            (serde_json::Value::Object(base_obj), serde_json::Value::Object(overlay_obj)) => {
                let mut merged = base_obj.clone();
                
                for (key, value) in overlay_obj {
                    if let Some(base_value) = merged.get(key) {
                        // Recursively merge nested objects
                        merged.insert(key.clone(), self.merge_json_objects(base_value, value)?);
                    } else {
                        merged.insert(key.clone(), value.clone());
                    }
                }
                
                Ok(serde_json::Value::Object(merged))
            }
            _ => {
                // For non-objects, overlay takes precedence
                Ok(overlay.clone())
            }
        }
    }
}

impl Default for ConflictResolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::websocket_types::ChangeType;
    use serde_json::json;

    fn create_test_change(
        entity_id: &str,
        version: u32,
        client_id: ClientId,
        timestamp: DateTime<Utc>,
    ) -> ContextChange {
        ContextChange {
            change_id: Uuid::new_v4(),
            change_type: ChangeType::Update,
            entity_type: "business_rule".to_string(),
            entity_id: entity_id.to_string(),
            project_id: "test-project".to_string(),
            feature_area: Some("authentication".to_string()),
            delta: None,
            full_entity: Some(json!({
                "id": entity_id,
                "name": "Test Rule",
                "description": "A test business rule"
            })),
            metadata: ChangeMetadata {
                user_id: Some("test-user".to_string()),
                client_id,
                timestamp,
                version,
                conflict_resolution: None,
            },
        }
    }

    #[tokio::test]
    async fn test_conflict_resolution_engine_creation() {
        let engine = ConflictResolutionEngine::new();
        assert_eq!(engine.active_conflicts.len(), 0);
        assert_eq!(engine.config.default_strategy, ConflictStrategy::LastWriterWins);
    }

    #[tokio::test]
    async fn test_version_conflict_detection() {
        let mut engine = ConflictResolutionEngine::new();
        let client_id = Uuid::new_v4();
        let now = Utc::now();

        // Create a change with version 1
        let change = create_test_change("rule-1", 1, client_id, now);

        // Create existing entity with version 2
        let existing_entity = EnhancedContextItem {
            id: "rule-1".to_string(),
            project_id: "test-project".to_string(),
            version: 2,
            ..Default::default()
        };

        let conflict = engine
            .detect_conflict(&change, Some(&existing_entity), &[])
            .await
            .unwrap();

        assert!(conflict.is_some());
        let conflict_info = conflict.unwrap();
        assert_eq!(conflict_info.conflict_type, ConflictType::VersionConflict);
        assert_eq!(conflict_info.conflicting_changes.len(), 1);
    }

    #[tokio::test]
    async fn test_content_conflict_detection() {
        let mut engine = ConflictResolutionEngine::new();
        let client1 = Uuid::new_v4();
        let client2 = Uuid::new_v4();
        let now = Utc::now();

        // Create two changes within the concurrent threshold
        let change1 = create_test_change("rule-1", 1, client1, now);
        let change2 = create_test_change("rule-1", 1, client2, now + chrono::Duration::seconds(10));

        let recent_changes = vec![change1];

        let conflict = engine
            .detect_conflict(&change2, None, &recent_changes)
            .await
            .unwrap();

        assert!(conflict.is_some());
        let conflict_info = conflict.unwrap();
        assert_eq!(conflict_info.conflict_type, ConflictType::ContentConflict);
    }

    #[tokio::test]
    async fn test_last_writer_wins_resolution() {
        let mut engine = ConflictResolutionEngine::new();
        let client1 = Uuid::new_v4();
        let client2 = Uuid::new_v4();
        let now = Utc::now();

        // Create conflicting changes
        let change1 = create_test_change("rule-1", 1, client1, now);
        let change2 = create_test_change("rule-1", 1, client2, now + chrono::Duration::seconds(10));

        // Simulate conflict detection
        let conflict_info = ConflictInfo {
            conflict_id: Uuid::new_v4().to_string(),
            entity_type: "business_rule".to_string(),
            entity_id: "rule-1".to_string(),
            project_id: "test-project".to_string(),
            conflicting_changes: vec![
                ConflictingChange {
                    change_id: change1.change_id,
                    change: change1,
                    base_version: 1,
                    client_info: ClientInfo {
                        client_id: client1,
                        user_id: Some("user1".to_string()),
                        client_type: "test".to_string(),
                        timestamp: now,
                    },
                },
                ConflictingChange {
                    change_id: change2.change_id,
                    change: change2,
                    base_version: 1,
                    client_info: ClientInfo {
                        client_id: client2,
                        user_id: Some("user2".to_string()),
                        client_type: "test".to_string(),
                        timestamp: now + chrono::Duration::seconds(10),
                    },
                },
            ],
            conflict_type: ConflictType::ContentConflict,
            detected_at: now,
            resolution_strategy: None,
            resolved_at: None,
            resolved_by: None,
            resolution_result: None,
        };

        engine.active_conflicts.insert(conflict_info.conflict_id.clone(), conflict_info.clone());

        let result = engine
            .resolve_conflict(&conflict_info.conflict_id, ConflictStrategy::LastWriterWins, Some("test-resolver".to_string()))
            .await
            .unwrap();

        assert_eq!(result.strategy_used, ConflictStrategy::LastWriterWins);
        assert!(result.resolved_entity.is_some());
        assert_eq!(result.discarded_changes.len(), 1);
    }

    #[tokio::test]
    async fn test_auto_merge_resolution() {
        let mut engine = ConflictResolutionEngine::new();
        let client1 = Uuid::new_v4();
        let client2 = Uuid::new_v4();
        let now = Utc::now();

        // Create changes with different data
        let mut change1 = create_test_change("rule-1", 1, client1, now);
        change1.full_entity = Some(json!({
            "id": "rule-1",
            "name": "Rule from Client 1",
            "description": "Original description"
        }));

        let mut change2 = create_test_change("rule-1", 1, client2, now + chrono::Duration::seconds(10));
        change2.full_entity = Some(json!({
            "id": "rule-1",
            "name": "Rule from Client 2",
            "priority": "high"
        }));

        let conflict_info = ConflictInfo {
            conflict_id: Uuid::new_v4().to_string(),
            entity_type: "business_rule".to_string(),
            entity_id: "rule-1".to_string(),
            project_id: "test-project".to_string(),
            conflicting_changes: vec![
                ConflictingChange {
                    change_id: change1.change_id,
                    change: change1,
                    base_version: 1,
                    client_info: ClientInfo {
                        client_id: client1,
                        user_id: Some("user1".to_string()),
                        client_type: "test".to_string(),
                        timestamp: now,
                    },
                },
                ConflictingChange {
                    change_id: change2.change_id,
                    change: change2,
                    base_version: 1,
                    client_info: ClientInfo {
                        client_id: client2,
                        user_id: Some("user2".to_string()),
                        client_type: "test".to_string(),
                        timestamp: now + chrono::Duration::seconds(10),
                    },
                },
            ],
            conflict_type: ConflictType::ContentConflict,
            detected_at: now,
            resolution_strategy: None,
            resolved_at: None,
            resolved_by: None,
            resolution_result: None,
        };

        engine.active_conflicts.insert(conflict_info.conflict_id.clone(), conflict_info.clone());

        let result = engine
            .resolve_conflict(&conflict_info.conflict_id, ConflictStrategy::AutoMerge, Some("test-resolver".to_string()))
            .await
            .unwrap();

        assert_eq!(result.strategy_used, ConflictStrategy::AutoMerge);
        assert!(result.resolved_entity.is_some());
        assert!(result.merge_details.is_some());
        assert_eq!(result.discarded_changes.len(), 0); // No changes discarded in merge
    }

    #[tokio::test]
    async fn test_manual_resolution() {
        let mut engine = ConflictResolutionEngine::new();
        let conflict_id = Uuid::new_v4().to_string();

        // Create a conflict
        let conflict_info = ConflictInfo {
            conflict_id: conflict_id.clone(),
            entity_type: "business_rule".to_string(),
            entity_id: "rule-1".to_string(),
            project_id: "test-project".to_string(),
            conflicting_changes: vec![],
            conflict_type: ConflictType::ContentConflict,
            detected_at: Utc::now(),
            resolution_strategy: None,
            resolved_at: None,
            resolved_by: None,
            resolution_result: None,
        };

        engine.active_conflicts.insert(conflict_id.clone(), conflict_info);

        let manual_request = ManualResolutionRequest {
            conflict_id: conflict_id.clone(),
            resolution_strategy: ConflictStrategy::ManualResolution,
            resolved_entity: Some(json!({
                "id": "rule-1",
                "name": "Manually Resolved Rule",
                "description": "This was resolved manually"
            })),
            resolution_notes: Some("Resolved by human reviewer".to_string()),
            resolved_by: "human-reviewer".to_string(),
        };

        let result = engine
            .resolve_conflict_manually(manual_request)
            .await
            .unwrap();

        assert_eq!(result.strategy_used, ConflictStrategy::ManualResolution);
        assert!(result.resolved_entity.is_some());
        assert!(result.resolution_notes.is_some());

        // Check that conflict is marked as resolved
        let updated_conflict = engine.get_conflict_info(&conflict_id).unwrap();
        assert!(updated_conflict.resolved_at.is_some());
        assert_eq!(updated_conflict.resolved_by, Some("human-reviewer".to_string()));
    }

    #[tokio::test]
    async fn test_get_active_conflicts() {
        let mut engine = ConflictResolutionEngine::new();
        let project_id = "test-project";

        // Add an active conflict
        let active_conflict = ConflictInfo {
            conflict_id: "active-1".to_string(),
            entity_type: "business_rule".to_string(),
            entity_id: "rule-1".to_string(),
            project_id: project_id.to_string(),
            conflicting_changes: vec![],
            conflict_type: ConflictType::ContentConflict,
            detected_at: Utc::now(),
            resolution_strategy: None,
            resolved_at: None,
            resolved_by: None,
            resolution_result: None,
        };

        // Add a resolved conflict
        let resolved_conflict = ConflictInfo {
            conflict_id: "resolved-1".to_string(),
            entity_type: "business_rule".to_string(),
            entity_id: "rule-2".to_string(),
            project_id: project_id.to_string(),
            conflicting_changes: vec![],
            conflict_type: ConflictType::VersionConflict,
            detected_at: Utc::now(),
            resolution_strategy: Some(ConflictStrategy::LastWriterWins),
            resolved_at: Some(Utc::now()),
            resolved_by: Some("test-resolver".to_string()),
            resolution_result: None,
        };

        engine.active_conflicts.insert("active-1".to_string(), active_conflict);
        engine.active_conflicts.insert("resolved-1".to_string(), resolved_conflict);

        let active_conflicts = engine.get_active_conflicts(project_id);
        assert_eq!(active_conflicts.len(), 1);
        assert_eq!(active_conflicts[0].conflict_id, "active-1");

        let resolved_conflicts = engine.get_resolved_conflicts(project_id);
        assert_eq!(resolved_conflicts.len(), 1);
        assert_eq!(resolved_conflicts[0].conflict_id, "resolved-1");
    }
}