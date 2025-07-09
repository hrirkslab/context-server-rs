use async_trait::async_trait;
use crate::models::context::{ProjectConvention, FeatureContext};
use crate::models::flutter::{PrivacyRule, PrivacyViolation, ArchitectureLayerConfig, ModelContext, CodeTemplate, PrivacyRuleType, Severity, ViolationStatus, TemplateType};
use crate::repositories::{
    ProjectConventionRepository, FeatureContextRepository, PrivacyRuleRepository, 
    PrivacyViolationRepository, ArchitectureLayerRepository, ModelContextRepository, 
    CodeTemplateRepository
};
use rmcp::model::ErrorData as McpError;
use rusqlite::{Connection, Result as SqliteResult, params};
use std::sync::{Arc, Mutex};

// Project Convention Repository
pub struct SqliteProjectConventionRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteProjectConventionRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ProjectConventionRepository for SqliteProjectConventionRepository {
    async fn create(&self, convention: &ProjectConvention) -> Result<ProjectConvention, McpError> {
        let db = self.db.lock().map_err(|e| McpError::internal_error(format!("Database lock error: {}", e), None))?;

        db.execute(
            "INSERT INTO project_conventions (id, project_id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                convention.id,
                convention.project_id,
                convention.convention_type,
                convention.convention_rule,
                convention.good_examples,
                convention.bad_examples,
                convention.rationale,
                convention.created_at
            ],
        ).map_err(|e| McpError::internal_error(format!("Failed to create project convention: {}", e), None))?;

