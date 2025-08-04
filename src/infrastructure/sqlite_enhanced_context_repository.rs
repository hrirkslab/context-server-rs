use crate::models::enhanced_context::{EnhancedContextItem, ContextType, ContextId, ProjectId};
use crate::repositories::EnhancedContextRepository;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;
use rusqlite::{params, Connection, Row};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

/// SQLite implementation of EnhancedContextRepository
pub struct SqliteEnhancedContextRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteEnhancedContextRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }
    
    fn db_error(msg: &str, e: impl std::fmt::Display) -> McpError {
        McpError::internal_error(format!("{}: {}", msg, e), None)
    }

    /// Initialize the enhanced context tables
    pub fn initialize_tables(&self) -> Result<(), McpError> {
        let db = self.db.lock().map_err(|e| Self::db_error("Database lock error", e))?;

        // Create enhanced_context_items table
        db.execute(
            r#"
            CREATE TABLE IF NOT EXISTS enhanced_context_items (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                content_type TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                data TEXT NOT NULL, -- JSON data
                source_file TEXT,
                source_line INTEGER,
                quality_score REAL NOT NULL DEFAULT 0.0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                -- Metadata fields
                tags TEXT, -- JSON array
                priority TEXT NOT NULL DEFAULT 'medium',
                confidence REAL NOT NULL DEFAULT 1.0,
                source TEXT NOT NULL DEFAULT 'manual',
                validation_status TEXT NOT NULL DEFAULT 'pending',
                last_accessed TEXT,
                access_count INTEGER NOT NULL DEFAULT 0,
                custom_fields TEXT -- JSON object
            )
            "#,
            [],
        ).map_err(|e| Self::db_error("Failed to create enhanced_context_items table", e))?;

        // Create indexes for better performance
        db.execute("CREATE INDEX IF NOT EXISTS idx_enhanced_context_project ON enhanced_context_items (project_id)", []).ok();
        db.execute("CREATE INDEX IF NOT EXISTS idx_enhanced_context_type ON enhanced_context_items (content_type)", []).ok();

        Ok(())
    }

    fn row_to_enhanced_context_item(&self, row: &Row) -> Result<EnhancedContextItem, rusqlite::Error> {
        use crate::models::enhanced_context::*;
        
        let id: String = row.get("id")?;
        let project_id: String = row.get("project_id")?;
        let content_type_str: String = row.get("content_type")?;
        let title: String = row.get("title")?;
        let description: String = row.get("description")?;
        let data_str: String = row.get("data")?;
        let source_file: Option<String> = row.get("source_file")?;
        let source_line: Option<u32> = row.get("source_line")?;
        let quality_score: f64 = row.get("quality_score")?;
        let created_at_str: String = row.get("created_at")?;
        let updated_at_str: String = row.get("updated_at")?;
        let version: u32 = row.get("version")?;
        
        // Parse metadata fields
        let tags_str: Option<String> = row.get("tags")?;
        let priority_str: String = row.get("priority")?;
        let confidence: f64 = row.get("confidence")?;
        let source_str: String = row.get("source")?;
        let validation_status_str: String = row.get("validation_status")?;
        let last_accessed_str: Option<String> = row.get("last_accessed")?;
        let access_count: u64 = row.get("access_count")?;
        let custom_fields_str: Option<String> = row.get("custom_fields")?;

        // Parse content type
        let content_type = match content_type_str.as_str() {
            "business_rule" => ContextType::BusinessRule,
            "architectural_decision" => ContextType::ArchitecturalDecision,
            "performance_requirement" => ContextType::PerformanceRequirement,
            "security_policy" => ContextType::SecurityPolicy,
            "project_convention" => ContextType::ProjectConvention,
            "feature_context" => ContextType::FeatureContext,
            "code_pattern" => ContextType::CodePattern,
            "api_specification" => ContextType::ApiSpecification,
            "database_schema" => ContextType::DatabaseSchema,
            "test_case" => ContextType::TestCase,
            "documentation" => ContextType::Documentation,
            custom => ContextType::Custom(custom.to_string()),
        };

        // Parse data
        let data = serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);

        // Parse dates
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        // Parse tags
        let tags = if let Some(tags_str) = tags_str {
            serde_json::from_str::<Vec<String>>(&tags_str).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Parse priority
        let priority = match priority_str.as_str() {
            "critical" => Priority::Critical,
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::Medium,
        };

        // Parse source
        let source = match source_str.as_str() {
            "manual" => ContextSource::Manual,
            "auto_detected" => ContextSource::AutoDetected,
            "code_analysis" => ContextSource::CodeAnalysis,
            "documentation" => ContextSource::Documentation,
            "git" => ContextSource::Git,
            plugin => ContextSource::Plugin(plugin.to_string()),
        };

        // Parse validation status
        let validation_status = match validation_status_str.as_str() {
            "pending" => ValidationStatus::Pending,
            "valid" => ValidationStatus::Valid,
            "invalid" => ValidationStatus::Invalid,
            "needs_review" => ValidationStatus::NeedsReview,
            "outdated" => ValidationStatus::Outdated,
            _ => ValidationStatus::Pending,
        };

        // Parse last accessed
        let last_accessed = if let Some(last_accessed_str) = last_accessed_str {
            DateTime::parse_from_rfc3339(&last_accessed_str)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        } else {
            None
        };

        // Parse custom fields
        let custom_fields = if let Some(custom_fields_str) = custom_fields_str {
            serde_json::from_str(&custom_fields_str).unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

        Ok(EnhancedContextItem {
            id,
            project_id,
            content: ContextContent {
                content_type,
                title,
                description,
                data,
                source_file,
                source_line,
            },
            metadata: ContextMetadata {
                tags,
                priority,
                confidence,
                source,
                validation_status,
                last_accessed,
                access_count,
                custom_fields,
            },
            relationships: Vec::new(), // Will be loaded separately if needed
            quality_score,
            usage_stats: UsageStatistics::default(), // Will be loaded separately if needed
            semantic_tags: Vec::new(), // Will be loaded separately if needed
            created_at,
            updated_at,
            version,
        })
    }
}

