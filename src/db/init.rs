// Database initialization logic for context tables
use rusqlite::{Connection, Result};

pub fn init_db(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    // Business Rules
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS business_rules (
            id TEXT PRIMARY KEY,
            rule_name TEXT NOT NULL,
            description TEXT,
            domain_area TEXT,
            implementation_pattern TEXT,
            constraints TEXT,
            examples TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS architectural_decisions (
            id TEXT PRIMARY KEY,
            decision_title TEXT NOT NULL,
            context TEXT,
            decision TEXT,
            consequences TEXT,
            alternatives_considered TEXT,
            status TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS performance_requirements (
            id TEXT PRIMARY KEY,
            component_area TEXT,
            requirement_type TEXT,
            target_value TEXT,
            optimization_patterns TEXT,
            avoid_patterns TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS security_policies (
            id TEXT PRIMARY KEY,
            policy_name TEXT NOT NULL,
            policy_area TEXT,
            requirements TEXT,
            implementation_pattern TEXT,
            forbidden_patterns TEXT,
            compliance_notes TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS project_conventions (
            id TEXT PRIMARY KEY,
            convention_type TEXT,
            convention_rule TEXT,
            good_examples TEXT,
            bad_examples TEXT,
            rationale TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS feature_context (
            id TEXT PRIMARY KEY,
            feature_name TEXT NOT NULL,
            business_purpose TEXT,
            user_personas TEXT,
            key_workflows TEXT,
            integration_points TEXT,
            edge_cases TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
    "#)?;
    Ok(conn)
}
