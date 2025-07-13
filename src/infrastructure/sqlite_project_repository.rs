use crate::models::context::Project;
use crate::repositories::ProjectRepository;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// SQLite implementation of ProjectRepository
pub struct SqliteProjectRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteProjectRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn create(&self, project: &Project) -> Result<Project, McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT INTO projects (id, name, description, repository_url, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            (
                &project.id,
                &project.name,
                project.description.as_deref(),
                project.repository_url.as_deref(),
                project.created_at.as_deref(),
                project.updated_at.as_deref(),
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(project.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Project>, McpError> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare("SELECT id, name, description, repository_url, created_at, updated_at FROM projects WHERE id = ?")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let mut project_iter = stmt
            .query_map([id], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    repository_url: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        match project_iter.next() {
            Some(Ok(project)) => Ok(Some(project)),
            Some(Err(e)) => Err(McpError::internal_error(
                format!("Database error: {}", e),
                None,
            )),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> Result<Vec<Project>, McpError> {
        let db = self.db.lock().unwrap();
        let mut projects = Vec::new();

        let mut stmt = db.prepare("SELECT id, name, description, repository_url, created_at, updated_at FROM projects")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let project_rows = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    repository_url: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for project in project_rows {
            match project {
                Ok(project) => projects.push(project),
                Err(e) => tracing::warn!("Failed to parse project: {}", e),
            }
        }

        Ok(projects)
    }

    async fn update(&self, project: &Project) -> Result<Project, McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "UPDATE projects SET name = ?, description = ?, repository_url = ?, updated_at = ? WHERE id = ?",
            (
                &project.name,
                project.description.as_deref(),
                project.repository_url.as_deref(),
                project.updated_at.as_deref(),
                &project.id,
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(project.clone())
    }

    async fn delete(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().unwrap();

        let rows_affected = db
            .execute("DELETE FROM projects WHERE id = ?", [id])
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(rows_affected > 0)
    }
}
