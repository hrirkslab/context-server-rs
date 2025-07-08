// Database initialization logic for context tables
use rusqlite::{Connection, Result};

pub fn init_db(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    
    // Projects table
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            repository_url TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );
    "#)?;
    
    // Business Rules
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS business_rules (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            rule_name TEXT NOT NULL,
            description TEXT,
            domain_area TEXT,
            implementation_pattern TEXT,
            constraints TEXT,
            examples TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
        CREATE TABLE IF NOT EXISTS architectural_decisions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            decision_title TEXT NOT NULL,
            context TEXT,
            decision TEXT,
            consequences TEXT,
            alternatives_considered TEXT,
            status TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
        CREATE TABLE IF NOT EXISTS performance_requirements (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            component_area TEXT,
            requirement_type TEXT,
            target_value TEXT,
            optimization_patterns TEXT,
            avoid_patterns TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
        CREATE TABLE IF NOT EXISTS security_policies (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            policy_name TEXT NOT NULL,
            policy_area TEXT,
            requirements TEXT,
            implementation_pattern TEXT,
            forbidden_patterns TEXT,
            compliance_notes TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
        CREATE TABLE IF NOT EXISTS project_conventions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            convention_type TEXT,
            convention_rule TEXT,
            good_examples TEXT,
            bad_examples TEXT,
            rationale TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
        CREATE TABLE IF NOT EXISTS feature_context (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            feature_name TEXT NOT NULL,
            business_purpose TEXT,
            user_personas TEXT,
            key_workflows TEXT,
            integration_points TEXT,
            edge_cases TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
    "#)?;
    Ok(conn)
}
