use crate::models::specification::{
    AcceptanceCriterion, CriterionStatus, CriterionType, Priority, ProjectSpecification,
    Requirement, RequirementStatus, SpecFormat, SpecStatus, SpecType, Task, TaskStatus, TaskType,
    SpecContent, RequirementMetadata, TaskMetadata,
};
use crate::repositories::SpecificationRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rmcp::model::ErrorData as McpError;
use rusqlite::{params, Connection, Row};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// SQLite implementation of SpecificationRepository
pub struct SqliteSpecificationRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteSpecificationRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    /// Initialize database tables for specifications
    pub fn initialize_tables(&self) -> Result<(), McpError> {
        let db = self.db.lock().unwrap();

        // Create specifications table
        db.execute(
            r#"
            CREATE TABLE IF NOT EXISTS specifications (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                spec_type TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                content_format TEXT NOT NULL,
                raw_content TEXT NOT NULL,
                parsed_sections TEXT, -- JSON
                content_metadata TEXT, -- JSON
                status TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                file_path TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT -- JSON
            )
            "#,
            [],
        ).map_err(|e| McpError::internal_error(format!("Failed to create specifications table: {}", e), None))?;

        // Create requirements table
        db.execute(
            r#"
            CREATE TABLE IF NOT EXISTS requirements (
                id TEXT PRIMARY KEY,
                spec_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                user_story TEXT,
                priority TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT, -- JSON
                FOREIGN KEY (spec_id) REFERENCES specifications (id) ON DELETE CASCADE
            )
            "#,
            [],
        ).map_err(|e| McpError::internal_error(format!("Failed to create requirements table: {}", e), None))?;

        // Create acceptance_criteria table
        db.execute(
            r#"
            CREATE TABLE IF NOT EXISTS acceptance_criteria (
                id TEXT PRIMARY KEY,
                requirement_id TEXT NOT NULL,
                description TEXT NOT NULL,
                criterion_type TEXT NOT NULL,
                status TEXT NOT NULL,
                test_cases TEXT, -- JSON array
                created_at TEXT NOT NULL,
                FOREIGN KEY (requirement_id) REFERENCES requirements (id) ON DELETE CASCADE
            )
            "#,
            [],
        ).map_err(|e| McpError::internal_error(format!("Failed to create acceptance_criteria table: {}", e), None))?;

        // Create tasks table
        db.execute(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                spec_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                task_type TEXT NOT NULL,
                parent_task TEXT,
                estimated_effort TEXT,
                actual_effort TEXT,
                assigned_to TEXT,
                progress REAL NOT NULL DEFAULT 0.0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT,
                metadata TEXT, -- JSON
                FOREIGN KEY (spec_id) REFERENCES specifications (id) ON DELETE CASCADE,
                FOREIGN KEY (parent_task) REFERENCES tasks (id) ON DELETE SET NULL
            )
            "#,
            [],
        ).map_err(|e| McpError::internal_error(format!("Failed to create tasks table: {}", e), None))?;

        // Create task_dependencies table
        db.execute(
            r#"
            CREATE TABLE IF NOT EXISTS task_dependencies (
                task_id TEXT NOT NULL,
                depends_on_task_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (task_id, depends_on_task_id),
                FOREIGN KEY (task_id) REFERENCES tasks (id) ON DELETE CASCADE,
                FOREIGN KEY (depends_on_task_id) REFERENCES tasks (id) ON DELETE CASCADE
            )
            "#,
            [],
        ).map_err(|e| McpError::internal_error(format!("Failed to create task_dependencies table: {}", e), None))?;

        // Create requirement_context_links table
        db.execute(
            r#"
            CREATE TABLE IF NOT EXISTS requirement_context_links (
                requirement_id TEXT NOT NULL,
                context_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (requirement_id, context_id),
                FOREIGN KEY (requirement_id) REFERENCES requirements (id) ON DELETE CASCADE
            )
            "#,
            [],
        ).map_err(|e| McpError::internal_error(format!("Failed to create requirement_context_links table: {}", e), None))?;

        // Create task_context_links table
        db.execute(
            r#"
            CREATE TABLE IF NOT EXISTS task_context_links (
                task_id TEXT NOT NULL,
                context_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (task_id, context_id),
                FOREIGN KEY (task_id) REFERENCES tasks (id) ON DELETE CASCADE
            )
            "#,
            [],
        ).map_err(|e| McpError::internal_error(format!("Failed to create task_context_links table: {}", e), None))?;

        // Create task_requirement_links table
        db.execute(
            r#"
            CREATE TABLE IF NOT EXISTS task_requirement_links (
                task_id TEXT NOT NULL,
                requirement_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (task_id, requirement_id),
                FOREIGN KEY (task_id) REFERENCES tasks (id) ON DELETE CASCADE,
                FOREIGN KEY (requirement_id) REFERENCES requirements (id) ON DELETE CASCADE
            )
            "#,
            [],
        ).map_err(|e| McpError::internal_error(format!("Failed to create task_requirement_links table: {}", e), None))?;

        // Create indexes for better performance
        db.execute("CREATE INDEX IF NOT EXISTS idx_specifications_project_id ON specifications (project_id)", [])
            .map_err(|e| McpError::internal_error(format!("Failed to create index: {}", e), None))?;
        
        db.execute("CREATE INDEX IF NOT EXISTS idx_requirements_spec_id ON requirements (spec_id)", [])
            .map_err(|e| McpError::internal_error(format!("Failed to create index: {}", e), None))?;
        
        db.execute("CREATE INDEX IF NOT EXISTS idx_tasks_spec_id ON tasks (spec_id)", [])
            .map_err(|e| McpError::internal_error(format!("Failed to create index: {}", e), None))?;

        Ok(())
    }

    fn row_to_specification(row: &Row) -> Result<ProjectSpecification, rusqlite::Error> {
        let parsed_sections: HashMap<String, String> = row.get::<_, Option<String>>(8)?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let content_metadata: HashMap<String, serde_json::Value> = row.get::<_, Option<String>>(9)?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let spec_metadata = row.get::<_, Option<String>>(15)?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let content = SpecContent {
            format: SpecFormat::from_extension(&row.get::<_, String>(5)?),
            raw_content: row.get(6)?,
            parsed_sections,
            metadata: content_metadata,
        };

        Ok(ProjectSpecification {
            id: row.get(0)?,
            project_id: row.get(1)?,
            spec_type: Self::parse_spec_type(&row.get::<_, String>(2)?),
            title: row.get(3)?,
            description: row.get(4)?,
            content,
            requirements: Vec::new(), // Will be loaded separately
            tasks: Vec::new(), // Will be loaded separately
            status: Self::parse_spec_status(&row.get::<_, String>(10)?),
            version: row.get::<_, i64>(11)? as u32,
            file_path: row.get(12)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(13, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(14)?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(14, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            metadata: spec_metadata,
        })
    }

    fn row_to_requirement(row: &Row) -> Result<Requirement, rusqlite::Error> {
        let metadata: RequirementMetadata = row.get::<_, Option<String>>(9)?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Ok(Requirement {
            id: row.get(0)?,
            spec_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            user_story: row.get(4)?,
            acceptance_criteria: Vec::new(), // Will be loaded separately
            priority: Self::parse_priority(&row.get::<_, String>(5)?),
            status: Self::parse_requirement_status(&row.get::<_, String>(6)?),
            linked_tasks: Vec::new(), // Will be loaded separately
            linked_context: Vec::new(), // Will be loaded separately
            dependencies: Vec::new(), // Will be loaded separately
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(7, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(8, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            metadata,
        })
    }

    fn row_to_task(row: &Row) -> Result<Task, rusqlite::Error> {
        let metadata: TaskMetadata = row.get::<_, Option<String>>(15)?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let started_at = row.get::<_, Option<String>>(13)?
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let completed_at = row.get::<_, Option<String>>(14)?
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(Task {
            id: row.get(0)?,
            spec_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            status: Self::parse_task_status(&row.get::<_, String>(4)?),
            task_type: Self::parse_task_type(&row.get::<_, String>(5)?),
            dependencies: Vec::new(), // Will be loaded separately
            subtasks: Vec::new(), // Will be loaded separately
            parent_task: row.get(6)?,
            estimated_effort: row.get(7)?,
            actual_effort: row.get(8)?,
            assigned_to: row.get(9)?,
            linked_requirements: Vec::new(), // Will be loaded separately
            linked_context: Vec::new(), // Will be loaded separately
            progress: row.get(10)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(11)?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(11, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(12)?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(12, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            started_at,
            completed_at,
            metadata,
        })
    }

    fn parse_spec_type(s: &str) -> SpecType {
        match s {
            "feature" => SpecType::Feature,
            "architecture" => SpecType::Architecture,
            "api" => SpecType::API,
            "database" => SpecType::Database,
            "security" => SpecType::Security,
            "performance" => SpecType::Performance,
            "requirements" => SpecType::Requirements,
            "design" => SpecType::Design,
            "tasks" => SpecType::Tasks,
            _ => SpecType::Custom(s.to_string()),
        }
    }

    fn parse_spec_status(s: &str) -> SpecStatus {
        match s {
            "draft" => SpecStatus::Draft,
            "in_review" => SpecStatus::InReview,
            "approved" => SpecStatus::Approved,
            "in_progress" => SpecStatus::InProgress,
            "completed" => SpecStatus::Completed,
            "archived" => SpecStatus::Archived,
            "deprecated" => SpecStatus::Deprecated,
            _ => SpecStatus::Draft,
        }
    }

    fn parse_priority(s: &str) -> Priority {
        match s {
            "critical" => Priority::Critical,
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::Medium,
        }
    }

    fn parse_requirement_status(s: &str) -> RequirementStatus {
        match s {
            "draft" => RequirementStatus::Draft,
            "defined" => RequirementStatus::Defined,
            "in_progress" => RequirementStatus::InProgress,
            "implemented" => RequirementStatus::Implemented,
            "tested" => RequirementStatus::Tested,
            "accepted" => RequirementStatus::Accepted,
            "rejected" => RequirementStatus::Rejected,
            "deferred" => RequirementStatus::Deferred,
            _ => RequirementStatus::Draft,
        }
    }

    fn parse_task_status(s: &str) -> TaskStatus {
        match s {
            "not_started" => TaskStatus::NotStarted,
            "in_progress" => TaskStatus::InProgress,
            "completed" => TaskStatus::Completed,
            "blocked" => TaskStatus::Blocked,
            "on_hold" => TaskStatus::OnHold,
            "cancelled" => TaskStatus::Cancelled,
            "deferred" => TaskStatus::Deferred,
            _ => TaskStatus::NotStarted,
        }
    }

    fn parse_task_type(s: &str) -> TaskType {
        match s {
            "implementation" => TaskType::Implementation,
            "testing" => TaskType::Testing,
            "documentation" => TaskType::Documentation,
            "research" => TaskType::Research,
            "design" => TaskType::Design,
            "review" => TaskType::Review,
            "deployment" => TaskType::Deployment,
            "maintenance" => TaskType::Maintenance,
            _ => TaskType::Custom(s.to_string()),
        }
    }
}

#[async_trait]
impl SpecificationRepository for SqliteSpecificationRepository {
    async fn create_specification(&self, spec: &ProjectSpecification) -> Result<ProjectSpecification, McpError> {
        let db = self.db.lock().unwrap();

        let parsed_sections_json = serde_json::to_string(&spec.content.parsed_sections)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize parsed sections: {}", e), None))?;

        let content_metadata_json = serde_json::to_string(&spec.content.metadata)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize content metadata: {}", e), None))?;

        let spec_metadata_json = serde_json::to_string(&spec.metadata)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize spec metadata: {}", e), None))?;

        db.execute(
            r#"
            INSERT INTO specifications (
                id, project_id, spec_type, title, description, content_format, raw_content,
                parsed_sections, content_metadata, status, version, file_path, created_at, updated_at, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                &spec.id,
                &spec.project_id,
                spec.spec_type.as_str(),
                &spec.title,
                &spec.description,
                spec.content.format.as_str(),
                &spec.content.raw_content,
                parsed_sections_json,
                content_metadata_json,
                spec.status.as_str(),
                spec.version,
                &spec.file_path,
                spec.created_at.to_rfc3339(),
                spec.updated_at.to_rfc3339(),
                spec_metadata_json,
            ],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(spec.clone())
    }

    async fn find_specification_by_id(&self, id: &str) -> Result<Option<ProjectSpecification>, McpError> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare(
            r#"
            SELECT id, project_id, spec_type, title, description, content_format, raw_content,
                   parsed_sections, content_metadata, status, version, file_path, created_at, updated_at, metadata
            FROM specifications WHERE id = ?
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let mut spec_iter = stmt.query_map([id], Self::row_to_specification)
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        match spec_iter.next() {
            Some(Ok(spec)) => Ok(Some(spec)),
            Some(Err(e)) => Err(McpError::internal_error(format!("Database error: {}", e), None)),
            None => Ok(None),
        }
    }

    async fn find_specifications_by_project(&self, project_id: &str) -> Result<Vec<ProjectSpecification>, McpError> {
        let db = self.db.lock().unwrap();
        let mut specifications = Vec::new();

        let mut stmt = db.prepare(
            r#"
            SELECT id, project_id, spec_type, title, description, content_format, raw_content,
                   parsed_sections, content_metadata, status, version, file_path, created_at, updated_at, metadata
            FROM specifications WHERE project_id = ? ORDER BY created_at DESC
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let spec_rows = stmt.query_map([project_id], Self::row_to_specification)
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for spec in spec_rows {
            match spec {
                Ok(spec) => specifications.push(spec),
                Err(e) => tracing::warn!("Failed to parse specification: {}", e),
            }
        }

        Ok(specifications)
    }

    async fn find_specifications_by_type(&self, project_id: &str, spec_type: &str) -> Result<Vec<ProjectSpecification>, McpError> {
        let db = self.db.lock().unwrap();
        let mut specifications = Vec::new();

        let mut stmt = db.prepare(
            r#"
            SELECT id, project_id, spec_type, title, description, content_format, raw_content,
                   parsed_sections, content_metadata, status, version, file_path, created_at, updated_at, metadata
            FROM specifications WHERE project_id = ? AND spec_type = ? ORDER BY created_at DESC
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let spec_rows = stmt.query_map([project_id, spec_type], Self::row_to_specification)
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for spec in spec_rows {
            match spec {
                Ok(spec) => specifications.push(spec),
                Err(e) => tracing::warn!("Failed to parse specification: {}", e),
            }
        }

        Ok(specifications)
    }

    async fn update_specification(&self, spec: &ProjectSpecification) -> Result<ProjectSpecification, McpError> {
        let db = self.db.lock().unwrap();

        let parsed_sections_json = serde_json::to_string(&spec.content.parsed_sections)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize parsed sections: {}", e), None))?;

        let content_metadata_json = serde_json::to_string(&spec.content.metadata)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize content metadata: {}", e), None))?;

        let spec_metadata_json = serde_json::to_string(&spec.metadata)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize spec metadata: {}", e), None))?;

        db.execute(
            r#"
            UPDATE specifications SET
                title = ?, description = ?, content_format = ?, raw_content = ?,
                parsed_sections = ?, content_metadata = ?, status = ?, version = ?,
                file_path = ?, updated_at = ?, metadata = ?
            WHERE id = ?
            "#,
            params![
                &spec.title,
                &spec.description,
                spec.content.format.as_str(),
                &spec.content.raw_content,
                parsed_sections_json,
                content_metadata_json,
                spec.status.as_str(),
                spec.version,
                &spec.file_path,
                spec.updated_at.to_rfc3339(),
                spec_metadata_json,
                &spec.id,
            ],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(spec.clone())
    }

    async fn delete_specification(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().unwrap();

        let rows_affected = db.execute("DELETE FROM specifications WHERE id = ?", [id])
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(rows_affected > 0)
    }

    async fn create_requirement(&self, requirement: &Requirement) -> Result<Requirement, McpError> {
        let db = self.db.lock().unwrap();

        let metadata_json = serde_json::to_string(&requirement.metadata)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize requirement metadata: {}", e), None))?;

        db.execute(
            r#"
            INSERT INTO requirements (
                id, spec_id, title, description, user_story, priority, status,
                created_at, updated_at, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                &requirement.id,
                &requirement.spec_id,
                &requirement.title,
                &requirement.description,
                &requirement.user_story,
                requirement.priority.as_str(),
                requirement.status.as_str(),
                requirement.created_at.to_rfc3339(),
                requirement.updated_at.to_rfc3339(),
                metadata_json,
            ],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        // Insert acceptance criteria
        for criterion in &requirement.acceptance_criteria {
            let test_cases_json = serde_json::to_string(&criterion.test_cases)
                .map_err(|e| McpError::internal_error(format!("Failed to serialize test cases: {}", e), None))?;

            db.execute(
                r#"
                INSERT INTO acceptance_criteria (
                    id, requirement_id, description, criterion_type, status, test_cases, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    &criterion.id,
                    &requirement.id,
                    &criterion.description,
                    format!("{:?}", criterion.criterion_type),
                    format!("{:?}", criterion.status),
                    test_cases_json,
                    criterion.created_at.to_rfc3339(),
                ],
            ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        }

        Ok(requirement.clone())
    }

    async fn find_requirement_by_id(&self, id: &str) -> Result<Option<Requirement>, McpError> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare(
            r#"
            SELECT id, spec_id, title, description, user_story, priority, status,
                   created_at, updated_at, metadata
            FROM requirements WHERE id = ?
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let mut req_iter = stmt.query_map([id], Self::row_to_requirement)
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        match req_iter.next() {
            Some(Ok(mut requirement)) => {
                // Load acceptance criteria
                let mut criteria_stmt = db.prepare(
                    "SELECT id, description, criterion_type, status, test_cases, created_at FROM acceptance_criteria WHERE requirement_id = ?"
                ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

                let criteria_rows = criteria_stmt.query_map([&requirement.id], |row| {
                    let test_cases: Vec<String> = row.get::<_, Option<String>>(4)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default();

                    Ok(AcceptanceCriterion {
                        id: row.get(0)?,
                        description: row.get(1)?,
                        criterion_type: CriterionType::Functional, // Simplified for now
                        status: CriterionStatus::Pending, // Simplified for now
                        test_cases,
                        created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                            .map_err(|_| rusqlite::Error::InvalidColumnType(5, "created_at".to_string(), rusqlite::types::Type::Text))?
                            .with_timezone(&Utc),
                    })
                }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

                for criterion in criteria_rows {
                    match criterion {
                        Ok(criterion) => requirement.acceptance_criteria.push(criterion),
                        Err(e) => tracing::warn!("Failed to parse acceptance criterion: {}", e),
                    }
                }

                Ok(Some(requirement))
            }
            Some(Err(e)) => Err(McpError::internal_error(format!("Database error: {}", e), None)),
            None => Ok(None),
        }
    }

    async fn find_requirements_by_spec(&self, spec_id: &str) -> Result<Vec<Requirement>, McpError> {
        let db = self.db.lock().unwrap();
        let mut requirements = Vec::new();

        let mut stmt = db.prepare(
            r#"
            SELECT id, spec_id, title, description, user_story, priority, status,
                   created_at, updated_at, metadata
            FROM requirements WHERE spec_id = ? ORDER BY created_at ASC
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let req_rows = stmt.query_map([spec_id], Self::row_to_requirement)
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for req in req_rows {
            match req {
                Ok(requirement) => requirements.push(requirement),
                Err(e) => tracing::warn!("Failed to parse requirement: {}", e),
            }
        }

        Ok(requirements)
    }

    async fn update_requirement(&self, requirement: &Requirement) -> Result<Requirement, McpError> {
        let db = self.db.lock().unwrap();

        let metadata_json = serde_json::to_string(&requirement.metadata)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize requirement metadata: {}", e), None))?;

        db.execute(
            r#"
            UPDATE requirements SET
                title = ?, description = ?, user_story = ?, priority = ?, status = ?,
                updated_at = ?, metadata = ?
            WHERE id = ?
            "#,
            params![
                &requirement.title,
                &requirement.description,
                &requirement.user_story,
                requirement.priority.as_str(),
                requirement.status.as_str(),
                requirement.updated_at.to_rfc3339(),
                metadata_json,
                &requirement.id,
            ],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(requirement.clone())
    }

    async fn delete_requirement(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().unwrap();

        let rows_affected = db.execute("DELETE FROM requirements WHERE id = ?", [id])
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(rows_affected > 0)
    }

    async fn create_task(&self, task: &Task) -> Result<Task, McpError> {
        let db = self.db.lock().unwrap();

        let metadata_json = serde_json::to_string(&task.metadata)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize task metadata: {}", e), None))?;

        db.execute(
            r#"
            INSERT INTO tasks (
                id, spec_id, title, description, status, task_type, parent_task,
                estimated_effort, actual_effort, assigned_to, progress,
                created_at, updated_at, started_at, completed_at, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                &task.id,
                &task.spec_id,
                &task.title,
                &task.description,
                task.status.as_str(),
                task.task_type.as_str(),
                &task.parent_task,
                &task.estimated_effort,
                &task.actual_effort,
                &task.assigned_to,
                task.progress,
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
                task.started_at.map(|dt| dt.to_rfc3339()),
                task.completed_at.map(|dt| dt.to_rfc3339()),
                metadata_json,
            ],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        // Insert task dependencies
        for dep_id in &task.dependencies {
            db.execute(
                "INSERT INTO task_dependencies (task_id, depends_on_task_id, created_at) VALUES (?, ?, ?)",
                params![&task.id, dep_id, Utc::now().to_rfc3339()],
            ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        }

        Ok(task.clone())
    }

    async fn find_task_by_id(&self, id: &str) -> Result<Option<Task>, McpError> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare(
            r#"
            SELECT id, spec_id, title, description, status, task_type, parent_task,
                   estimated_effort, actual_effort, assigned_to, progress,
                   created_at, updated_at, started_at, completed_at, metadata
            FROM tasks WHERE id = ?
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let mut task_iter = stmt.query_map([id], Self::row_to_task)
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        match task_iter.next() {
            Some(Ok(mut task)) => {
                // Load dependencies
                let mut dep_stmt = db.prepare(
                    "SELECT depends_on_task_id FROM task_dependencies WHERE task_id = ?"
                ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

                let dep_rows = dep_stmt.query_map([&task.id], |row| {
                    Ok(row.get::<_, String>(0)?)
                }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

                for dep in dep_rows {
                    match dep {
                        Ok(dep_id) => task.dependencies.push(dep_id),
                        Err(e) => tracing::warn!("Failed to parse task dependency: {}", e),
                    }
                }

                // Load subtasks
                let mut subtask_stmt = db.prepare(
                    "SELECT id FROM tasks WHERE parent_task = ?"
                ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

                let subtask_rows = subtask_stmt.query_map([&task.id], |row| {
                    Ok(row.get::<_, String>(0)?)
                }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

                for subtask in subtask_rows {
                    match subtask {
                        Ok(subtask_id) => task.subtasks.push(subtask_id),
                        Err(e) => tracing::warn!("Failed to parse subtask: {}", e),
                    }
                }

                Ok(Some(task))
            }
            Some(Err(e)) => Err(McpError::internal_error(format!("Database error: {}", e), None)),
            None => Ok(None),
        }
    }

    async fn find_tasks_by_spec(&self, spec_id: &str) -> Result<Vec<Task>, McpError> {
        let db = self.db.lock().unwrap();
        let mut tasks = Vec::new();

        let mut stmt = db.prepare(
            r#"
            SELECT id, spec_id, title, description, status, task_type, parent_task,
                   estimated_effort, actual_effort, assigned_to, progress,
                   created_at, updated_at, started_at, completed_at, metadata
            FROM tasks WHERE spec_id = ? ORDER BY created_at ASC
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let task_rows = stmt.query_map([spec_id], Self::row_to_task)
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for task in task_rows {
            match task {
                Ok(task) => tasks.push(task),
                Err(e) => tracing::warn!("Failed to parse task: {}", e),
            }
        }

        Ok(tasks)
    }

    async fn find_tasks_by_status(&self, spec_id: &str, status: &str) -> Result<Vec<Task>, McpError> {
        let db = self.db.lock().unwrap();
        let mut tasks = Vec::new();

        let mut stmt = db.prepare(
            r#"
            SELECT id, spec_id, title, description, status, task_type, parent_task,
                   estimated_effort, actual_effort, assigned_to, progress,
                   created_at, updated_at, started_at, completed_at, metadata
            FROM tasks WHERE spec_id = ? AND status = ? ORDER BY created_at ASC
            "#
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let task_rows = stmt.query_map([spec_id, status], Self::row_to_task)
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for task in task_rows {
            match task {
                Ok(task) => tasks.push(task),
                Err(e) => tracing::warn!("Failed to parse task: {}", e),
            }
        }

        Ok(tasks)
    }

    async fn update_task(&self, task: &Task) -> Result<Task, McpError> {
        let db = self.db.lock().unwrap();

        let metadata_json = serde_json::to_string(&task.metadata)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize task metadata: {}", e), None))?;

        db.execute(
            r#"
            UPDATE tasks SET
                title = ?, description = ?, status = ?, task_type = ?, parent_task = ?,
                estimated_effort = ?, actual_effort = ?, assigned_to = ?, progress = ?,
                updated_at = ?, started_at = ?, completed_at = ?, metadata = ?
            WHERE id = ?
            "#,
            params![
                &task.title,
                &task.description,
                task.status.as_str(),
                task.task_type.as_str(),
                &task.parent_task,
                &task.estimated_effort,
                &task.actual_effort,
                &task.assigned_to,
                task.progress,
                task.updated_at.to_rfc3339(),
                task.started_at.map(|dt| dt.to_rfc3339()),
                task.completed_at.map(|dt| dt.to_rfc3339()),
                metadata_json,
                &task.id,
            ],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(task.clone())
    }

    async fn delete_task(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().unwrap();

        let rows_affected = db.execute("DELETE FROM tasks WHERE id = ?", [id])
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(rows_affected > 0)
    }

    async fn link_requirement_to_context(&self, requirement_id: &str, context_id: &str) -> Result<(), McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT OR IGNORE INTO requirement_context_links (requirement_id, context_id, created_at) VALUES (?, ?, ?)",
            params![requirement_id, context_id, Utc::now().to_rfc3339()],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(())
    }

    async fn link_task_to_context(&self, task_id: &str, context_id: &str) -> Result<(), McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT OR IGNORE INTO task_context_links (task_id, context_id, created_at) VALUES (?, ?, ?)",
            params![task_id, context_id, Utc::now().to_rfc3339()],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(())
    }

    async fn link_task_to_requirement(&self, task_id: &str, requirement_id: &str) -> Result<(), McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT OR IGNORE INTO task_requirement_links (task_id, requirement_id, created_at) VALUES (?, ?, ?)",
            params![task_id, requirement_id, Utc::now().to_rfc3339()],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(())
    }

    async fn unlink_requirement_from_context(&self, requirement_id: &str, context_id: &str) -> Result<(), McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "DELETE FROM requirement_context_links WHERE requirement_id = ? AND context_id = ?",
            params![requirement_id, context_id],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(())
    }

    async fn unlink_task_from_context(&self, task_id: &str, context_id: &str) -> Result<(), McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "DELETE FROM task_context_links WHERE task_id = ? AND context_id = ?",
            params![task_id, context_id],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(())
    }

    async fn unlink_task_from_requirement(&self, task_id: &str, requirement_id: &str) -> Result<(), McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "DELETE FROM task_requirement_links WHERE task_id = ? AND requirement_id = ?",
            params![task_id, requirement_id],
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(())
    }
}