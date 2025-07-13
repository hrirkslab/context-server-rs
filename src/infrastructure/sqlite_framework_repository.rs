use crate::models::framework::FrameworkComponent;
use crate::repositories::FrameworkRepository;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// SQLite implementation of FrameworkRepository
pub struct SqliteFrameworkRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteFrameworkRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FrameworkRepository for SqliteFrameworkRepository {
    async fn create(&self, component: &FrameworkComponent) -> Result<FrameworkComponent, McpError> {
        let db = self
            .db
            .lock()
            .map_err(|e| McpError::internal_error(format!("Database lock error: {}", e), None))?;

        let metadata_json = component
            .metadata
            .as_ref()
            .map(|m| {
                serde_json::to_string(m).map_err(|e| {
                    McpError::internal_error(format!("Failed to serialize metadata: {}", e), None)
                })
            })
            .transpose()?
            .unwrap_or_default();

        let dependencies_json = serde_json::to_string(&component.dependencies).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize dependencies: {}", e), None)
        })?;

        db.execute(
            "INSERT INTO framework_components (id, project_id, component_name, component_type, architecture_layer, file_path, dependencies, metadata, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (
                &component.id,
                &component.project_id,
                &component.component_name,
                &component.component_type,
                &component.architecture_layer,
                &component.file_path,
                &dependencies_json,
                &metadata_json,
                &component.created_at,
                &component.updated_at,
            ),
        ).map_err(|e|
            McpError::internal_error(format!("Failed to create framework component: {}", e), None)
        )?;

        Ok(component.clone())
    }
    async fn find_by_project_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<FrameworkComponent>, McpError> {
        let db = self
            .db
            .lock()
            .map_err(|e| McpError::internal_error(format!("Database lock error: {}", e), None))?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, component_name, component_type, architecture_layer, file_path, dependencies, metadata, created_at, updated_at 
             FROM framework_components WHERE project_id = ?1"
        ).map_err(|e|
            McpError::internal_error(format!("Failed to prepare statement: {}", e), None)
        )?;

        let component_iter = stmt
            .query_map([project_id], |row| {
                let dependencies_str: String = row.get(6)?;
                let dependencies: Vec<String> =
                    serde_json::from_str(&dependencies_str).unwrap_or_default();

                let metadata_str: String = row.get(7)?;
                let metadata: Option<serde_json::Value> = if metadata_str.is_empty() {
                    None
                } else {
                    serde_json::from_str(&metadata_str).ok()
                };

                Ok(FrameworkComponent {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    component_name: row.get(2)?,
                    component_type: row.get(3)?,
                    architecture_layer: row.get(4)?,
                    file_path: row.get(5)?,
                    dependencies,
                    metadata,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| {
                McpError::internal_error(
                    format!("Failed to query framework components: {}", e),
                    None,
                )
            })?;

        let mut components = Vec::new();
        for component in component_iter {
            components.push(component.map_err(|e| {
                McpError::internal_error(
                    format!("Failed to parse framework component: {}", e),
                    None,
                )
            })?);
        }

        Ok(components)
    }
    async fn find_by_id(&self, id: &str) -> Result<Option<FrameworkComponent>, McpError> {
        let db = self
            .db
            .lock()
            .map_err(|e| McpError::internal_error(format!("Database lock error: {}", e), None))?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, component_name, component_type, architecture_layer, file_path, dependencies, metadata, created_at, updated_at 
             FROM framework_components WHERE id = ?1"
        ).map_err(|e|
            McpError::internal_error(format!("Failed to prepare statement: {}", e), None)
        )?;

        let mut component_iter = stmt
            .query_map([id], |row| {
                let dependencies_str: String = row.get(6)?;
                let dependencies: Vec<String> =
                    serde_json::from_str(&dependencies_str).unwrap_or_default();

                let metadata_str: String = row.get(7)?;
                let metadata: Option<serde_json::Value> = if metadata_str.is_empty() {
                    None
                } else {
                    serde_json::from_str(&metadata_str).ok()
                };

                Ok(FrameworkComponent {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    component_name: row.get(2)?,
                    component_type: row.get(3)?,
                    architecture_layer: row.get(4)?,
                    file_path: row.get(5)?,
                    dependencies,
                    metadata,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| {
                McpError::internal_error(
                    format!("Failed to query framework component: {}", e),
                    None,
                )
            })?;

        match component_iter.next() {
            Some(component) => Ok(Some(component.map_err(|e| {
                McpError::internal_error(
                    format!("Failed to parse framework component: {}", e),
                    None,
                )
            })?)),
            None => Ok(None),
        }
    }
    async fn update(&self, component: &FrameworkComponent) -> Result<FrameworkComponent, McpError> {
        let db = self
            .db
            .lock()
            .map_err(|e| McpError::internal_error(format!("Database lock error: {}", e), None))?;

        let metadata_json = component
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default())
            .unwrap_or_default();

        let dependencies_json =
            serde_json::to_string(&component.dependencies).unwrap_or_else(|_| "[]".to_string());

        db.execute(
            "UPDATE framework_components SET project_id = ?2, component_name = ?3, component_type = ?4, architecture_layer = ?5, file_path = ?6, dependencies = ?7, metadata = ?8, updated_at = ?9 WHERE id = ?1",
            (
                &component.id,
                &component.project_id,
                &component.component_name,
                &component.component_type,
                &component.architecture_layer,
                &component.file_path,
                &dependencies_json,
                &metadata_json,
                &component.updated_at,
            ),
        ).map_err(|e|
            McpError::internal_error(format!("Failed to update framework component: {}", e), None)
        )?;

        Ok(component.clone())
    }
    async fn delete(&self, id: &str) -> Result<bool, McpError> {
        let db = self
            .db
            .lock()
            .map_err(|e| McpError::internal_error(format!("Database lock error: {}", e), None))?;

        let rows_affected = db
            .execute("DELETE FROM framework_components WHERE id = ?1", [id])
            .map_err(|e| {
                McpError::internal_error(
                    format!("Failed to delete framework component: {}", e),
                    None,
                )
            })?;

        Ok(rows_affected > 0)
    }
    async fn find_by_architecture_layer(
        &self,
        project_id: &str,
        layer: &str,
    ) -> Result<Vec<FrameworkComponent>, McpError> {
        let db = self
            .db
            .lock()
            .map_err(|e| McpError::internal_error(format!("Database lock error: {}", e), None))?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, component_name, component_type, architecture_layer, file_path, dependencies, metadata, created_at, updated_at 
             FROM framework_components WHERE project_id = ?1 AND architecture_layer = ?2"
        ).map_err(|e|
            McpError::internal_error(format!("Failed to prepare statement: {}", e), None)
        )?;

        let component_iter = stmt
            .query_map([project_id, layer], |row| {
                let dependencies_str: String = row.get(6)?;
                let dependencies: Vec<String> =
                    serde_json::from_str(&dependencies_str).unwrap_or_default();

                let metadata_str: String = row.get(7)?;
                let metadata: Option<serde_json::Value> = if metadata_str.is_empty() {
                    None
                } else {
                    serde_json::from_str(&metadata_str).ok()
                };

                Ok(FrameworkComponent {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    component_name: row.get(2)?,
                    component_type: row.get(3)?,
                    architecture_layer: row.get(4)?,
                    file_path: row.get(5)?,
                    dependencies,
                    metadata,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| {
                McpError::internal_error(
                    format!("Failed to query framework components by layer: {}", e),
                    None,
                )
            })?;

        let mut components = Vec::new();
        for component in component_iter {
            components.push(component.map_err(|e| {
                McpError::internal_error(
                    format!("Failed to parse framework component: {}", e),
                    None,
                )
            })?);
        }

        Ok(components)
    }
}
