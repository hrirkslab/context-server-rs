/// SQLite repository for constraints and dependencies
use crate::models::constraint::{ComponentDependency, Constraint, ConstraintType, DependencyType};
use std::sync::Arc;

pub trait ConstraintRepository: Send {
    fn create_constraint(&self, constraint: &Constraint) -> anyhow::Result<()>;
    fn get_constraint(&self, id: &str) -> anyhow::Result<Option<Constraint>>;
    fn list_constraints(&self, project_id: &str) -> anyhow::Result<Vec<Constraint>>;
    fn list_constraints_by_type(&self, project_id: &str, constraint_type: &str) -> anyhow::Result<Vec<Constraint>>;
    fn list_constraints_by_target(&self, project_id: &str, target: &str) -> anyhow::Result<Vec<Constraint>>;
    fn update_constraint(&self, constraint: &Constraint) -> anyhow::Result<()>;
    fn delete_constraint(&self, id: &str) -> anyhow::Result<()>;
}

pub struct SqliteConstraintRepository {
    conn: Arc<rusqlite::Connection>,
}

impl SqliteConstraintRepository {
    pub fn new(conn: Arc<rusqlite::Connection>) -> Self {
        Self { conn }
    }

    pub fn init_table(&self) -> anyhow::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS constraints (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                constraint_type TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                target TEXT NOT NULL,
                value TEXT NOT NULL,
                severity TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                last_modified_at TEXT NOT NULL,
                tags TEXT,
                enforcement_action TEXT,
                FOREIGN KEY(project_id) REFERENCES projects(id)
            )",
            [],
        )?;

        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_constraint_project ON constraints(project_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_constraint_type ON constraints(constraint_type)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_constraint_target ON constraints(target)",
            [],
        );

        Ok(())
    }
}