        Ok(convention.clone())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<ProjectConvention>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at 
             FROM project_conventions WHERE id = ?1"
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to prepare statement: {}", e),
        })?;

        let convention_result = stmt.query_row(params![id], |row| {
            Ok(ProjectConvention {
                id: row.get(0)?,
                project_id: row.get(1)?,
                convention_type: row.get(2)?,
                convention_rule: row.get(3)?,
                good_examples: row.get(4)?,
                bad_examples: row.get(5)?,
                rationale: row.get(6)?,
                created_at: row.get(7)?,
            })
        });

        match convention_result {
            Ok(convention) => Ok(Some(convention)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(McpError {
                code: -1,
                message: format!("Failed to get project convention: {}", e),
            }),
        }
    }

    async fn update(&self, convention: &ProjectConvention) -> Result<ProjectConvention, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        db.execute(
            "UPDATE project_conventions SET project_id = ?2, convention_type = ?3, convention_rule = ?4, good_examples = ?5, bad_examples = ?6, rationale = ?7, created_at = ?8 WHERE id = ?1",
            params![
                convention.id,
                convention.project_id,
                convention.convention_type,
                convention.convention_rule,
                convention.good_examples,
                convention.bad_examples,
                convention.rationale,
                convention.created_at
            ],
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to update project convention: {}", e),
        })?;

        Ok(convention.clone())
    }

    async fn delete(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let rows_affected = db.execute("DELETE FROM project_conventions WHERE id = ?1", params![id])
            .map_err(|e| McpError {
                code: -1,
                message: format!("Failed to delete project convention: {}", e),
            })?;

        Ok(rows_affected > 0)
    }

    async fn list_by_project(&self, project_id: &str) -> Result<Vec<ProjectConvention>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at 
             FROM project_conventions WHERE project_id = ?1 ORDER BY created_at DESC"
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to prepare statement: {}", e),
        })?;

        let convention_iter = stmt.query_map(params![project_id], |row| {
            Ok(ProjectConvention {
                id: row.get(0)?,
                project_id: row.get(1)?,
                convention_type: row.get(2)?,
                convention_rule: row.get(3)?,
                good_examples: row.get(4)?,
                bad_examples: row.get(5)?,
                rationale: row.get(6)?,
                created_at: row.get(7)?,
            })
        }).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to query project conventions: {}", e),
        })?;

        let mut conventions = Vec::new();
        for convention in convention_iter {
            conventions.push(convention.map_err(|e| McpError {
                code: -1,
                message: format!("Failed to process project convention row: {}", e),
            })?);
        }

        Ok(conventions)
    }

    async fn list_by_convention_type(&self, project_id: &str, convention_type: &str) -> Result<Vec<ProjectConvention>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at 
             FROM project_conventions WHERE project_id = ?1 AND convention_type = ?2 ORDER BY created_at DESC"
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to prepare statement: {}", e),
        })?;

        let convention_iter = stmt.query_map(params![project_id, convention_type], |row| {
            Ok(ProjectConvention {
                id: row.get(0)?,
                project_id: row.get(1)?,
                convention_type: row.get(2)?,
                convention_rule: row.get(3)?,
                good_examples: row.get(4)?,
                bad_examples: row.get(5)?,
                rationale: row.get(6)?,
                created_at: row.get(7)?,
            })
        }).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to query project conventions: {}", e),
        })?;

        let mut conventions = Vec::new();
        for convention in convention_iter {
            conventions.push(convention.map_err(|e| McpError {
                code: -1,
                message: format!("Failed to process project convention row: {}", e),
            })?);
        }

        Ok(conventions)
    }

    async fn bulk_create(&self, conventions: &[ProjectConvention]) -> Result<Vec<ProjectConvention>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let tx = db.unchecked_transaction().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to start transaction: {}", e),
        })?;

        for convention in conventions {
            tx.execute(
                "INSERT INTO project_conventions (id, project_id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    convention.id,
                    convention.project_id,
                    convention.convention_type,
                    convention.convention_rule,
                    convention.good_examples,
                    convention.bad_examples,
                    convention.rationale,
                    convention.created_at
                ],
            ).map_err(|e| McpError {
                code: -1,
                message: format!("Failed to insert project convention: {}", e),
            })?;
        }

        tx.commit().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(conventions.to_vec())
    }

    async fn bulk_update(&self, conventions: &[ProjectConvention]) -> Result<Vec<ProjectConvention>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let tx = db.unchecked_transaction().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to start transaction: {}", e),
        })?;

        for convention in conventions {
            tx.execute(
                "UPDATE project_conventions SET project_id = ?2, convention_type = ?3, convention_rule = ?4, good_examples = ?5, bad_examples = ?6, rationale = ?7, created_at = ?8 WHERE id = ?1",
                params![
                    convention.id,
                    convention.project_id,
                    convention.convention_type,
                    convention.convention_rule,
                    convention.good_examples,
                    convention.bad_examples,
                    convention.rationale,
                    convention.created_at
                ],
            ).map_err(|e| McpError {
                code: -1,
                message: format!("Failed to update project convention: {}", e),
            })?;
        }

        tx.commit().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(conventions.to_vec())
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
            let rows_affected = tx.execute("DELETE FROM project_conventions WHERE id = ?1", params![id])
                .map_err(|e| McpError {
                    code: -1,
                    message: format!("Failed to delete project convention: {}", e),
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

// Feature Context Repository Implementation
pub struct SqliteFeatureContextRepository {
    db: Arc<Mutex<Connection>>,
}

impl SqliteFeatureContextRepository {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FeatureContextRepository for SqliteFeatureContextRepository {
    async fn create(&self, feature_context: &FeatureContext) -> Result<FeatureContext, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        db.execute(
            "INSERT INTO feature_context (id, project_id, feature_name, business_purpose, user_personas, key_workflows, integration_points, edge_cases, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                feature_context.id,
                feature_context.project_id,
                feature_context.feature_name,
                feature_context.business_purpose,
                feature_context.user_personas,
                feature_context.key_workflows,
                feature_context.integration_points,
                feature_context.edge_cases,
                feature_context.created_at
            ],
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to create feature context: {}", e),
        })?;

        Ok(feature_context.clone())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<FeatureContext>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, feature_name, business_purpose, user_personas, key_workflows, integration_points, edge_cases, created_at 
             FROM feature_context WHERE id = ?1"
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to prepare statement: {}", e),
        })?;

        let feature_result = stmt.query_row(params![id], |row| {
            Ok(FeatureContext {
                id: row.get(0)?,
                project_id: row.get(1)?,
                feature_name: row.get(2)?,
                business_purpose: row.get(3)?,
                user_personas: row.get(4)?,
                key_workflows: row.get(5)?,
                integration_points: row.get(6)?,
                edge_cases: row.get(7)?,
                created_at: row.get(8)?,
            })
        });

        match feature_result {
            Ok(feature_context) => Ok(Some(feature_context)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(McpError {
                code: -1,
                message: format!("Failed to get feature context: {}", e),
            }),
        }
    }

    async fn update(&self, feature_context: &FeatureContext) -> Result<FeatureContext, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        db.execute(
            "UPDATE feature_context SET project_id = ?2, feature_name = ?3, business_purpose = ?4, user_personas = ?5, key_workflows = ?6, integration_points = ?7, edge_cases = ?8, created_at = ?9 WHERE id = ?1",
            params![
                feature_context.id,
                feature_context.project_id,
                feature_context.feature_name,
                feature_context.business_purpose,
                feature_context.user_personas,
                feature_context.key_workflows,
                feature_context.integration_points,
                feature_context.edge_cases,
                feature_context.created_at
            ],
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to update feature context: {}", e),
        })?;

        Ok(feature_context.clone())
    }

    async fn delete(&self, id: &str) -> Result<bool, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let rows_affected = db.execute("DELETE FROM feature_context WHERE id = ?1", params![id])
            .map_err(|e| McpError {
                code: -1,
                message: format!("Failed to delete feature context: {}", e),
            })?;

        Ok(rows_affected > 0)
    }

    async fn list_by_project(&self, project_id: &str) -> Result<Vec<FeatureContext>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, feature_name, business_purpose, user_personas, key_workflows, integration_points, edge_cases, created_at 
             FROM feature_context WHERE project_id = ?1 ORDER BY created_at DESC"
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to prepare statement: {}", e),
        })?;

        let feature_iter = stmt.query_map(params![project_id], |row| {
            Ok(FeatureContext {
                id: row.get(0)?,
                project_id: row.get(1)?,
                feature_name: row.get(2)?,
                business_purpose: row.get(3)?,
                user_personas: row.get(4)?,
                key_workflows: row.get(5)?,
                integration_points: row.get(6)?,
                edge_cases: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to query feature contexts: {}", e),
        })?;

        let mut features = Vec::new();
        for feature in feature_iter {
            features.push(feature.map_err(|e| McpError {
                code: -1,
                message: format!("Failed to process feature context row: {}", e),
            })?);
        }

        Ok(features)
    }

    async fn get_by_feature_name(&self, project_id: &str, feature_name: &str) -> Result<Option<FeatureContext>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let mut stmt = db.prepare(
            "SELECT id, project_id, feature_name, business_purpose, user_personas, key_workflows, integration_points, edge_cases, created_at 
             FROM feature_context WHERE project_id = ?1 AND feature_name = ?2"
        ).map_err(|e| McpError {
            code: -1,
            message: format!("Failed to prepare statement: {}", e),
        })?;

        let feature_result = stmt.query_row(params![project_id, feature_name], |row| {
            Ok(FeatureContext {
                id: row.get(0)?,
                project_id: row.get(1)?,
                feature_name: row.get(2)?,
                business_purpose: row.get(3)?,
                user_personas: row.get(4)?,
                key_workflows: row.get(5)?,
                integration_points: row.get(6)?,
                edge_cases: row.get(7)?,
                created_at: row.get(8)?,
            })
        });

        match feature_result {
            Ok(feature_context) => Ok(Some(feature_context)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(McpError {
                code: -1,
                message: format!("Failed to get feature context by name: {}", e),
            }),
        }
    }

    async fn bulk_create(&self, feature_contexts: &[FeatureContext]) -> Result<Vec<FeatureContext>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let tx = db.unchecked_transaction().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to start transaction: {}", e),
        })?;

        for feature_context in feature_contexts {
            tx.execute(
                "INSERT INTO feature_context (id, project_id, feature_name, business_purpose, user_personas, key_workflows, integration_points, edge_cases, created_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    feature_context.id,
                    feature_context.project_id,
                    feature_context.feature_name,
                    feature_context.business_purpose,
                    feature_context.user_personas,
                    feature_context.key_workflows,
                    feature_context.integration_points,
                    feature_context.edge_cases,
                    feature_context.created_at
                ],
            ).map_err(|e| McpError {
                code: -1,
                message: format!("Failed to insert feature context: {}", e),
            })?;
        }

        tx.commit().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(feature_contexts.to_vec())
    }

    async fn bulk_update(&self, feature_contexts: &[FeatureContext]) -> Result<Vec<FeatureContext>, McpError> {
        let db = self.db.lock().map_err(|e| McpError {
            code: -1,
            message: format!("Database lock error: {}", e),
        })?;

        let tx = db.unchecked_transaction().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to start transaction: {}", e),
        })?;

        for feature_context in feature_contexts {
            tx.execute(
                "UPDATE feature_context SET project_id = ?2, feature_name = ?3, business_purpose = ?4, user_personas = ?5, key_workflows = ?6, integration_points = ?7, edge_cases = ?8, created_at = ?9 WHERE id = ?1",
                params![
                    feature_context.id,
                    feature_context.project_id,
                    feature_context.feature_name,
                    feature_context.business_purpose,
                    feature_context.user_personas,
                    feature_context.key_workflows,
                    feature_context.integration_points,
                    feature_context.edge_cases,
                    feature_context.created_at
                ],
            ).map_err(|e| McpError {
                code: -1,
                message: format!("Failed to update feature context: {}", e),
            })?;
        }

        tx.commit().map_err(|e| McpError {
            code: -1,
            message: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(feature_contexts.to_vec())
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
            let rows_affected = tx.execute("DELETE FROM feature_context WHERE id = ?1", params![id])
                .map_err(|e| McpError {
                    code: -1,
                    message: format!("Failed to delete feature context: {}", e),
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
