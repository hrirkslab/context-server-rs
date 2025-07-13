use async_trait::async_trait;
use rusqlite::{Connection, Result, params};
use std::sync::{Arc, Mutex};
use crate::models::framework::FrameworkComponent;
use crate::repositories::ComponentRepository;
use rmcp::model::ErrorData as McpError;

/// SQLite implementation of ComponentRepository
#[derive(Debug)]
pub struct SqliteComponentRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteComponentRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl ComponentRepository for SqliteComponentRepository {
    async fn create(&self, component: &FrameworkComponent) -> Result<FrameworkComponent, McpError> {
        let conn = self.conn.lock().map_err(|e| 
            McpError::internal_error(format!("Failed to acquire database lock: {}", e), None)
        )?;

        let metadata = match &component.metadata {
            Some(m) => serde_json::to_string(m)
                .map_err(|e| McpError::internal_error(format!("Failed to serialize metadata: {}", e), None))?,
            None => "{}".to_string(),
        };

        // Serialize dependencies as JSON
        let dependencies = serde_json::to_string(&component.dependencies)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize dependencies: {}", e), None))?;

        // Insert the component into the database
        conn.execute(
            "INSERT INTO framework_components (
                id, project_id, component_name, component_type, architecture_layer, 
                file_path, dependencies, metadata, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                component.id,
                component.project_id,
                component.component_name,
                component.component_type,
                component.architecture_layer,
                component.file_path,
                dependencies,
                metadata,
                component.created_at,
                component.updated_at,
            ],
        )
        .map_err(|e| McpError::internal_error(format!("Failed to insert component: {}", e), None))?;

        // Return the created component
        Ok(component.clone())
    }

    async fn find_by_project_id(&self, project_id: &str) -> Result<Vec<FrameworkComponent>, McpError> {
        let conn = self.conn.lock().map_err(|e| 
            McpError::internal_error(format!("Failed to acquire database lock: {}", e), None)
        )?;
        
        let mut stmt = conn.prepare(
            "SELECT 
                id, project_id, component_name, component_type, architecture_layer,
                file_path, dependencies, metadata, created_at, updated_at
             FROM framework_components 
             WHERE project_id = ?",
        )
        .map_err(|e| McpError::internal_error(format!("Failed to prepare statement: {}", e), None))?;

        let rows = stmt.query_map(params![project_id], |row| {
            let dependencies_str: String = row.get(6)?;
            let dependencies: Vec<String> = serde_json::from_str(&dependencies_str)
                .unwrap_or_else(|_| Vec::new());

            let metadata_str: String = row.get(7)?;
            let metadata = serde_json::from_str(&metadata_str).ok();

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
        .map_err(|e| McpError::internal_error(format!("Failed to query components: {}", e), None))?;

        let mut components = Vec::new();
        for row in rows {
            components.push(row.map_err(|e| McpError::internal_error(format!("Failed to get row: {}", e), None))?);
        }

        Ok(components)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<FrameworkComponent>, McpError> {
        let conn = self.conn.lock().map_err(|e| 
            McpError::internal_error(format!("Failed to acquire database lock: {}", e), None)
        )?;
        
        let mut stmt = conn.prepare(
            "SELECT 
                id, project_id, component_name, component_type, architecture_layer,
                file_path, dependencies, metadata, created_at, updated_at
             FROM framework_components 
             WHERE id = ?",
        )
        .map_err(|e| McpError::internal_error(format!("Failed to prepare statement: {}", e), None))?;

        let mut rows = stmt.query_map(params![id], |row| {
            let dependencies_str: String = row.get(6)?;
            let dependencies: Vec<String> = serde_json::from_str(&dependencies_str)
                .unwrap_or_else(|_| Vec::new());

            let metadata_str: String = row.get(7)?;
            let metadata = serde_json::from_str(&metadata_str).ok();

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
        .map_err(|e| McpError::internal_error(format!("Failed to query component: {}", e), None))?;

        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| McpError::internal_error(format!("Failed to get row: {}", e), None))?)),
            None => Ok(None),
        }
    }

    async fn update(&self, component: &FrameworkComponent) -> Result<FrameworkComponent, McpError> {
        let conn = self.conn.lock().map_err(|e| 
            McpError::internal_error(format!("Failed to acquire database lock: {}", e), None)
        )?;

        let metadata = match &component.metadata {
            Some(m) => serde_json::to_string(m)
                .map_err(|e| McpError::internal_error(format!("Failed to serialize metadata: {}", e), None))?,
            None => "{}".to_string(),
        };

        // Serialize dependencies as JSON
        let dependencies = serde_json::to_string(&component.dependencies)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize dependencies: {}", e), None))?;

        // Update the component in the database
        conn.execute(
            "UPDATE framework_components 
             SET project_id = ?, component_name = ?, component_type = ?, architecture_layer = ?,
                 file_path = ?, dependencies = ?, metadata = ?, updated_at = ?
             WHERE id = ?",
            params![
                component.project_id,
                component.component_name,
                component.component_type,
                component.architecture_layer,
                component.file_path,
                dependencies,
                metadata,
                component.updated_at,
                component.id,
            ],
        )
        .map_err(|e| McpError::internal_error(format!("Failed to update component: {}", e), None))?;

        // Return the updated component
        Ok(component.clone())
    }

    async fn delete(&self, id: &str) -> Result<bool, McpError> {
        let conn = self.conn.lock().map_err(|e| 
            McpError::internal_error(format!("Failed to acquire database lock: {}", e), None)
        )?;

        // Delete the component from the database
        let rows_affected = conn.execute("DELETE FROM framework_components WHERE id = ?", params![id])
            .map_err(|e| McpError::internal_error(format!("Failed to delete component: {}", e), None))?;

        // Return true if a component was deleted, false otherwise
        Ok(rows_affected > 0)
    }

    async fn find_by_architecture_layer(&self, project_id: &str, layer: &str) -> Result<Vec<FrameworkComponent>, McpError> {
        let conn = self.conn.lock().map_err(|e| 
            McpError::internal_error(format!("Failed to acquire database lock: {}", e), None)
        )?;
        
        let mut stmt = conn.prepare(
            "SELECT 
                id, project_id, component_name, component_type, architecture_layer,
                file_path, dependencies, metadata, created_at, updated_at
             FROM framework_components 
             WHERE project_id = ? AND architecture_layer = ?",
        )
        .map_err(|e| McpError::internal_error(format!("Failed to prepare statement: {}", e), None))?;

        let rows = stmt.query_map(params![project_id, layer], |row| {
            let dependencies_str: String = row.get(6)?;
            let dependencies: Vec<String> = serde_json::from_str(&dependencies_str)
                .unwrap_or_else(|_| Vec::new());

            let metadata_str: String = row.get(7)?;
            let metadata = serde_json::from_str(&metadata_str).ok();

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
        .map_err(|e| McpError::internal_error(format!("Failed to query components: {}", e), None))?;

        let mut components = Vec::new();
        for row in rows {
            components.push(row.map_err(|e| McpError::internal_error(format!("Failed to get row: {}", e), None))?);
        }

        Ok(components)
    }
}
