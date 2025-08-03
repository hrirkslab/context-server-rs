use crate::services::analytics_service::{AnalyticsEvent, AnalyticsEventType, AnalyticsRepository, UsageStatistics};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row, OptionalExtension};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// SQLite implementation of the analytics repository
pub struct SqliteAnalyticsRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteAnalyticsRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    /// Initialize the analytics tables
    pub fn init_tables(&self) -> Result<()> {
        let conn = self.db.lock().unwrap();
        
        // Create analytics_events table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS analytics_events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                project_id TEXT,
                entity_type TEXT,
                entity_id TEXT,
                user_agent TEXT,
                metadata TEXT,
                timestamp TEXT NOT NULL,
                duration_ms INTEGER,
                success BOOLEAN NOT NULL,
                error_message TEXT
            )",
            [],
        )?;

        // Create indexes for better query performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_analytics_events_project_id ON analytics_events(project_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_analytics_events_entity ON analytics_events(entity_type, entity_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_analytics_events_timestamp ON analytics_events(timestamp)",
            [],
        )?;

        Ok(())
    }

    fn row_to_analytics_event(row: &Row) -> Result<AnalyticsEvent, rusqlite::Error> {
        let event_type_str: String = row.get("event_type")?;
        let event_type = match event_type_str.as_str() {
            "ContextQuery" => AnalyticsEventType::ContextQuery,
            "EntityCreate" => AnalyticsEventType::EntityCreate,
            "EntityUpdate" => AnalyticsEventType::EntityUpdate,
            "EntityDelete" => AnalyticsEventType::EntityDelete,
            "BulkOperation" => AnalyticsEventType::BulkOperation,
            "ArchitectureValidation" => AnalyticsEventType::ArchitectureValidation,
            "CacheOperation" => AnalyticsEventType::CacheOperation,
            _ => AnalyticsEventType::ContextQuery, // Default fallback
        };

        let metadata_str: Option<String> = row.get("metadata")?;
        let metadata = if let Some(meta_str) = metadata_str {
            serde_json::from_str(&meta_str).unwrap_or_default()
        } else {
            HashMap::new()
        };

        let timestamp_str: String = row.get("timestamp")?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "timestamp".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        Ok(AnalyticsEvent {
            id: row.get("id")?,
            event_type,
            project_id: row.get("project_id")?,
            entity_type: row.get("entity_type")?,
            entity_id: row.get("entity_id")?,
            user_agent: row.get("user_agent")?,
            metadata,
            timestamp,
            duration_ms: row.get("duration_ms")?,
            success: row.get("success")?,
            error_message: row.get("error_message")?,
        })
    }
}

#[async_trait]
impl AnalyticsRepository for SqliteAnalyticsRepository {
    async fn store_event(&self, event: AnalyticsEvent) -> Result<()> {
        let conn = self.db.lock().unwrap();
        
        let event_type_str = match event.event_type {
            AnalyticsEventType::ContextQuery => "ContextQuery",
            AnalyticsEventType::EntityCreate => "EntityCreate",
            AnalyticsEventType::EntityUpdate => "EntityUpdate",
            AnalyticsEventType::EntityDelete => "EntityDelete",
            AnalyticsEventType::BulkOperation => "BulkOperation",
            AnalyticsEventType::ArchitectureValidation => "ArchitectureValidation",
            AnalyticsEventType::CacheOperation => "CacheOperation",
        };

        let metadata_json = serde_json::to_string(&event.metadata)?;
        let timestamp_str = event.timestamp.to_rfc3339();

        conn.execute(
            "INSERT INTO analytics_events (
                id, event_type, project_id, entity_type, entity_id, 
                user_agent, metadata, timestamp, duration_ms, success, error_message
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                event.id,
                event_type_str,
                event.project_id,
                event.entity_type,
                event.entity_id,
                event.user_agent,
                metadata_json,
                timestamp_str,
                event.duration_ms,
                event.success,
                event.error_message
            ],
        )?;

