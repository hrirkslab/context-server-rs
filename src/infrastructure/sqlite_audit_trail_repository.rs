/// SQLite repository for audit trails
use crate::models::audit_log::{AuditEventType, AuditTrail};
use std::sync::Arc;

pub trait AuditTrailRepository: Send {
    fn log_event(&self, audit: &AuditTrail) -> anyhow::Result<()>;
    fn get_audit_trail(&self, id: &str) -> anyhow::Result<Option<AuditTrail>>;
    fn get_entity_history(&self, entity_type: &str, entity_id: &str) -> anyhow::Result<Vec<AuditTrail>>;
    fn get_project_audit_log(&self, project_id: &str, limit: i64) -> anyhow::Result<Vec<AuditTrail>>;
    fn get_initiator_actions(&self, initiator: &str, limit: i64) -> anyhow::Result<Vec<AuditTrail>>;
}

pub struct SqliteAuditTrailRepository {
    conn: Arc<rusqlite::Connection>,
}

impl SqliteAuditTrailRepository {
    pub fn new(conn: Arc<rusqlite::Connection>) -> Self {
        Self { conn }
    }

    pub fn init_table(&self) -> anyhow::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_trails (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                initiator TEXT NOT NULL,
                previous_state TEXT,
                new_state TEXT,
                change_summary TEXT NOT NULL,
                project_id TEXT,
                metadata TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            )",
            [],
        )?;

        // Create index for faster queries
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_trails(entity_type, entity_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_project ON audit_trails(project_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_initiator ON audit_trails(initiator)",
            [],
        );

        Ok(())
    }
}