#[async_trait]
impl EnhancedContextRepository for SqliteEnhancedContextRepository {
    async fn create_context(&self, context: &EnhancedContextItem) -> Result<EnhancedContextItem, McpError> {
        let db = self.db.lock().map_err(|e| Self::db_error("Database lock error", e))?;

        let data_json = serde_json::to_string(&context.content.data).unwrap_or_default();
        let tags_json = serde_json::to_string(&context.metadata.tags).unwrap_or_default();
        let custom_fields_json = serde_json::to_string(&context.metadata.custom_fields).unwrap_or_default();

        db.execute(
            r#"
            INSERT INTO enhanced_context_items (
                id, project_id, content_type, title, description, data, source_file, source_line,
                quality_score, created_at, updated_at, version, tags, priority, confidence,
                source, validation_status, last_accessed, access_count, custom_fields
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
            "#,
            params![
                context.id,
                context.project_id,
                context.content.content_type.as_str(),
                context.content.title,
                context.content.description,
                data_json,
                context.content.source_file,
                context.content.source_line,
                context.quality_score,
                context.created_at.to_rfc3339(),
                context.updated_at.to_rfc3339(),
                context.version,
                tags_json,
                context.metadata.priority.as_str(),
                context.metadata.confidence,
                context.metadata.source.as_str(),
                context.metadata.validation_status.as_str(),
                context.metadata.last_accessed.map(|dt| dt.to_rfc3339()),
                context.metadata.access_count,
                custom_fields_json,
            ],
        ).map_err(|e| Self::db_error("Failed to create enhanced context item", e))?;

        Ok(context.clone())
    }

    async fn find_context_by_id(&self, id: &str) -> Result<Option<EnhancedContextItem>, McpError> {
        let db = self.db.lock().map_err(|e| Self::db_error("Database lock error", e))?;

        let mut stmt = db.prepare(
            "SELECT * FROM enhanced_context_items WHERE id = ?1"
        ).map_err(|e| Self::db_error("Failed to prepare statement", e))?;

        let context_iter = stmt.query_map(params![id], |row| {
            self.row_to_enhanced_context_item(row)
        }).map_err(|e| Self::db_error("Failed to query context", e))?;

        for context in context_iter {
            return Ok(Some(context.map_err(|e| Self::db_error("Failed to parse context", e))?));
        }

        Ok(None)
    }

    async fn find_contexts_by_project(&self, project_id: &str) -> Result<Vec<EnhancedContextItem>, McpError> {
        let db = self.db.lock().map_err(|e| Self::db_error("Database lock error", e))?;

        let mut stmt = db.prepare(
            "SELECT * FROM enhanced_context_items WHERE project_id = ?1 ORDER BY updated_at DESC"
        ).map_err(|e| Self::db_error("Failed to prepare statement", e))?;

        let context_iter = stmt.query_map(params![project_id], |row| {
            self.row_to_enhanced_context_item(row)
        }).map_err(|e| Self::db_error("Failed to query contexts", e))?;

        let mut contexts = Vec::new();
        for context in context_iter {
            contexts.push(context.map_err(|e| Self::db_error("Failed to parse context", e))?);
        }

        Ok(contexts)
    }

    async fn find_contexts_by_type(&self, project_id: &str, context_type: ContextType) -> Result<Vec<EnhancedContextItem>, McpError> {
        let db = self.db.lock().map_err(|e| Self::db_error("Database lock error", e))?;

        let mut stmt = db.prepare(
            "SELECT * FROM enhanced_context_items WHERE project_id = ?1 AND content_type = ?2 ORDER BY updated_at DESC"
        ).map_err(|e| Self::db_error("Failed to prepare statement", e))?;

        let context_iter = stmt.query_map(params![project_id, context_type.as_str()], |row| {
            self.row_to_enhanced_context_item(row)
        }).map_err(|e| Self::db_error("Failed to query contexts", e))?;

        let mut contexts = Vec::new();
        for context in context_iter {
            contexts.push(context.map_err(|e| Self::db_error("Failed to parse context", e))?);
        }

        Ok(contexts)
    }

    async fn find_contexts_by_keywords(&self, project_id: &str, keywords: &[String]) -> Result<Vec<EnhancedContextItem>, McpError> {
        if keywords.is_empty() {
            return self.find_contexts_by_project(project_id).await;
        }

        let db = self.db.lock().map_err(|e| Self::db_error("Database lock error", e))?;

        // Build a simple keyword search query
        let keyword_conditions: Vec<String> = keywords.iter()
            .map(|_| "(title LIKE ? OR description LIKE ?)".to_string())
            .collect();
        let where_clause = keyword_conditions.join(" OR ");

        let query = format!(
            "SELECT * FROM enhanced_context_items WHERE project_id = ? AND ({}) ORDER BY updated_at DESC",
            where_clause
        );

        let mut stmt = db.prepare(&query).map_err(|e| Self::db_error("Failed to prepare statement", e))?;

        // Build parameters
        let mut params = vec![project_id.to_string()];
        for keyword in keywords {
            let pattern = format!("%{}%", keyword);
            params.push(pattern.clone());
            params.push(pattern);
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter()
            .map(|p| p as &dyn rusqlite::ToSql)
            .collect();

        let context_iter = stmt.query_map(&param_refs[..], |row| {
            self.row_to_enhanced_context_item(row)
        }).map_err(|e| Self::db_error("Failed to query contexts", e))?;

        let mut contexts = Vec::new();
        for context in context_iter {
            contexts.push(context.map_err(|e| Self::db_error("Failed to parse context", e))?);
        }

        Ok(contexts)
    }

    async fn update_context(&self, context: &EnhancedContextItem) -> Result<EnhancedContextItem, McpError> {
        let db = self.db.lock().map_err(|e| Self::db_error("Database lock error", e))?;

        let data_json = serde_json::to_string(&context.content.data).unwrap_or_default();
        let tags_json = serde_json::to_string(&context.metadata.tags).unwrap_or_default();
        let custom_fields_json = serde_json::to_string(&context.metadata.custom_fields).unwrap_or_default();

        db.execute(
            r#"
            UPDATE enhanced_context_items SET
                project_id = ?2, content_type = ?3, title = ?4, description = ?5, data = ?6,
                source_file = ?7, source_line = ?8, quality_score = ?9, updated_at = ?10,
                version = ?11, tags = ?12, priority = ?13, confidence = ?14, source = ?15,
                validation_status = ?16, last_accessed = ?17, access_count = ?18, custom_fields = ?19
            WHERE id = ?1
            "#,
            params![
                context.id,
                context.project_id,
                context.content.content_type.as_str(),
                context.content.title,
                context.content.description,
                data_json,
                context.content.source_file,
                context.content.source_line,
                context.quality_score,
                context.updated_at.to_rfc3339(),
                context.version,
                tags_json,
                context.metadata.priority.as_str(),
                context.metadata.confidence,
                context.metadata.source.as_str(),
                context.metadata.validation_status.as_str(),
                context.metadata.last_accessed.map(|dt| dt.to_rfc3339()),
                context.metadata.access_count,
                custom_fields_json,
            ],
        ).map_err(|e| Self::db_error("Failed to update enhanced context item", e))?;

        Ok(context.clone())
    }

    async fn delete_context(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().map_err(|e| Self::db_error("Database lock error", e))?;

        let rows_affected = db.execute(
            "DELETE FROM enhanced_context_items WHERE id = ?1",
            params![id],
        ).map_err(|e| Self::db_error("Failed to delete enhanced context item", e))?;

        Ok(rows_affected > 0)
    }

    async fn find_contexts_linked_to_requirement(&self, _requirement_id: &str) -> Result<Vec<EnhancedContextItem>, McpError> {
        // Simplified implementation - in a full implementation, you'd have a linking table
        Ok(Vec::new())
    }

    async fn find_contexts_linked_to_task(&self, _task_id: &str) -> Result<Vec<EnhancedContextItem>, McpError> {
        // Simplified implementation - in a full implementation, you'd have a linking table
        Ok(Vec::new())
    }

    async fn find_related_contexts(&self, _context_id: &str) -> Result<Vec<EnhancedContextItem>, McpError> {
        // Simplified implementation - in a full implementation, you'd have a relationships table
        Ok(Vec::new())
    }

    async fn update_quality_score(&self, context_id: &str, score: f64) -> Result<(), McpError> {
        let db = self.db.lock().map_err(|e| Self::db_error("Database lock error", e))?;

        db.execute(
            "UPDATE enhanced_context_items SET quality_score = ?1, updated_at = ?2 WHERE id = ?3",
            params![score, Utc::now().to_rfc3339(), context_id],
        ).map_err(|e| Self::db_error("Failed to update quality score", e))?;

        Ok(())
    }

    async fn record_context_usage(&self, context_id: &str) -> Result<(), McpError> {
        let db = self.db.lock().map_err(|e| Self::db_error("Database lock error", e))?;

        let now = Utc::now().to_rfc3339();

        // Update context metadata
        db.execute(
            r#"
            UPDATE enhanced_context_items SET
                access_count = access_count + 1,
                last_accessed = ?1
            WHERE id = ?2
            "#,
            params![now, context_id],
        ).map_err(|e| Self::db_error("Failed to update context access", e))?;

        Ok(())
    }
}