        Ok(())
    }

    async fn get_entity_usage(&self, entity_type: &str, entity_id: &str) -> Result<UsageStatistics> {
        let conn = self.db.lock().unwrap();
        
        // Get total queries
        let total_queries: u64 = conn.query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE entity_type = ?1 AND entity_id = ?2",
            params![entity_type, entity_id],
            |row| row.get(0),
        )?;

        // Get successful queries
        let successful_queries: u64 = conn.query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE entity_type = ?1 AND entity_id = ?2 AND success = 1",
            params![entity_type, entity_id],
            |row| row.get(0),
        )?;

        let failed_queries = total_queries - successful_queries;

        // Get last query timestamp
        let last_query: Option<String> = conn.query_row(
            "SELECT timestamp FROM analytics_events WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY timestamp DESC LIMIT 1",
            params![entity_type, entity_id],
            |row| row.get(0),
        ).optional()?;

        let last_query_parsed = if let Some(ref timestamp_str) = last_query {
            DateTime::parse_from_rfc3339(timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        } else {
            None
        };

        // Get average response time
        let avg_response_time: Option<f64> = conn.query_row(
            "SELECT AVG(duration_ms) FROM analytics_events WHERE entity_type = ?1 AND entity_id = ?2 AND duration_ms IS NOT NULL",
            params![entity_type, entity_id],
            |row| row.get(0),
        ).optional()?;

        // Get most common operations (simplified - just get event types)
        let mut stmt = conn.prepare(
            "SELECT event_type, COUNT(*) as count FROM analytics_events 
             WHERE entity_type = ?1 AND entity_id = ?2 
             GROUP BY event_type ORDER BY count DESC LIMIT 5"
        )?;
        
        let operation_rows = stmt.query_map(params![entity_type, entity_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
        })?;

        let mut most_common_operations = Vec::new();
        for row in operation_rows {
            let (operation, _count) = row?;
            most_common_operations.push(operation);
        }

        Ok(UsageStatistics {
            total_queries,
            successful_queries,
            failed_queries,
            last_query: last_query_parsed,
            average_response_time_ms: avg_response_time.unwrap_or(0.0),
            most_common_operations,
        })
    }

    async fn get_project_events(&self, project_id: &str) -> Result<Vec<AnalyticsEvent>> {
        let conn = self.db.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, event_type, project_id, entity_type, entity_id, 
                    user_agent, metadata, timestamp, duration_ms, success, error_message
             FROM analytics_events WHERE project_id = ?1 ORDER BY timestamp DESC"
        )?;
        
        let event_rows = stmt.query_map(params![project_id], Self::row_to_analytics_event)?;
        
        let mut events = Vec::new();
        for event_result in event_rows {
            events.push(event_result?);
        }
        
        Ok(events)
    }

    async fn get_global_statistics(&self) -> Result<HashMap<String, serde_json::Value>> {
        let conn = self.db.lock().unwrap();
        
        let mut stats = HashMap::new();
        
        // Total events
        let total_events: u64 = conn.query_row(
            "SELECT COUNT(*) FROM analytics_events",
            [],
            |row| row.get(0),
        )?;
        stats.insert("total_events".to_string(), serde_json::Value::Number(total_events.into()));
        
        // Success rate
        let successful_events: u64 = conn.query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE success = 1",
            [],
            |row| row.get(0),
        )?;
        let success_rate = if total_events > 0 {
            successful_events as f64 / total_events as f64
        } else {
            0.0
        };
        stats.insert("success_rate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(success_rate).unwrap_or(serde_json::Number::from(0))));
        
        // Average response time
        let avg_response_time: Option<f64> = conn.query_row(
            "SELECT AVG(duration_ms) FROM analytics_events WHERE duration_ms IS NOT NULL",
            [],
            |row| row.get(0),
        ).optional()?;
        stats.insert("average_response_time_ms".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(avg_response_time.unwrap_or(0.0)).unwrap_or(serde_json::Number::from(0))));
        
        // Event type distribution
        let mut stmt = conn.prepare(
            "SELECT event_type, COUNT(*) as count FROM analytics_events GROUP BY event_type ORDER BY count DESC"
        )?;
        
        let event_type_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
        })?;
        
        let mut event_type_distribution = HashMap::new();
        for row in event_type_rows {
            let (event_type, count) = row?;
            event_type_distribution.insert(event_type, serde_json::Value::Number(count.into()));
        }
        stats.insert("event_type_distribution".to_string(), serde_json::Value::Object(event_type_distribution.into_iter().collect()));
        
        Ok(stats)
    }

    async fn generate_usage_report(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<serde_json::Value> {
        let conn = self.db.lock().unwrap();
        
        let start_str = start_date.to_rfc3339();
        let end_str = end_date.to_rfc3339();
        
        // Events in time range
        let events_in_range: u64 = conn.query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE timestamp >= ?1 AND timestamp <= ?2",
            params![start_str, end_str],
            |row| row.get(0),
        )?;
        
        // Success rate in time range
        let successful_events_in_range: u64 = conn.query_row(
            "SELECT COUNT(*) FROM analytics_events WHERE timestamp >= ?1 AND timestamp <= ?2 AND success = 1",
            params![start_str, end_str],
            |row| row.get(0),
        )?;
        
        let success_rate = if events_in_range > 0 {
            successful_events_in_range as f64 / events_in_range as f64
        } else {
            0.0
        };
        
        // Most active projects
        let mut stmt = conn.prepare(
            "SELECT project_id, COUNT(*) as count FROM analytics_events 
             WHERE timestamp >= ?1 AND timestamp <= ?2 AND project_id IS NOT NULL
             GROUP BY project_id ORDER BY count DESC LIMIT 10"
        )?;
        
        let project_rows = stmt.query_map(params![start_str, end_str], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
        })?;
        
        let mut most_active_projects = Vec::new();
        for row in project_rows {
            let (project_id, count) = row?;
            most_active_projects.push(serde_json::json!({
                "project_id": project_id,
                "event_count": count
            }));
        }
        
        Ok(serde_json::json!({
            "report_period": {
                "start": start_str,
                "end": end_str
            },
            "summary": {
                "total_events": events_in_range,
                "successful_events": successful_events_in_range,
                "success_rate": success_rate
            },
            "most_active_projects": most_active_projects
        }))
    }
}