impl AuditTrailRepository for SqliteAuditTrailRepository {
    fn log_event(&self, audit: &AuditTrail) -> anyhow::Result<()> {
        let previous_state = audit.previous_state.as_ref().map(|v| v.to_string());
        let new_state = audit.new_state.as_ref().map(|v| v.to_string());
        let metadata = audit.metadata.as_ref().map(|v| v.to_string());

        self.conn.execute(
            "INSERT INTO audit_trails (id, timestamp, event_type, entity_type, entity_id, initiator, 
             previous_state, new_state, change_summary, project_id, metadata)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                audit.id,
                audit.timestamp.to_rfc3339(),
                audit.event_type.as_str(),
                audit.entity_type,
                audit.entity_id,
                audit.initiator,
                previous_state,
                new_state,
                audit.change_summary,
                audit.project_id,
                metadata,
            ],
        )?;

        Ok(())
    }

    fn get_audit_trail(&self, id: &str) -> anyhow::Result<Option<AuditTrail>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, event_type, entity_type, entity_id, initiator, 
             previous_state, new_state, change_summary, project_id, metadata 
             FROM audit_trails WHERE id = ? LIMIT 1",
        )?;

        let result = stmt.query_row(rusqlite::params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        });

        match result {
            Ok((id, timestamp, event_type, entity_type, entity_id, initiator, previous_state, new_state, change_summary, project_id, metadata)) => {
                let event_type = match event_type.as_str() {
                    "created" => AuditEventType::Created,
                    "updated" => AuditEventType::Updated,
                    "deleted" => AuditEventType::Deleted,
                    "query_executed" => AuditEventType::QueryExecuted,
                    "constraint_applied" => AuditEventType::ConstraintApplied,
                    _ => AuditEventType::Created,
                };

                let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                let audit = AuditTrail {
                    id,
                    timestamp,
                    event_type,
                    entity_type,
                    entity_id,
                    initiator,
                    previous_state: previous_state.and_then(|s| serde_json::from_str(&s).ok()),
                    new_state: new_state.and_then(|s| serde_json::from_str(&s).ok()),
                    change_summary,
                    project_id,
                    metadata: metadata.and_then(|s| serde_json::from_str(&s).ok()),
                };

                Ok(Some(audit))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn get_entity_history(&self, entity_type: &str, entity_id: &str) -> anyhow::Result<Vec<AuditTrail>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, event_type, entity_type, entity_id, initiator, 
             previous_state, new_state, change_summary, project_id, metadata 
             FROM audit_trails 
             WHERE entity_type = ? AND entity_id = ?
             ORDER BY timestamp DESC",
        )?;

        let audits = stmt.query_map(rusqlite::params![entity_type, entity_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        })?;

        let mut result = Vec::new();
        for audit_row in audits {
            let (id, timestamp, event_type, entity_type, entity_id, initiator, previous_state, new_state, change_summary, project_id, metadata) = audit_row?;

            let event_type = match event_type.as_str() {
                "created" => AuditEventType::Created,
                "updated" => AuditEventType::Updated,
                "deleted" => AuditEventType::Deleted,
                "query_executed" => AuditEventType::QueryExecuted,
                "constraint_applied" => AuditEventType::ConstraintApplied,
                _ => AuditEventType::Created,
            };

            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            result.push(AuditTrail {
                id,
                timestamp,
                event_type,
                entity_type,
                entity_id,
                initiator,
                previous_state: previous_state.and_then(|s| serde_json::from_str(&s).ok()),
                new_state: new_state.and_then(|s| serde_json::from_str(&s).ok()),
                change_summary,
                project_id,
                metadata: metadata.and_then(|s| serde_json::from_str(&s).ok()),
            });
        }

        Ok(result)
    }

    fn get_project_audit_log(&self, project_id: &str, limit: i64) -> anyhow::Result<Vec<AuditTrail>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, event_type, entity_type, entity_id, initiator, 
             previous_state, new_state, change_summary, project_id, metadata 
             FROM audit_trails 
             WHERE project_id = ?
             ORDER BY timestamp DESC
             LIMIT ?",
        )?;

        let audits = stmt.query_map(rusqlite::params![project_id, limit], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        })?;

        let mut result = Vec::new();
        for audit_row in audits {
            let (id, timestamp, event_type, entity_type, entity_id, initiator, previous_state, new_state, change_summary, project_id, metadata) = audit_row?;

            let event_type = match event_type.as_str() {
                "created" => AuditEventType::Created,
                "updated" => AuditEventType::Updated,
                "deleted" => AuditEventType::Deleted,
                "query_executed" => AuditEventType::QueryExecuted,
                "constraint_applied" => AuditEventType::ConstraintApplied,
                _ => AuditEventType::Created,
            };

            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            result.push(AuditTrail {
                id,
                timestamp,
                event_type,
                entity_type,
                initiator,
                previous_state: previous_state.and_then(|s| serde_json::from_str(&s).ok()),
                new_state: new_state.and_then(|s| serde_json::from_str(&s).ok()),
                change_summary,
                project_id,
                metadata: metadata.and_then(|s| serde_json::from_str(&s).ok()),
                entity_id,
            });
        }

        Ok(result)
    }

    fn get_initiator_actions(&self, initiator: &str, limit: i64) -> anyhow::Result<Vec<AuditTrail>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, event_type, entity_type, entity_id, initiator, 
             previous_state, new_state, change_summary, project_id, metadata 
             FROM audit_trails 
             WHERE initiator = ?
             ORDER BY timestamp DESC
             LIMIT ?",
        )?;

        let audits = stmt.query_map(rusqlite::params![initiator, limit], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        })?;

        let mut result = Vec::new();
        for audit_row in audits {
            let (id, timestamp, event_type, entity_type, entity_id, initiator_val, previous_state, new_state, change_summary, project_id, metadata) = audit_row?;

            let event_type = match event_type.as_str() {
                "created" => AuditEventType::Created,
                "updated" => AuditEventType::Updated,
                "deleted" => AuditEventType::Deleted,
                "query_executed" => AuditEventType::QueryExecuted,
                "constraint_applied" => AuditEventType::ConstraintApplied,
                _ => AuditEventType::Created,
            };

            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            result.push(AuditTrail {
                id,
                timestamp,
                event_type,
                entity_type,
                entity_id,
                initiator: initiator_val,
                previous_state: previous_state.and_then(|s| serde_json::from_str(&s).ok()),
                new_state: new_state.and_then(|s| serde_json::from_str(&s).ok()),
                change_summary,
                project_id,
                metadata: metadata.and_then(|s| serde_json::from_str(&s).ok()),
            });
        }

        Ok(result)
    }
}
