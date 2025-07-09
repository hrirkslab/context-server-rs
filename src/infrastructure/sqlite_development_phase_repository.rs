use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use rusqlite::Connection;
use crate::models::flutter::{DevelopmentPhase, PhaseStatus};
use crate::repositories::DevelopmentPhaseRepository;
use rmcp::model::ErrorData as McpError;

/// SQLite implementation of DevelopmentPhaseRepository  
pub struct SqliteDevelopmentPhaseRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteDevelopmentPhaseRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    fn parse_phase_status(status_str: &str) -> PhaseStatus {
        match status_str {
            "pending" => PhaseStatus::Pending,
            "in_progress" => PhaseStatus::InProgress,
            "completed" => PhaseStatus::Completed,
            "blocked" => PhaseStatus::Blocked,
            _ => PhaseStatus::Pending,
        }
    }

    fn phase_status_to_string(status: &PhaseStatus) -> &'static str {
        match status {
            PhaseStatus::Pending => "pending",
            PhaseStatus::InProgress => "in_progress",
            PhaseStatus::Completed => "completed",
            PhaseStatus::Blocked => "blocked",
        }
    }
}

#[async_trait]
impl DevelopmentPhaseRepository for SqliteDevelopmentPhaseRepository {
    async fn create(&self, phase: &DevelopmentPhase) -> Result<DevelopmentPhase, McpError> {
        let db = self.db.lock().unwrap();
        
        let completion_criteria_json = serde_json::to_string(&phase.completion_criteria)
            .map_err(|e| McpError::internal_error(format!("JSON serialization error: {}", e), None))?;
        let dependencies_json = serde_json::to_string(&phase.dependencies)
            .map_err(|e| McpError::internal_error(format!("JSON serialization error: {}", e), None))?;
        
        db.execute(
            "INSERT INTO development_phases (id, project_id, phase_name, phase_order, status, description, completion_criteria, dependencies, started_at, completed_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                &phase.id,
                &phase.project_id,
                &phase.phase_name,
                phase.phase_order,
                Self::phase_status_to_string(&phase.status),
                phase.description.as_deref(),
                &completion_criteria_json,
                &dependencies_json,
                phase.started_at.as_deref(),
                phase.completed_at.as_deref(),
                phase.created_at.as_deref(),
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(phase.clone())
    }

    async fn find_by_project_id(&self, project_id: &str) -> Result<Vec<DevelopmentPhase>, McpError> {
        let db = self.db.lock().unwrap();
        let mut phases = Vec::new();
        
        let mut stmt = db.prepare("SELECT id, project_id, phase_name, phase_order, status, description, completion_criteria, dependencies, started_at, completed_at, created_at FROM development_phases WHERE project_id = ? ORDER BY phase_order")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        
        let phase_rows = stmt.query_map([project_id], |row| {
            let status_str: String = row.get(4)?;
            let completion_criteria_str: String = row.get(6)?;
            let dependencies_str: String = row.get(7)?;
            
            let status = Self::parse_phase_status(&status_str);
            let completion_criteria: Vec<String> = serde_json::from_str(&completion_criteria_str).unwrap_or_default();
            let dependencies: Vec<String> = serde_json::from_str(&dependencies_str).unwrap_or_default();

            Ok(DevelopmentPhase {
                id: row.get(0)?,
                project_id: row.get(1)?,
                phase_name: row.get(2)?,
                phase_order: row.get(3)?,
                status,
                description: row.get(5)?,
                completion_criteria,
                dependencies,
                started_at: row.get(8)?,
                completed_at: row.get(9)?,
                created_at: row.get(10)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for phase in phase_rows {
            match phase {
                Ok(phase) => phases.push(phase),
                Err(e) => tracing::warn!("Failed to parse development phase: {}", e),
            }
        }

        Ok(phases)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<DevelopmentPhase>, McpError> {
        let db = self.db.lock().unwrap();
        
        let mut stmt = db.prepare("SELECT id, project_id, phase_name, phase_order, status, description, completion_criteria, dependencies, started_at, completed_at, created_at FROM development_phases WHERE id = ?")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        
        let mut phase_iter = stmt.query_map([id], |row| {
            let status_str: String = row.get(4)?;
            let completion_criteria_str: String = row.get(6)?;
            let dependencies_str: String = row.get(7)?;
            
            let status = Self::parse_phase_status(&status_str);
            let completion_criteria: Vec<String> = serde_json::from_str(&completion_criteria_str).unwrap_or_default();
            let dependencies: Vec<String> = serde_json::from_str(&dependencies_str).unwrap_or_default();

            Ok(DevelopmentPhase {
                id: row.get(0)?,
                project_id: row.get(1)?,
                phase_name: row.get(2)?,
                phase_order: row.get(3)?,
                status,
                description: row.get(5)?,
                completion_criteria,
                dependencies,
                started_at: row.get(8)?,
                completed_at: row.get(9)?,
                created_at: row.get(10)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        match phase_iter.next() {
            Some(Ok(phase)) => Ok(Some(phase)),
            Some(Err(e)) => Err(McpError::internal_error(format!("Database error: {}", e), None)),
            None => Ok(None),
        }
    }

    async fn update(&self, phase: &DevelopmentPhase) -> Result<DevelopmentPhase, McpError> {
        let db = self.db.lock().unwrap();
        
        let completion_criteria_json = serde_json::to_string(&phase.completion_criteria)
            .map_err(|e| McpError::internal_error(format!("JSON serialization error: {}", e), None))?;
        let dependencies_json = serde_json::to_string(&phase.dependencies)
            .map_err(|e| McpError::internal_error(format!("JSON serialization error: {}", e), None))?;
        
        db.execute(
            "UPDATE development_phases SET project_id = ?, phase_name = ?, phase_order = ?, status = ?, description = ?, completion_criteria = ?, dependencies = ?, started_at = ?, completed_at = ? WHERE id = ?",
            (
                &phase.project_id,
                &phase.phase_name,
                phase.phase_order,
                Self::phase_status_to_string(&phase.status),
                phase.description.as_deref(),
                &completion_criteria_json,
                &dependencies_json,
                phase.started_at.as_deref(),
                phase.completed_at.as_deref(),
                &phase.id,
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(phase.clone())
    }

    async fn delete(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().unwrap();
        
        let rows_affected = db.execute("DELETE FROM development_phases WHERE id = ?", [id])
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(rows_affected > 0)
    }
}
