use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Analytics event types for tracking context usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsEventType {
    ContextQuery,
    EntityCreate,
    EntityUpdate,
    EntityDelete,
    BulkOperation,
    ArchitectureValidation,
    CacheOperation,
}

/// Analytics event data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub id: String,
    pub event_type: AnalyticsEventType,
    pub project_id: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: Option<u64>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Usage statistics for a specific context or entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub last_query: Option<DateTime<Utc>>,
    pub average_response_time_ms: f64,
    pub most_common_operations: Vec<String>,
}

/// Project-level analytics insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInsights {
    pub project_id: String,
    pub total_events: u64,
    pub most_active_entity_types: Vec<(String, u64)>,
    pub success_rate: f64,
    pub average_response_time_ms: f64,
    pub peak_usage_hours: Vec<u8>,
    pub context_health_score: f64,
    pub recommendations: Vec<String>,
}

/// Analytics service trait for tracking usage and generating insights
#[async_trait]
pub trait AnalyticsService: Send + Sync {
    /// Track a usage event
    async fn track_event(&self, event: AnalyticsEvent) -> Result<()>;
    
    /// Get usage statistics for a specific entity
    async fn get_entity_usage(&self, entity_type: &str, entity_id: &str) -> Result<UsageStatistics>;
    
    /// Get project-level insights
    async fn get_project_insights(&self, project_id: &str) -> Result<ProjectInsights>;
    
    /// Get global usage statistics
    async fn get_global_statistics(&self) -> Result<HashMap<String, serde_json::Value>>;
    
    /// Generate a usage report for a time period
    async fn generate_usage_report(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<serde_json::Value>;
}

/// Default implementation of the analytics service
pub struct DefaultAnalyticsService {
    repository: Box<dyn AnalyticsRepository>,
}

impl DefaultAnalyticsService {
    pub fn new(repository: Box<dyn AnalyticsRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl AnalyticsService for DefaultAnalyticsService {
    async fn track_event(&self, event: AnalyticsEvent) -> Result<()> {
        self.repository.store_event(event).await
    }
    
    async fn get_entity_usage(&self, entity_type: &str, entity_id: &str) -> Result<UsageStatistics> {
        self.repository.get_entity_usage(entity_type, entity_id).await
    }
    
    async fn get_project_insights(&self, project_id: &str) -> Result<ProjectInsights> {
        let events = self.repository.get_project_events(project_id).await?;
        
        let total_events = events.len() as u64;
        let successful_events = events.iter().filter(|e| e.success).count() as u64;
        let success_rate = if total_events > 0 {
            successful_events as f64 / total_events as f64
        } else {
            0.0
        };
        
        // Calculate average response time
        let total_duration: u64 = events.iter()
            .filter_map(|e| e.duration_ms)
            .sum();
        let events_with_duration = events.iter()
            .filter(|e| e.duration_ms.is_some())
            .count();
        let average_response_time_ms = if events_with_duration > 0 {
            total_duration as f64 / events_with_duration as f64
        } else {
            0.0
        };
        
        // Count entity types
        let mut entity_type_counts: HashMap<String, u64> = HashMap::new();
        for event in &events {
            if let Some(entity_type) = &event.entity_type {
                *entity_type_counts.entry(entity_type.clone()).or_insert(0) += 1;
            }
        }
        
        let mut most_active_entity_types: Vec<(String, u64)> = entity_type_counts.into_iter().collect();
        most_active_entity_types.sort_by(|a, b| b.1.cmp(&a.1));
        most_active_entity_types.truncate(5);
        
        // Calculate context health score (simplified)
        let context_health_score = success_rate * 100.0;
        
        // Generate recommendations
        let mut recommendations = Vec::new();
        if success_rate < 0.9 {
            recommendations.push("Consider reviewing error patterns to improve success rate".to_string());
        }
        if average_response_time_ms > 1000.0 {
            recommendations.push("Response times are high, consider optimizing queries".to_string());
        }
        if total_events < 10 {
            recommendations.push("Low usage detected, consider promoting context server features".to_string());
        }
        
        Ok(ProjectInsights {
            project_id: project_id.to_string(),
            total_events,
            most_active_entity_types,
            success_rate,
            average_response_time_ms,
            peak_usage_hours: vec![], // TODO: Implement peak hours calculation
            context_health_score,
            recommendations,
        })
    }
    
    async fn get_global_statistics(&self) -> Result<HashMap<String, serde_json::Value>> {
        self.repository.get_global_statistics().await
    }
    
    async fn generate_usage_report(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<serde_json::Value> {
        self.repository.generate_usage_report(start_date, end_date).await
    }
}

/// Repository trait for analytics data persistence
#[async_trait]
pub trait AnalyticsRepository: Send + Sync {
    async fn store_event(&self, event: AnalyticsEvent) -> Result<()>;
    async fn get_entity_usage(&self, entity_type: &str, entity_id: &str) -> Result<UsageStatistics>;
    async fn get_project_events(&self, project_id: &str) -> Result<Vec<AnalyticsEvent>>;
    async fn get_global_statistics(&self) -> Result<HashMap<String, serde_json::Value>>;
    async fn generate_usage_report(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<serde_json::Value>;
}