use crate::services::analytics_service::{AnalyticsEvent, AnalyticsEventType};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Helper functions for creating analytics events
pub struct AnalyticsHelper;

impl AnalyticsHelper {
    /// Create a context query analytics event
    pub fn create_context_query_event(
        project_id: Option<String>,
        feature_area: Option<String>,
        task_type: Option<String>,
        components: Option<Vec<String>>,
        duration_ms: Option<u64>,
        success: bool,
        error_message: Option<String>,
    ) -> AnalyticsEvent {
        let mut metadata = HashMap::new();
        
        if let Some(feature) = feature_area {
            metadata.insert("feature_area".to_string(), serde_json::Value::String(feature));
        }
        if let Some(task) = task_type {
            metadata.insert("task_type".to_string(), serde_json::Value::String(task));
        }
        if let Some(comps) = components {
            metadata.insert("components".to_string(), serde_json::Value::Array(
                comps.into_iter().map(serde_json::Value::String).collect()
            ));
        }

        AnalyticsEvent {
            id: Uuid::new_v4().to_string(),
            event_type: AnalyticsEventType::ContextQuery,
            project_id,
            entity_type: Some("context_query".to_string()),
            entity_id: None,
            user_agent: None,
            metadata,
            timestamp: Utc::now(),
            duration_ms,
            success,
            error_message,
        }
    }

    /// Create an entity creation analytics event
    pub fn create_entity_create_event(
        project_id: Option<String>,
        entity_type: String,
        entity_id: String,
        duration_ms: Option<u64>,
        success: bool,
        error_message: Option<String>,
    ) -> AnalyticsEvent {
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), serde_json::Value::String("create".to_string()));

        AnalyticsEvent {
            id: Uuid::new_v4().to_string(),
            event_type: AnalyticsEventType::EntityCreate,
            project_id,
            entity_type: Some(entity_type),
            entity_id: Some(entity_id),
            user_agent: None,
            metadata,
            timestamp: Utc::now(),
            duration_ms,
            success,
            error_message,
        }
    }

    /// Create an entity update analytics event
    pub fn create_entity_update_event(
        project_id: Option<String>,
        entity_type: String,
        entity_id: String,
        duration_ms: Option<u64>,
        success: bool,
        error_message: Option<String>,
    ) -> AnalyticsEvent {
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), serde_json::Value::String("update".to_string()));

        AnalyticsEvent {
            id: Uuid::new_v4().to_string(),
            event_type: AnalyticsEventType::EntityUpdate,
            project_id,
            entity_type: Some(entity_type),
            entity_id: Some(entity_id),
            user_agent: None,
            metadata,
            timestamp: Utc::now(),
            duration_ms,
            success,
            error_message,
        }
    }

    /// Create an entity delete analytics event
    pub fn create_entity_delete_event(
        project_id: Option<String>,
        entity_type: String,
        entity_id: String,
        duration_ms: Option<u64>,
        success: bool,
        error_message: Option<String>,
    ) -> AnalyticsEvent {
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), serde_json::Value::String("delete".to_string()));

        AnalyticsEvent {
            id: Uuid::new_v4().to_string(),
            event_type: AnalyticsEventType::EntityDelete,
            project_id,
            entity_type: Some(entity_type),
            entity_id: Some(entity_id),
            user_agent: None,
            metadata,
            timestamp: Utc::now(),
            duration_ms,
            success,
            error_message,
        }
    }

    /// Create a bulk operation analytics event
    pub fn create_bulk_operation_event(
        project_id: Option<String>,
        entity_type: String,
        operation: String,
        count: usize,
        duration_ms: Option<u64>,
        success: bool,
        error_message: Option<String>,
    ) -> AnalyticsEvent {
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), serde_json::Value::String(operation));
        metadata.insert("count".to_string(), serde_json::Value::Number(count.into()));

        AnalyticsEvent {
            id: Uuid::new_v4().to_string(),
            event_type: AnalyticsEventType::BulkOperation,
            project_id,
            entity_type: Some(entity_type),
            entity_id: None,
            user_agent: None,
            metadata,
            timestamp: Utc::now(),
            duration_ms,
            success,
            error_message,
        }
    }

    /// Create an architecture validation analytics event
    pub fn create_architecture_validation_event(
        project_id: String,
        violations_count: usize,
        duration_ms: Option<u64>,
        success: bool,
        error_message: Option<String>,
    ) -> AnalyticsEvent {
        let mut metadata = HashMap::new();
        metadata.insert("violations_count".to_string(), serde_json::Value::Number(violations_count.into()));

        AnalyticsEvent {
            id: Uuid::new_v4().to_string(),
            event_type: AnalyticsEventType::ArchitectureValidation,
            project_id: Some(project_id),
            entity_type: Some("architecture_validation".to_string()),
            entity_id: None,
            user_agent: None,
            metadata,
            timestamp: Utc::now(),
            duration_ms,
            success,
            error_message,
        }
    }

    /// Create a cache operation analytics event
    pub fn create_cache_operation_event(
        project_id: Option<String>,
        operation: String,
        duration_ms: Option<u64>,
        success: bool,
        error_message: Option<String>,
    ) -> AnalyticsEvent {
        let mut metadata = HashMap::new();
        metadata.insert("cache_operation".to_string(), serde_json::Value::String(operation));

        AnalyticsEvent {
            id: Uuid::new_v4().to_string(),
            event_type: AnalyticsEventType::CacheOperation,
            project_id,
            entity_type: Some("cache_operation".to_string()),
            entity_id: None,
            user_agent: None,
            metadata,
            timestamp: Utc::now(),
            duration_ms,
            success,
            error_message,
        }
    }

    /// Create a general analytics event for analytics operations
    pub fn create_analytics_event(
        operation: String,
        context: Option<String>,
        duration_ms: Option<u64>,
        success: bool,
        error_message: Option<String>,
    ) -> AnalyticsEvent {
        let mut metadata = HashMap::new();
        metadata.insert("analytics_operation".to_string(), serde_json::Value::String(operation.clone()));
        
        if let Some(ctx) = context {
            metadata.insert("context".to_string(), serde_json::Value::String(ctx));
        }

        AnalyticsEvent {
            id: Uuid::new_v4().to_string(),
            event_type: AnalyticsEventType::ContextQuery, // Using ContextQuery as the closest match
            project_id: None,
            entity_type: Some("analytics_operation".to_string()),
            entity_id: Some(operation),
            user_agent: None,
            metadata,
            timestamp: Utc::now(),
            duration_ms,
            success,
            error_message,
        }
    }
}