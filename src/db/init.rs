// Database initialization logic for context tables
use rusqlite::{Connection, Result};

pub fn init_db(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // Projects table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            repository_url TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );
    "#,
    )?;

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
        
        -- Framework-agnostic component tables
        CREATE TABLE IF NOT EXISTS framework_components (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            component_name TEXT NOT NULL,
            component_type TEXT NOT NULL, -- 'widget', 'provider', 'service', 'repository', 'model', 'utility'
            architecture_layer TEXT NOT NULL, -- 'presentation', 'domain', 'data', 'core'
            file_path TEXT,
            dependencies TEXT, -- JSON array of dependencies
            metadata TEXT, -- JSON metadata for framework-specific properties
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
        
        -- Flutter-specific context tables (legacy)
        CREATE TABLE IF NOT EXISTS flutter_components (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            component_name TEXT NOT NULL,
            component_type TEXT NOT NULL, -- 'widget', 'provider', 'service', 'repository'
            architecture_layer TEXT NOT NULL, -- 'presentation', 'domain', 'data', 'core'
            file_path TEXT,
            dependencies TEXT, -- JSON array of dependencies
            riverpod_scope TEXT, -- 'global', 'scoped', 'local'
            widget_type TEXT, -- 'StatelessWidget', 'StatefulWidget', 'ConsumerWidget'
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
        
        CREATE TABLE IF NOT EXISTS development_phases (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            phase_name TEXT NOT NULL, -- 'Setup', 'Chat UI', 'Model Management', 'Polish'
            phase_order INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'in_progress', 'completed', 'blocked'
            description TEXT,
            completion_criteria TEXT, -- JSON array of criteria
            dependencies TEXT, -- JSON array of phase dependencies
            started_at TEXT,
            completed_at TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
        
        CREATE TABLE IF NOT EXISTS privacy_rules (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            rule_name TEXT NOT NULL,
            rule_type TEXT NOT NULL, -- 'forbidden_import', 'required_local_storage', 'data_flow'
            pattern TEXT NOT NULL, -- import pattern, storage key, etc.
            description TEXT,
            severity TEXT DEFAULT 'error', -- 'error', 'warning', 'info'
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
        
        CREATE TABLE IF NOT EXISTS privacy_violations (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            rule_id TEXT NOT NULL,
            file_path TEXT NOT NULL,
            line_number INTEGER,
            violation_text TEXT,
            status TEXT DEFAULT 'open', -- 'open', 'resolved', 'suppressed'
            detected_at TEXT DEFAULT (datetime('now')),
            resolved_at TEXT,
            FOREIGN KEY (project_id) REFERENCES projects(id),
            FOREIGN KEY (rule_id) REFERENCES privacy_rules(id)
        );
        
        CREATE TABLE IF NOT EXISTS architecture_layers (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            layer_name TEXT NOT NULL, -- 'presentation', 'domain', 'data', 'core'
            allowed_dependencies TEXT, -- JSON array of allowed layer dependencies
            forbidden_imports TEXT, -- JSON array of forbidden import patterns
            description TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
        
        CREATE TABLE IF NOT EXISTS model_context (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            model_name TEXT NOT NULL,
            model_path TEXT,
            model_type TEXT, -- 'GGUF', 'ONNX', etc.
            model_size TEXT,
            performance_metrics TEXT, -- JSON with inference times, memory usage
            configuration TEXT, -- JSON with model settings
            is_active BOOLEAN DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
        
        CREATE TABLE IF NOT EXISTS code_templates (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            template_name TEXT NOT NULL,
            template_type TEXT NOT NULL, -- 'widget', 'provider', 'repository', 'test'
            template_content TEXT NOT NULL,
            variables TEXT, -- JSON array of template variables
            description TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );
    "#)?;
    Ok(conn)
}
