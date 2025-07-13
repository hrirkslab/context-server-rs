use crate::models::context::ArchitecturalDecision;
use crate::repositories::ArchitecturalDecisionRepository;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// SQLite implementation of ArchitecturalDecisionRepository
pub struct SqliteArchitecturalDecisionRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteArchitecturalDecisionRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ArchitecturalDecisionRepository for SqliteArchitecturalDecisionRepository {
    async fn create(
        &self,
        decision: &ArchitecturalDecision,
    ) -> Result<ArchitecturalDecision, McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT INTO architectural_decisions (id, project_id, decision_title, context, decision, consequences, alternatives_considered, status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                &decision.id,
                &decision.project_id,
                &decision.decision_title,
                decision.context.as_deref(),
                decision.decision.as_deref(),
                decision.consequences.as_deref(),
                decision.alternatives_considered.as_deref(),
                decision.status.as_deref(),
                decision.created_at.as_deref(),
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(decision.clone())
    }

    async fn find_by_project_id(
        &self,
        project_id: &str,
    ) -> Result<Vec<ArchitecturalDecision>, McpError> {
        let db = self.db.lock().unwrap();
        let mut decisions = Vec::new();

        let mut stmt = db.prepare("SELECT id, project_id, decision_title, context, decision, consequences, alternatives_considered, status, created_at FROM architectural_decisions WHERE project_id = ?")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let decision_rows = stmt
            .query_map([project_id], |row| {
                Ok(ArchitecturalDecision {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    decision_title: row.get(2)?,
                    context: row.get(3)?,
                    decision: row.get(4)?,
                    consequences: row.get(5)?,
                    alternatives_considered: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for decision in decision_rows {
            match decision {
                Ok(decision) => decisions.push(decision),
                Err(e) => tracing::warn!("Failed to parse architectural decision: {}", e),
            }
        }

        Ok(decisions)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<ArchitecturalDecision>, McpError> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare("SELECT id, project_id, decision_title, context, decision, consequences, alternatives_considered, status, created_at FROM architectural_decisions WHERE id = ?")
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        let mut decision_iter = stmt
            .query_map([id], |row| {
                Ok(ArchitecturalDecision {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    decision_title: row.get(2)?,
                    context: row.get(3)?,
                    decision: row.get(4)?,
                    consequences: row.get(5)?,
                    alternatives_considered: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        match decision_iter.next() {
            Some(Ok(decision)) => Ok(Some(decision)),
            Some(Err(e)) => Err(McpError::internal_error(
                format!("Database error: {}", e),
                None,
            )),
            None => Ok(None),
        }
    }

    async fn update(
        &self,
        decision: &ArchitecturalDecision,
    ) -> Result<ArchitecturalDecision, McpError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "UPDATE architectural_decisions SET project_id = ?, decision_title = ?, context = ?, decision = ?, consequences = ?, alternatives_considered = ?, status = ? WHERE id = ?",
            (
                &decision.project_id,
                &decision.decision_title,
                decision.context.as_deref(),
                decision.decision.as_deref(),
                decision.consequences.as_deref(),
                decision.alternatives_considered.as_deref(),
                decision.status.as_deref(),
                &decision.id,
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(decision.clone())
    }

    async fn delete(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().unwrap();

        let rows_affected = db
            .execute("DELETE FROM architectural_decisions WHERE id = ?", [id])
            .map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(rows_affected > 0)
    }
}
