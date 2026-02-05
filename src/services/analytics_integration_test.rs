#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::AppContainer;
    use crate::services::{AnalyticsHelper, AnalyticsEventType};
    use tempfile::tempdir;
    use tokio_test;

    #[tokio::test]
    async fn test_analytics_service_integration() {
        // Create a temporary database
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_analytics.db");
        let db_path_str = db_path.to_str().unwrap();

        // Initialize the container with analytics service
        let container = AppContainer::new(db_path_str).unwrap();

        // Test creating an analytics event
        let event = AnalyticsHelper::create_context_query_event(
            Some("test_project".to_string()),
            Some("authentication".to_string()),
            Some("implement".to_string()),
            Some(vec!["user_service".to_string(), "auth_controller".to_string()]),
            Some(150),
            true,
            None,
        );

        // Track the event
        let result = container.analytics_service.track_event(event).await;
        assert!(result.is_ok(), "Failed to track analytics event: {:?}", result);

        // Test getting project insights
        let insights_result = container.analytics_service.get_project_insights("test_project").await;
        assert!(insights_result.is_ok(), "Failed to get project insights: {:?}", insights_result);

        let insights = insights_result.unwrap();
        assert_eq!(insights.project_id, "test_project");
        assert_eq!(insights.total_events, 1);
        assert_eq!(insights.success_rate, 1.0);

        // Test getting global statistics
        let global_stats_result = container.analytics_service.get_global_statistics().await;
        assert!(global_stats_result.is_ok(), "Failed to get global statistics: {:?}", global_stats_result);

        let global_stats = global_stats_result.unwrap();
        assert!(global_stats.contains_key("total_events"));
        assert!(global_stats.contains_key("success_rate"));
    }

    #[tokio::test]
    async fn test_analytics_helper_event_creation() {
        // Test context query event creation
        let query_event = AnalyticsHelper::create_context_query_event(
            Some("project1".to_string()),
            Some("payments".to_string()),
            Some("fix".to_string()),
            Some(vec!["payment_service".to_string()]),
            Some(200),
            true,
            None,
        );

        assert!(matches!(query_event.event_type, AnalyticsEventType::ContextQuery));
        assert_eq!(query_event.project_id, Some("project1".to_string()));
        assert_eq!(query_event.duration_ms, Some(200));
        assert!(query_event.success);

        // Test entity create event creation
        let create_event = AnalyticsHelper::create_entity_create_event(
            Some("project1".to_string()),
            "business_rule".to_string(),
            "rule123".to_string(),
            Some(100),
            true,
            None,
        );

        assert!(matches!(create_event.event_type, AnalyticsEventType::EntityCreate));
        assert_eq!(create_event.entity_type, Some("business_rule".to_string()));
        assert_eq!(create_event.entity_id, Some("rule123".to_string()));

        // Test bulk operation event creation
        let bulk_event = AnalyticsHelper::create_bulk_operation_event(
            Some("project1".to_string()),
            "framework_component".to_string(),
            "bulk_create".to_string(),
            5,
            Some(500),
            true,
            None,
        );

        assert!(matches!(bulk_event.event_type, AnalyticsEventType::BulkOperation));
        assert_eq!(bulk_event.metadata.get("count").unwrap().as_u64(), Some(5));
        assert_eq!(bulk_event.metadata.get("operation").unwrap().as_str(), Some("bulk_create"));
    }
}