/// End-to-end tests for OpenClaw integration workflow
use context_server_rs::models::{AuditEventType, AuditTrail, Constraint, ConstraintType, ComponentDependency, DependencyType};
use tempfile::tempdir;
use std::sync::Arc;

// Simulating OpenClaw workflow
#[tokio::test]
async fn test_openclaw_workflow_deployment() {
    // Setup
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("openclaw_test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Initialize database connection
    let conn = rusqlite::Connection::open(db_path_str).unwrap();
    conn.execute_batch(
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
            enforcement_action TEXT
        );
        CREATE TABLE IF NOT EXISTS component_dependencies (
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
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS audit_trails (
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
        );"
    ).unwrap();

    let conn = Arc::new(conn);

    // Initialize repositories
    let audit_repo = context_server_rs::infrastructure::SqliteAuditTrailRepository::new(conn.clone());
    audit_repo.init_table().unwrap();

    let constraint_repo = context_server_rs::infrastructure::SqliteConstraintRepository::new(conn.clone());
    constraint_repo.init_table().unwrap();

    let dependency_repo = context_server_rs::infrastructure::SqliteDependencyRepository::new(conn.clone());
    dependency_repo.init_table().unwrap();

    let project_id = "project_openclaw_01";

    // Step 1: Setup constraints (OpenClaw loads these at startup)
    println!("=== Step 1: Loading Safety Constraints ===");
    
    let constraint1 = Constraint::new(
        project_id.to_string(),
        ConstraintType::ApprovalRequired,
        "Production Deployment Approval".to_string(),
        "Requires manual approval from team leads".to_string(),
        "deployment:production".to_string(),
        "approval_threshold:2_team_leads".to_string(),
        "critical".to_string(),
    );
    constraint_repo.create_constraint(&constraint1).unwrap();
    println!("✓ Loaded constraint: {}", constraint1.name);

    let constraint2 = Constraint::new(
        project_id.to_string(),
        ConstraintType::SafetyGuard,
        "Staging Test Required".to_string(),
        "All changes must be tested in staging first".to_string(),
        "deployment:all".to_string(),
        "require_staging_test:true".to_string(),
        "critical".to_string(),
    );
    constraint_repo.create_constraint(&constraint2).unwrap();
    println!("✓ Loaded constraint: {}", constraint2.name);

    let constraint3 = Constraint::new(
        project_id.to_string(),
        ConstraintType::ResourceLimit,
        "Max Database Connections".to_string(),
        "Maximum simultaneous connections to database".to_string(),
        "service:database".to_string(),
        "max_connections:100".to_string(),
        "high".to_string(),
    );
    constraint_repo.create_constraint(&constraint3).unwrap();
    println!("✓ Loaded constraint: {}", constraint3.name);

    // Step 2: Setup dependencies (for impact analysis)
    println!("\n=== Step 2: Loading Component Dependencies ===");
    
    let dep1 = ComponentDependency::new(
        project_id.to_string(),
        "api-service".to_string(),
        "service".to_string(),
        "database".to_string(),
        "service".to_string(),
        DependencyType::DependsOn,
        "API queries database for all requests".to_string(),
    );
    dependency_repo.create_dependency(&dep1).unwrap();
    println!("✓ Registered dependency: {} depends on {}", dep1.source_component, dep1.target_component);

    let dep2 = ComponentDependency::new(
        project_id.to_string(),
        "api-service".to_string(),
        "service".to_string(),
        "auth-service".to_string(),
        "service".to_string(),
        DependencyType::DependsOn,
        "API validates requests with auth service".to_string(),
    );
    dependency_repo.create_dependency(&dep2).unwrap();
    println!("✓ Registered dependency: {} depends on {}", dep2.source_component, dep2.target_component);

    // Step 3: Query constraints before deployment
    println!("\n=== Step 3: OpenClaw Checks Constraints ===");
    let constraints = constraint_repo.list_constraints_by_target(project_id, "deployment:production").unwrap();
    println!("Found {} constraints for production deployment", constraints.len());
    
    let has_approval_required = constraints.iter().any(|c| matches!(c.constraint_type, ConstraintType::ApprovalRequired));
    assert!(has_approval_required, "Should require approval for production deployment");
    println!("⚠ Workflow requires: Manual approval from team leads");

    // Step 4: Query dependencies for impact analysis
    println!("\n=== Step 4: OpenClaw Analyzes Impact ===");
    let dependents = dependency_repo.get_dependents_of(project_id, "database").unwrap();
    println!("Found {} services that depend on database", dependents.len());
    
    assert!(dependents.len() > 0, "Should find services depending on database");
    for dep in &dependents {
        println!("  ⚠ Impact: {} will be affected", dep.source_component);
    }

    // Step 5: Simulate approval
    println!("\n=== Step 5: Deployment Approved ===");
    println!("Team leads approved deployment of v2.0");

    // Step 6: Log pre-deployment checks
    println!("\n=== Step 6: Pre-Deployment Checks ===");
    let audit_check = AuditTrail::new(
        AuditEventType::Created,
        "deployment".to_string(),
        "deploy_staging_v2.0".to_string(),
        "openclaw".to_string(),
        "Ran tests in staging environment".to_string(),
    )
    .with_project_id(project_id.to_string())
    .with_metadata(serde_json::json!({
        "environment": "staging",
        "version": "2.0",
        "status": "passed"
    }));
    audit_repo.log_event(&audit_check).unwrap();
    println!("✓ Staging tests passed");

    // Step 7: Execute deployment
    println!("\n=== Step 7: Deploying to Production ===");
    let audit_deploy = AuditTrail::new(
        AuditEventType::Created,
        "deployment".to_string(),
        "deploy_prod_v2.0".to_string(),
        "openclaw".to_string(),
        "Deployed service v2.0 to production".to_string(),
    )
    .with_project_id(project_id.to_string())
    .with_metadata(serde_json::json!({
        "environment": "production",
        "version": "2.0",
        "duration_seconds": 45,
        "status": "success",
        "deployed_at": "2024-02-04T15:30:00Z"
    }));
    audit_repo.log_event(&audit_deploy).unwrap();
    println!("✓ Deployment successful");

    // Step 8: Verify audit trail
    println!("\n=== Step 8: Audit Trail Review ===");
    let history = audit_repo.get_entity_history("deployment", "deploy_prod_v2.0").unwrap();
    assert!(history.len() > 0, "Should have audit trail entry");
    println!("✓ Audit trail recorded {} events", history.len());
    
    // Verify initiator is openclaw
    assert_eq!(history[0].initiator, "openclaw", "Should be initiated by openclaw");
    println!("✓ Initiator: {}", history[0].initiator);

    // Step 9: Monitor post-deployment
    println!("\n=== Step 9: Post-Deployment Monitoring ===");
    let audit_monitor = AuditTrail::new(
        AuditEventType::QueryExecuted,
        "monitoring".to_string(),
        "monitor_v2.0".to_string(),
        "openclaw".to_string(),
        "Verified health checks and performance metrics".to_string(),
    )
    .with_project_id(project_id.to_string())
    .with_metadata(serde_json::json!({
        "response_time_p99": "125ms",
        "error_rate": "0.01%",
        "availability": "99.99%",
        "status": "healthy"
    }));
    audit_repo.log_event(&audit_monitor).unwrap();
    println!("✓ All health checks passed");
    println!("✓ Performance metrics normal");

    // Step 10: Final report
    println!("\n=== Deployment Complete ===");
    let all_actions = audit_repo.get_initiator_actions("openclaw", 100).unwrap();
    println!("Total actions by OpenClaw: {}", all_actions.len());
    println!("✓ Workflow completed successfully\n");

    assert!(all_actions.len() >= 3, "Should have at least 3 audit entries");
}

#[tokio::test]
async fn test_openclaw_constraint_violation_prevention() {
    // Setup
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("constraint_test.db");
    let db_path_str = db_path.to_str().unwrap();

    let conn = rusqlite::Connection::open(db_path_str).unwrap();
    conn.execute_batch(
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
            enforcement_action TEXT
        );"
    ).unwrap();

    let conn = Arc::new(conn);
    let constraint_repo = context_server_rs::infrastructure::SqliteConstraintRepository::new(conn.clone());
    constraint_repo.init_table().unwrap();

    let project_id = "project_safety_01";

    // Create a critical safety constraint
    let critical_constraint = Constraint::new(
        project_id.to_string(),
        ConstraintType::SafetyGuard,
        "No Direct Database Modifications".to_string(),
        "Schema changes must use migrations only".to_string(),
        "database:schema".to_string(),
        "method:migrations_only".to_string(),
        "critical".to_string(),
    );
    constraint_repo.create_constraint(&critical_constraint).unwrap();

    // Step 1: OpenClaw plans a risky action
    println!("=== Safety Constraint Test ===");
    println!("OpenClaw: Planning direct schema modification...");

    // Step 2: Query constraints
    let constraints = constraint_repo.list_constraints_by_target(project_id, "database:schema").unwrap();
    assert!(constraints.len() > 0, "Should find safety constraint");

    // Step 3: Check enforcement action
    let safety_guard = constraints.iter().find(|c| matches!(c.constraint_type, ConstraintType::SafetyGuard));
    assert!(safety_guard.is_some(), "Should find safety guard");

    if let Some(guard) = safety_guard {
        println!("⚠ Constraint found: {}", guard.name);
        println!("✓ Constraint enforcement: {:?}", guard.enforcement_action);
        println!("✓ Action blocked: Cannot proceed with direct database modification");
        assert!(guard.enabled, "Constraint should be enabled");
    }
}