impl ConstraintRepository for SqliteConstraintRepository {
    fn create_constraint(&self, constraint: &Constraint) -> anyhow::Result<()> {
        let tags_json = serde_json::to_string(&constraint.tags)?;
        
        self.conn.execute(
            "INSERT INTO constraints (id, project_id, constraint_type, name, description, target, 
             value, severity, enabled, created_at, last_modified_at, tags, enforcement_action)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                constraint.id,
                constraint.project_id,
                constraint.constraint_type.as_str(),
                constraint.name,
                constraint.description,
                constraint.target,
                constraint.value,
                constraint.severity,
                constraint.enabled as i32,
                constraint.created_at,
                constraint.last_modified_at,
                tags_json,
                constraint.enforcement_action,
            ],
        )?;

        Ok(())
    }

    fn get_constraint(&self, id: &str) -> anyhow::Result<Option<Constraint>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, constraint_type, name, description, target, value, 
             severity, enabled, created_at, last_modified_at, tags, enforcement_action
             FROM constraints WHERE id = ? LIMIT 1",
        )?;

        let result = stmt.query_row(rusqlite::params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, i32>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, Option<String>>(12)?,
            ))
        });

        match result {
            Ok((id, project_id, constraint_type_str, name, description, target, value, severity, enabled, created_at, last_modified_at, tags_json, enforcement_action)) => {
                let constraint_type = match constraint_type_str.as_str() {
                    "resource_limit" => ConstraintType::ResourceLimit,
                    "safety_guard" => ConstraintType::SafetyGuard,
                    "rollback_procedure" => ConstraintType::RollbackProcedure,
                    "approval_required" => ConstraintType::ApprovalRequired,
                    "performance_target" => ConstraintType::PerformanceTarget,
                    "security_requirement" => ConstraintType::SecurityRequirement,
                    _ => ConstraintType::SafetyGuard,
                };

                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

                Ok(Some(Constraint {
                    id,
                    project_id,
                    constraint_type,
                    name,
                    description,
                    target,
                    value,
                    severity,
                    enabled: enabled != 0,
                    created_at,
                    last_modified_at,
                    tags,
                    enforcement_action,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn list_constraints(&self, project_id: &str) -> anyhow::Result<Vec<Constraint>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, constraint_type, name, description, target, value, 
             severity, enabled, created_at, last_modified_at, tags, enforcement_action
             FROM constraints WHERE project_id = ? ORDER BY severity DESC",
        )?;

        let constraints = stmt.query_map(rusqlite::params![project_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, i32>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, Option<String>>(12)?,
            ))
        })?;

        let mut result = Vec::new();
        for constraint_row in constraints {
            let (id, project_id, constraint_type_str, name, description, target, value, severity, enabled, created_at, last_modified_at, tags_json, enforcement_action) = constraint_row?;

            let constraint_type = match constraint_type_str.as_str() {
                "resource_limit" => ConstraintType::ResourceLimit,
                "safety_guard" => ConstraintType::SafetyGuard,
                "rollback_procedure" => ConstraintType::RollbackProcedure,
                "approval_required" => ConstraintType::ApprovalRequired,
                "performance_target" => ConstraintType::PerformanceTarget,
                "security_requirement" => ConstraintType::SecurityRequirement,
                _ => ConstraintType::SafetyGuard,
            };

            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

            result.push(Constraint {
                id,
                project_id,
                constraint_type,
                name,
                description,
                target,
                value,
                severity,
                enabled: enabled != 0,
                created_at,
                last_modified_at,
                tags,
                enforcement_action,
            });
        }

        Ok(result)
    }

    fn list_constraints_by_type(&self, project_id: &str, constraint_type: &str) -> anyhow::Result<Vec<Constraint>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, constraint_type, name, description, target, value, 
             severity, enabled, created_at, last_modified_at, tags, enforcement_action
             FROM constraints WHERE project_id = ? AND constraint_type = ? ORDER BY severity DESC",
        )?;

        let constraints = stmt.query_map(rusqlite::params![project_id, constraint_type], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, i32>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, Option<String>>(12)?,
            ))
        })?;

        let mut result = Vec::new();
        for constraint_row in constraints {
            let (id, project_id, constraint_type_str, name, description, target, value, severity, enabled, created_at, last_modified_at, tags_json, enforcement_action) = constraint_row?;

            let constraint_type = match constraint_type_str.as_str() {
                "resource_limit" => ConstraintType::ResourceLimit,
                "safety_guard" => ConstraintType::SafetyGuard,
                "rollback_procedure" => ConstraintType::RollbackProcedure,
                "approval_required" => ConstraintType::ApprovalRequired,
                "performance_target" => ConstraintType::PerformanceTarget,
                "security_requirement" => ConstraintType::SecurityRequirement,
                _ => ConstraintType::SafetyGuard,
            };

            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

            result.push(Constraint {
                id,
                project_id,
                constraint_type,
                name,
                description,
                target,
                value,
                severity,
                enabled: enabled != 0,
                created_at,
                last_modified_at,
                tags,
                enforcement_action,
            });
        }

        Ok(result)
    }

    fn list_constraints_by_target(&self, project_id: &str, target: &str) -> anyhow::Result<Vec<Constraint>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, constraint_type, name, description, target, value, 
             severity, enabled, created_at, last_modified_at, tags, enforcement_action
             FROM constraints WHERE project_id = ? AND target = ? ORDER BY severity DESC",
        )?;

        let constraints = stmt.query_map(rusqlite::params![project_id, target], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, i32>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, Option<String>>(12)?,
            ))
        })?;

        let mut result = Vec::new();
        for constraint_row in constraints {
            let (id, project_id, constraint_type_str, name, description, target, value, severity, enabled, created_at, last_modified_at, tags_json, enforcement_action) = constraint_row?;

            let constraint_type = match constraint_type_str.as_str() {
                "resource_limit" => ConstraintType::ResourceLimit,
                "safety_guard" => ConstraintType::SafetyGuard,
                "rollback_procedure" => ConstraintType::RollbackProcedure,
                "approval_required" => ConstraintType::ApprovalRequired,
                "performance_target" => ConstraintType::PerformanceTarget,
                "security_requirement" => ConstraintType::SecurityRequirement,
                _ => ConstraintType::SafetyGuard,
            };

            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

            result.push(Constraint {
                id,
                project_id,
                constraint_type,
                name,
                description,
                target,
                value,
                severity,
                enabled: enabled != 0,
                created_at,
                last_modified_at,
                tags,
                enforcement_action,
            });
        }

        Ok(result)
    }

    fn update_constraint(&self, constraint: &Constraint) -> anyhow::Result<()> {
        let tags_json = serde_json::to_string(&constraint.tags)?;
        
        self.conn.execute(
            "UPDATE constraints SET constraint_type = ?, name = ?, description = ?, target = ?, 
             value = ?, severity = ?, enabled = ?, last_modified_at = ?, tags = ?, enforcement_action = ?
             WHERE id = ?",
            rusqlite::params![
                constraint.constraint_type.as_str(),
                constraint.name,
                constraint.description,
                constraint.target,
                constraint.value,
                constraint.severity,
                constraint.enabled as i32,
                constraint.last_modified_at,
                tags_json,
                constraint.enforcement_action,
                constraint.id,
            ],
        )?;

        Ok(())
    }

    fn delete_constraint(&self, id: &str) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM constraints WHERE id = ?", rusqlite::params![id])?;
        Ok(())
    }
}

