use async_trait::async_trait;
use crate::models::context::SecurityPolicy;
use crate::repositories::SecurityPolicyRepository;
use rmcp::model::ErrorData as McpError;
use rusqlite::{Connection, Result as SqliteResult, params};
use std::sync::{Arc, Mutex};

pub struct SqliteSecurityPolicyRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteSecurityPolicyRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SecurityPolicyRepository for SqliteSecurityPolicyRepository {
    async fn create(&self, security_policy: &SecurityPolicy) -> Result<SecurityPolicy, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        db.execute(
            "INSERT INTO security_policies (id, project_id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                security_policy.id,
                security_policy.project_id,
                security_policy.policy_name,
                security_policy.policy_area,
                security_policy.requirements,
                security_policy.implementation_pattern,
                security_policy.forbidden_patterns,
                security_policy.compliance_notes,
                security_policy.created_at
            ],
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to create security policy: {}", e),
        })?;

        Ok(security_policy.clone())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<SecurityPolicy>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at 
             FROM security_policies WHERE id = ?1"
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to prepare statement: {}", e),
        })?;

        let security_policy_result = stmt.query_row(params![id], |row| {
            Ok(SecurityPolicy {
                id: row.get(0)?,
                project_id: row.get(1)?,
                policy_name: row.get(2)?,
                policy_area: row.get(3)?,
                requirements: row.get(4)?,
                implementation_pattern: row.get(5)?,
                forbidden_patterns: row.get(6)?,
                compliance_notes: row.get(7)?,
                created_at: row.get(8)?,
            })
        });

        match security_policy_result {
            Ok(security_policy) => Ok(Some(security_policy)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(McpError {
                code: -1,
                message: format!("Failed to get security policy: {}", e),
            }),
        }
    }

    async fn update(&self, security_policy: &SecurityPolicy) -> Result<SecurityPolicy, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        db.execute(
            "UPDATE security_policies SET project_id = ?2, policy_name = ?3, policy_area = ?4, requirements = ?5, implementation_pattern = ?6, forbidden_patterns = ?7, compliance_notes = ?8, created_at = ?9 WHERE id = ?1",
            params![
                security_policy.id,
                security_policy.project_id,
                security_policy.policy_name,
                security_policy.policy_area,
                security_policy.requirements,
                security_policy.implementation_pattern,
                security_policy.forbidden_patterns,
                security_policy.compliance_notes,
                security_policy.created_at
            ],
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to update security policy: {}", e),
        })?;

        Ok(security_policy.clone())
    }

    async fn delete(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let rows_affected = db.execute("DELETE FROM security_policies WHERE id = ?1", params![id])
            .map_err(|e| McpError {
                code: -1,
                message: format!("Failed to delete security policy: {}", e),
            })?;

        Ok(rows_affected > 0)
    }

    async fn list_by_project(&self, project_id: &str) -> Result<Vec<SecurityPolicy>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at 
             FROM security_policies WHERE project_id = ?1 ORDER BY created_at DESC"
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to prepare statement: {}", e),
        })?;

        let security_policy_iter = stmt.query_map(params![project_id], |row| {
            Ok(SecurityPolicy {
                id: row.get(0)?,
                project_id: row.get(1)?,
                policy_name: row.get(2)?,
                policy_area: row.get(3)?,
                requirements: row.get(4)?,
                implementation_pattern: row.get(5)?,
                forbidden_patterns: row.get(6)?,
                compliance_notes: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to query security policies: {}", e),
        })?;

        let mut security_policies = Vec::new();
        for security_policy in security_policy_iter {
            security_policies.push(security_policy.map_err(|e| McpError {
                code: -1,
                message: format!("Failed to process security policy row: {}", e),
            })?);
        }

        Ok(security_policies)
    }

    async fn list_by_policy_area(&self, project_id: &str, policy_area: &str) -> Result<Vec<SecurityPolicy>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at 
             FROM security_policies WHERE project_id = ?1 AND policy_area = ?2 ORDER BY created_at DESC"
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to prepare statement: {}", e),
        })?;

        let security_policy_iter = stmt.query_map(params![project_id, policy_area], |row| {
            Ok(SecurityPolicy {
                id: row.get(0)?,
                project_id: row.get(1)?,
                policy_name: row.get(2)?,
                policy_area: row.get(3)?,
                requirements: row.get(4)?,
                implementation_pattern: row.get(5)?,
                forbidden_patterns: row.get(6)?,
                compliance_notes: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to query security policies: {}", e),
        })?;

        let mut security_policies = Vec::new();
        for security_policy in security_policy_iter {
            security_policies.push(security_policy.map_err(|e| McpError {
                code: -1,
                message: format!("Failed to process security policy row: {}", e),
            })?);
        }

        Ok(security_policies)
    }

    async fn bulk_create(&self, security_policies: &[SecurityPolicy]) -> Result<Vec<SecurityPolicy>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let tx = db.unchecked_transaction().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to start transaction: {}", e),
        })?;

        for security_policy in security_policies {
            tx.execute(
                "INSERT INTO security_policies (id, project_id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    security_policy.id,
                    security_policy.project_id,
                    security_policy.policy_name,
                    security_policy.policy_area,
                    security_policy.requirements,
                    security_policy.implementation_pattern,
                    security_policy.forbidden_patterns,
                    security_policy.compliance_notes,
                    security_policy.created_at
                ],
            ).map_err(|e| McpError {
                code: -1,
                message: format!("Failed to insert security policy: {}", e),
            })?;
        }

        tx.commit().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(security_policies.to_vec())
    }

    async fn bulk_update(&self, security_policies: &[SecurityPolicy]) -> Result<Vec<SecurityPolicy>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let tx = db.unchecked_transaction().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to start transaction: {}", e),
        })?;

        for security_policy in security_policies {
            tx.execute(
                "UPDATE security_policies SET project_id = ?2, policy_name = ?3, policy_area = ?4, requirements = ?5, implementation_pattern = ?6, forbidden_patterns = ?7, compliance_notes = ?8, created_at = ?9 WHERE id = ?1",
                params![
                    security_policy.id,
                    security_policy.project_id,
                    security_policy.policy_name,
                    security_policy.policy_area,
                    security_policy.requirements,
                    security_policy.implementation_pattern,
                    security_policy.forbidden_patterns,
                    security_policy.compliance_notes,
                    security_policy.created_at
                ],
            ).map_err(|e| McpError {
                code: -1,
                message: format!("Failed to update security policy: {}", e),
            })?;
        }

        tx.commit().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(security_policies.to_vec())
    }

    async fn bulk_delete(&self, ids: &[String]) -> Result<usize, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let tx = db.unchecked_transaction().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to start transaction: {}", e),
        })?;

        let mut total_deleted = 0;
        for id in ids {
            let rows_affected = tx.execute("DELETE FROM security_policies WHERE id = ?1", params![id])
                .map_err(|e| McpError {
                    code: -1,
                    message: format!("Failed to delete security policy: {}", e),
                })?;
            total_deleted += rows_affected;
        }

        tx.commit().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(total_deleted)
    }
}