#[tokio::test]
async fn test_openclaw_dependency_impact_analysis() {
    // Setup
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("dependency_test.db");
    let db_path_str = db_path.to_str().unwrap();

    let conn = rusqlite::Connection::open(db_path_str).unwrap();
    conn.execute_batch(
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
            created_at TEXT NOT NULL
        );"
    ).unwrap();

    let conn = Arc::new(conn);
    let dependency_repo = context_server_rs::infrastructure::SqliteDependencyRepository::new(conn.clone());
    dependency_repo.init_table().unwrap();

    let project_id = "project_impact_01";

    // Create dependency chain
    println!("=== Impact Analysis Test ===");
    
    let db_dep = ComponentDependency::new(
        project_id.to_string(),
        "api".to_string(),
        "service".to_string(),
        "database".to_string(),
        "service".to_string(),
        DependencyType::DependsOn,
        "API queries database".to_string(),
    );
    dependency_repo.create_dependency(&db_dep).unwrap();

    let cache_dep = ComponentDependency::new(
        project_id.to_string(),
        "api".to_string(),
        "service".to_string(),
        "redis".to_string(),
        "service".to_string(),
        DependencyType::DependsOn,
        "API uses cache".to_string(),
    );
    dependency_repo.create_dependency(&cache_dep).unwrap();

    let worker_dep = ComponentDependency::new(
        project_id.to_string(),
        "worker".to_string(),
        "service".to_string(),
        "database".to_string(),
        "service".to_string(),
        DependencyType::DependsOn,
        "Worker processes database queue".to_string(),
    );
    dependency_repo.create_dependency(&worker_dep).unwrap();

    // Query impact: What happens if database goes down?
    println!("OpenClaw: Analyzing impact of database maintenance...");
    let impacted = dependency_repo.get_dependents_of(project_id, "database").unwrap();
    
    println!("✓ Found {} services impacted by database downtime:", impacted.len());
    for dep in impacted {
        println!("  - {} will be affected", dep.source_component);
    }

    assert!(impacted.len() == 2, "API and Worker should depend on database");

    // Query dependencies: What does API depend on?
    println!("\nOpenClaw: Checking API dependencies for deployment safety...");
    let api_deps = dependency_repo.get_dependencies_of(project_id, "api").unwrap();
    
    println!("✓ API depends on {} services:", api_deps.len());
    for dep in api_deps {
        println!("  - {} ({} criticality)", dep.target_component, dep.criticality);
    }

    assert!(api_deps.len() == 2, "API should depend on database and redis");
}