// Dependency Repository
pub trait DependencyRepository: Send {
    fn create_dependency(&self, dependency: &ComponentDependency) -> anyhow::Result<()>;
    fn get_dependency(&self, id: &str) -> anyhow::Result<Option<ComponentDependency>>;
    fn list_dependencies(&self, project_id: &str) -> anyhow::Result<Vec<ComponentDependency>>;
    fn get_dependencies_of(&self, project_id: &str, component: &str) -> anyhow::Result<Vec<ComponentDependency>>;
    fn get_dependents_of(&self, project_id: &str, component: &str) -> anyhow::Result<Vec<ComponentDependency>>;
    fn delete_dependency(&self, id: &str) -> anyhow::Result<()>;
}

pub struct SqliteDependencyRepository {
    conn: Arc<rusqlite::Connection>,
}

impl SqliteDependencyRepository {
    pub fn new(conn: Arc<rusqlite::Connection>) -> Self {
        Self { conn }
    }

    pub fn init_table(&self) -> anyhow::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS component_dependencies (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                source_component TEXT NOT NULL,
                source_type TEXT NOT NULL,
                target_component TEXT NOT NULL,
                target_type TEXT NOT NULL,
                dependency_type TEXT NOT NULL,
                description TEXT,
                criticality TEXT NOT NULL,
                impact_on_failure TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id)
            )",
            [],
        )?;

        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dependency_project ON component_dependencies(project_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dependency_source ON component_dependencies(source_component)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dependency_target ON component_dependencies(target_component)",
            [],
        );

        Ok(())
    }
}

impl DependencyRepository for SqliteDependencyRepository {
    fn create_dependency(&self, dependency: &ComponentDependency) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO component_dependencies (id, project_id, source_component, source_type, 
             target_component, target_type, dependency_type, description, criticality, impact_on_failure, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                dependency.id,
                dependency.project_id,
                dependency.source_component,
                dependency.source_type,
                dependency.target_component,
                dependency.target_type,
                dependency.dependency_type.as_str(),
                dependency.description,
                dependency.criticality,
                dependency.impact_on_failure,
                dependency.created_at,
            ],
        )?;

        Ok(())
    }

    fn get_dependency(&self, id: &str) -> anyhow::Result<Option<ComponentDependency>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, source_component, source_type, target_component, target_type, 
             dependency_type, description, criticality, impact_on_failure, created_at
             FROM component_dependencies WHERE id = ? LIMIT 1",
        )?;

        let result = stmt.query_row(rusqlite::params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, String>(10)?,
            ))
        });

        match result {
            Ok((id, project_id, source_component, source_type, target_component, target_type, dep_type_str, description, criticality, impact_on_failure, created_at)) => {
                let dependency_type = match dep_type_str.as_str() {
                    "requires" => DependencyType::Requires,
                    "required_by" => DependencyType::RequiredBy,
                    "depends_on" => DependencyType::DependsOn,
                    "blocks" => DependencyType::Blocks,
                    "triggers" => DependencyType::Triggers,
                    "communicates" => DependencyType::Communicates,
                    _ => DependencyType::DependsOn,
                };

                Ok(Some(ComponentDependency {
                    id,
                    project_id,
                    source_component,
                    source_type,
                    target_component,
                    target_type,
                    dependency_type,
                    description,
                    criticality,
                    impact_on_failure,
                    created_at,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn list_dependencies(&self, project_id: &str) -> anyhow::Result<Vec<ComponentDependency>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, source_component, source_type, target_component, target_type, 
             dependency_type, description, criticality, impact_on_failure, created_at
             FROM component_dependencies WHERE project_id = ? ORDER BY criticality DESC",
        )?;

        let deps = stmt.query_map(rusqlite::params![project_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, String>(10)?,
            ))
        })?;

        let mut result = Vec::new();
        for dep_row in deps {
            let (id, project_id, source_component, source_type, target_component, target_type, dep_type_str, description, criticality, impact_on_failure, created_at) = dep_row?;

            let dependency_type = match dep_type_str.as_str() {
                "requires" => DependencyType::Requires,
                "required_by" => DependencyType::RequiredBy,
                "depends_on" => DependencyType::DependsOn,
                "blocks" => DependencyType::Blocks,
                "triggers" => DependencyType::Triggers,
                "communicates" => DependencyType::Communicates,
                _ => DependencyType::DependsOn,
            };

            result.push(ComponentDependency {
                id,
                project_id,
                source_component,
                source_type,
                target_component,
                target_type,
                dependency_type,
                description,
                criticality,
                impact_on_failure,
                created_at,
            });
        }

        Ok(result)
    }

    fn get_dependencies_of(&self, project_id: &str, component: &str) -> anyhow::Result<Vec<ComponentDependency>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, source_component, source_type, target_component, target_type, 
             dependency_type, description, criticality, impact_on_failure, created_at
             FROM component_dependencies WHERE project_id = ? AND source_component = ?",
        )?;

        let deps = stmt.query_map(rusqlite::params![project_id, component], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, String>(10)?,
            ))
        })?;

        let mut result = Vec::new();
        for dep_row in deps {
            let (id, project_id, source_component, source_type, target_component, target_type, dep_type_str, description, criticality, impact_on_failure, created_at) = dep_row?;

            let dependency_type = match dep_type_str.as_str() {
                "requires" => DependencyType::Requires,
                "required_by" => DependencyType::RequiredBy,
                "depends_on" => DependencyType::DependsOn,
                "blocks" => DependencyType::Blocks,
                "triggers" => DependencyType::Triggers,
                "communicates" => DependencyType::Communicates,
                _ => DependencyType::DependsOn,
            };

            result.push(ComponentDependency {
                id,
                project_id,
                source_component,
                source_type,
                target_component,
                target_type,
                dependency_type,
                description,
                criticality,
                impact_on_failure,
                created_at,
            });
        }

        Ok(result)
    }

    fn get_dependents_of(&self, project_id: &str, component: &str) -> anyhow::Result<Vec<ComponentDependency>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, source_component, source_type, target_component, target_type, 
             dependency_type, description, criticality, impact_on_failure, created_at
             FROM component_dependencies WHERE project_id = ? AND target_component = ?",
        )?;

        let deps = stmt.query_map(rusqlite::params![project_id, component], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, String>(10)?,
            ))
        })?;

        let mut result = Vec::new();
        for dep_row in deps {
            let (id, project_id, source_component, source_type, target_component, target_type, dep_type_str, description, criticality, impact_on_failure, created_at) = dep_row?;

            let dependency_type = match dep_type_str.as_str() {
                "requires" => DependencyType::Requires,
                "required_by" => DependencyType::RequiredBy,
                "depends_on" => DependencyType::DependsOn,
                "blocks" => DependencyType::Blocks,
                "triggers" => DependencyType::Triggers,
                "communicates" => DependencyType::Communicates,
                _ => DependencyType::DependsOn,
            };

            result.push(ComponentDependency {
                id,
                project_id,
                source_component,
                source_type,
                target_component,
                target_type,
                dependency_type,
                description,
                criticality,
                impact_on_failure,
                created_at,
            });
        }

        Ok(result)
    }

    fn delete_dependency(&self, id: &str) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM component_dependencies WHERE id = ?", rusqlite::params![id])?;
        Ok(())
    }
}
