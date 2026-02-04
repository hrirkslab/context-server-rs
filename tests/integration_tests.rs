use tempfile::tempdir;

use context_server_rs::container::AppContainer;
use context_server_rs::db::init::init_db;
use context_server_rs::models::context::{
    ArchitecturalDecision, BusinessRule, DevelopmentPhase, PerformanceRequirement,
    SecurityPolicy,
};
use uuid::Uuid;

#[tokio::test]
async fn test_database_initialization() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Test database initialization
    let result = init_db(db_path_str);
    assert!(result.is_ok(), "Database initialization should succeed");

    // Test that we can create an app container
    let container_result = AppContainer::new(db_path_str);
    assert!(
        container_result.is_ok(),
        "AppContainer creation should succeed"
    );
}

#[tokio::test]
async fn test_project_crud_operations() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Initialize database and container
    init_db(db_path_str).unwrap();
    let container = AppContainer::new(db_path_str).unwrap();

    // Test project creation
    let project = container
        .project_service
        .create_project("Test Project", Some("A test project"), None)
        .await;

    assert!(project.is_ok(), "Project creation should succeed");
    let project = project.unwrap();
    assert_eq!(project.name, "Test Project");
    assert_eq!(project.description, Some("A test project".to_string()));

    // Test project retrieval
    let retrieved = container.project_service.get_project(&project.id).await;

    assert!(retrieved.is_ok(), "Project retrieval should succeed");
    let retrieved = retrieved.unwrap();
    assert!(retrieved.is_some(), "Project should exist");

    // Test project deletion
    let deleted = container.project_service.delete_project(&project.id).await;

    assert!(deleted.is_ok(), "Project deletion should succeed");
    assert!(deleted.unwrap(), "Project should be deleted");
}

#[tokio::test]
async fn test_framework_component_operations() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Initialize database and container
    init_db(db_path_str).unwrap();
    let container = AppContainer::new(db_path_str).unwrap();

    // Create a test project first
    let project = container
        .project_service
        .create_project("Test Project", Some("A test project"), None)
        .await
        .unwrap();

    // Test component creation using framework_service (was component_service)
    let component = container
        .framework_service
        .create_component(
            &project.id,
            "TestWidget",
            "widget",
            "presentation",
            Some("/src/widgets/test_widget.dart"),
            None,
        )
        .await;

    assert!(component.is_ok(), "Component creation should succeed");
    let component = component.unwrap();
    assert_eq!(component.component_name, "TestWidget");
    assert_eq!(component.architecture_layer, "presentation");

    // Test component retrieval using framework_service (was component_service)
    let retrieved = container
        .framework_service
        .get_component(&component.id)
        .await;

    assert!(retrieved.is_ok(), "Component retrieval should succeed");
    assert!(retrieved.unwrap().is_some(), "Component should exist");

    // Test listing components by project using framework_service (was component_service)
    let components = container
        .framework_service
        .list_components(&project.id)
        .await;

    assert!(components.is_ok(), "Component listing should succeed");
    assert_eq!(components.unwrap().len(), 1, "Should have one component");
}

#[tokio::test]
async fn test_business_rule_crud_operations() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Initialize database and container
    init_db(db_path_str).unwrap();
    let container = AppContainer::new(db_path_str).unwrap();

    // Create a test project first
    let project = container
        .project_service
        .create_project("Test Project", Some("A test project"), None)
        .await
        .unwrap();

    // Test business rule creation
    let rule = container
        .context_crud_service
        .create_business_rule(
            &project.id,
            "Authentication Rule",
            Some("Users must authenticate before accessing protected resources"),
            Some("authentication"),
        )
        .await;

    assert!(rule.is_ok(), "Business rule creation should succeed");
    let rule = rule.unwrap();
    assert_eq!(rule.rule_name, "Authentication Rule");
    assert_eq!(rule.domain_area, Some("authentication".to_string()));

    // Test business rule retrieval
    let retrieved = container
        .context_crud_service
        .get_business_rule(&rule.id)
        .await;

    assert!(retrieved.is_ok(), "Business rule retrieval should succeed");
    assert!(retrieved.unwrap().is_some(), "Business rule should exist");

    // Test business rule update
    let mut updated_rule = rule.clone();
    updated_rule.description = Some("Updated description".to_string());

    let update_result = container
        .context_crud_service
        .update_business_rule(&updated_rule)
        .await;

    assert!(update_result.is_ok(), "Business rule update should succeed");

    // Test business rule listing
    let rules = container
        .context_crud_service
        .list_business_rules(&project.id)
        .await;

    assert!(rules.is_ok(), "Business rule listing should succeed");
    assert_eq!(rules.unwrap().len(), 1, "Should have one business rule");

    // Test business rule deletion
    let deleted = container
        .context_crud_service
        .delete_business_rule(&rule.id)
        .await;

    assert!(deleted.is_ok(), "Business rule deletion should succeed");
    assert!(deleted.unwrap(), "Business rule should be deleted");
}

#[tokio::test]
async fn test_architectural_decision_crud_operations() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Initialize database and container
    init_db(db_path_str).unwrap();
    let container = AppContainer::new(db_path_str).unwrap();

    // Create a test project first
    let project = container
        .project_service
        .create_project("Test Project", Some("A test project"), None)
        .await
        .unwrap();

    // Test architectural decision creation
    let decision = container
        .context_crud_service
        .create_architectural_decision(
            &project.id,
            "Use MVC Pattern",
            Some("Need clear separation of concerns"),
            Some("Implement Model-View-Controller pattern"),
        )
        .await;

    assert!(decision.is_ok(), "Architectural decision creation should succeed");
    let decision = decision.unwrap();
    assert_eq!(decision.decision_title, "Use MVC Pattern");

    // Test architectural decision retrieval
    let retrieved = container
        .context_crud_service
        .get_architectural_decision(&decision.id)
        .await;

    assert!(retrieved.is_ok(), "Architectural decision retrieval should succeed");
    assert!(retrieved.unwrap().is_some(), "Architectural decision should exist");

    // Test architectural decision update
    let mut updated_decision = decision.clone();
    updated_decision.status = Some("active".to_string());

    let update_result = container
        .context_crud_service
        .update_architectural_decision(&updated_decision)
        .await;

    assert!(
        update_result.is_ok(),
        "Architectural decision update should succeed"
    );

    // Test architectural decision listing
    let decisions = container
        .context_crud_service
        .list_architectural_decisions(&project.id)
        .await;

    assert!(decisions.is_ok(), "Architectural decision listing should succeed");
    assert_eq!(decisions.unwrap().len(), 1, "Should have one architectural decision");

    // Test architectural decision deletion
    let deleted = container
        .context_crud_service
        .delete_architectural_decision(&decision.id)
        .await;

    assert!(deleted.is_ok(), "Architectural decision deletion should succeed");
    assert!(deleted.unwrap(), "Architectural decision should be deleted");
}

#[tokio::test]
async fn test_performance_requirement_crud_operations() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Initialize database and container
    init_db(db_path_str).unwrap();
    let container = AppContainer::new(db_path_str).unwrap();

    // Create a test project first
    let project = container
        .project_service
        .create_project("Test Project", Some("A test project"), None)
        .await
        .unwrap();

    // Test performance requirement creation
    let perf_req = container
        .context_crud_service
        .create_performance_requirement(
            &project.id,
            Some("API Response Time"),
            Some("response_time"),
            Some("< 100ms"),
            Some("Connection pooling, caching"),
            Some("N1 queries, blocking calls"),
        )
        .await;

    assert!(perf_req.is_ok(), "Performance requirement creation should succeed");
    let perf_req = perf_req.unwrap();
    assert_eq!(perf_req.component_area, Some("API Response Time".to_string()));
    assert_eq!(perf_req.target_value, Some("< 100ms".to_string()));

    // Test performance requirement retrieval
    let retrieved = container
        .context_crud_service
        .get_performance_requirement(&perf_req.id)
        .await;

    assert!(retrieved.is_ok(), "Performance requirement retrieval should succeed");
    assert!(retrieved.unwrap().is_some(), "Performance requirement should exist");

    // Test performance requirement update
    let mut updated_perf = perf_req.clone();
    updated_perf.target_value = Some("< 50ms".to_string());

    let update_result = container
        .context_crud_service
        .update_performance_requirement(&updated_perf)
        .await;

    assert!(
        update_result.is_ok(),
        "Performance requirement update should succeed"
    );

    // Test performance requirement listing
    let requirements = container
        .context_crud_service
        .list_performance_requirements(&project.id)
        .await;

    assert!(requirements.is_ok(), "Performance requirement listing should succeed");
    assert_eq!(requirements.unwrap().len(), 1, "Should have one performance requirement");

    // Test performance requirement deletion
    let deleted = container
        .context_crud_service
        .delete_performance_requirement(&perf_req.id)
        .await;

    assert!(deleted.is_ok(), "Performance requirement deletion should succeed");
    assert!(deleted.unwrap(), "Performance requirement should be deleted");
}

#[tokio::test]
async fn test_security_policy_crud_operations() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Initialize database and container
    init_db(db_path_str).unwrap();
    let container = AppContainer::new(db_path_str).unwrap();

    // Create a test project first
    let project = container
        .project_service
        .create_project("Test Project", Some("A test project"), None)
        .await
        .unwrap();

    // Test security policy creation
    let policy = container
        .context_crud_service
        .create_security_policy(
            &project.id,
            "GDPR Compliance",
            Some("data_storage"),
        )
        .await;

    assert!(policy.is_ok(), "Security policy creation should succeed");
    let policy = policy.unwrap();
    assert_eq!(policy.policy_name, "GDPR Compliance");
    assert_eq!(policy.policy_area, Some("data_storage".to_string()));

    // Test security policy retrieval
    let retrieved = container
        .context_crud_service
        .get_security_policy(&policy.id)
        .await;

    assert!(retrieved.is_ok(), "Security policy retrieval should succeed");
    assert!(retrieved.unwrap().is_some(), "Security policy should exist");

    // Test security policy update
    let mut updated_policy = policy.clone();
    updated_policy.compliance_notes = Some("GDPR and CCPA compliant".to_string());

    let update_result = container
        .context_crud_service
        .update_security_policy(&updated_policy)
        .await;

    assert!(
        update_result.is_ok(),
        "Security policy update should succeed"
    );

    // Test security policy listing
    let policies = container
        .context_crud_service
        .list_security_policies(&project.id)
        .await;

    assert!(policies.is_ok(), "Security policy listing should succeed");
    assert_eq!(policies.unwrap().len(), 1, "Should have one security policy");

    // Test security policy deletion
    let deleted = container
        .context_crud_service
        .delete_security_policy(&policy.id)
        .await;

    assert!(deleted.is_ok(), "Security policy deletion should succeed");
    assert!(deleted.unwrap(), "Security policy should be deleted");
}

#[tokio::test]
async fn test_feature_context_crud_operations() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Initialize database and container
    init_db(db_path_str).unwrap();
    let container = AppContainer::new(db_path_str).unwrap();

    // Create a test project first
    let project = container
        .project_service
        .create_project("Test Project", Some("A test project"), None)
        .await
        .unwrap();

    // Test feature context creation
    let feature_ctx = container
        .context_crud_service
        .create_feature_context(
            &project.id,
            "User Authentication",
            Some("Allow users to securely login"),
            Some("Admin, User, Guest"),
            Some("Login flow, Password reset"),
            Some("OAuth 2.0, Session management"),
            Some("Invalid credentials, Account lockout"),
        )
        .await;

    assert!(feature_ctx.is_ok(), "Feature context creation should succeed");
    let feature_ctx = feature_ctx.unwrap();
    assert_eq!(feature_ctx.feature_name, "User Authentication");

    // Test feature context retrieval
    let retrieved = container
        .context_crud_service
        .get_feature_context(&feature_ctx.id)
        .await;

    assert!(retrieved.is_ok(), "Feature context retrieval should succeed");
    assert!(retrieved.unwrap().is_some(), "Feature context should exist");

    // Test feature context update
    let mut updated_ctx = feature_ctx.clone();
    updated_ctx.business_purpose = Some("Secure user authentication and session management".to_string());

    let update_result = container
        .context_crud_service
        .update_feature_context(&updated_ctx)
        .await;

    assert!(
        update_result.is_ok(),
        "Feature context update should succeed"
    );

    // Test feature context listing
    let contexts = container
        .context_crud_service
        .list_feature_contexts(&project.id)
        .await;

    assert!(contexts.is_ok(), "Feature context listing should succeed");
    assert_eq!(contexts.unwrap().len(), 1, "Should have one feature context");

    // Test feature context deletion
    let deleted = container
        .context_crud_service
        .delete_feature_context(&feature_ctx.id)
        .await;

    assert!(deleted.is_ok(), "Feature context deletion should succeed");
    assert!(deleted.unwrap(), "Feature context should be deleted");
}

#[tokio::test]
async fn test_development_phase_crud_operations() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Initialize database and container
    init_db(db_path_str).unwrap();
    let container = AppContainer::new(db_path_str).unwrap();

    // Create a test project first
    let project = container
        .project_service
        .create_project("Test Project", Some("A test project"), None)
        .await
        .unwrap();

    // Test development phase creation
    let phase = container
        .development_phase_service
        .create_phase(
            &project.id,
            "Setup Phase",
            1,
            Some("Initial project setup and configuration"),
        )
        .await;

    assert!(phase.is_ok(), "Development phase creation should succeed");
    let phase = phase.unwrap();
    assert_eq!(phase.phase_name, "Setup Phase");
    assert_eq!(phase.phase_order, 1);

    // Test development phase retrieval
    let retrieved = container
        .development_phase_service
        .get_phase(&phase.id)
        .await;

    assert!(retrieved.is_ok(), "Development phase retrieval should succeed");
    assert!(retrieved.unwrap().is_some(), "Development phase should exist");

    // Test development phase update
    let mut updated_phase = phase.clone();
    updated_phase.description = Some("Initial setup, configuration, and scaffolding".to_string());

    let update_result = container
        .development_phase_service
        .update_phase(&updated_phase)
        .await;

    assert!(
        update_result.is_ok(),
        "Development phase update should succeed"
    );

    // Test development phase listing
    let phases = container
        .development_phase_service
        .list_phases(&project.id)
        .await;

    assert!(phases.is_ok(), "Development phase listing should succeed");
    assert_eq!(phases.unwrap().len(), 1, "Should have one development phase");

    // Test development phase deletion
    let deleted = container
        .development_phase_service
        .delete_phase(&phase.id)
        .await;

    assert!(deleted.is_ok(), "Development phase deletion should succeed");
    assert!(deleted.unwrap(), "Development phase should be deleted");
}

#[tokio::test]
async fn test_combined_crud_workflow() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    // Initialize database and container
    init_db(db_path_str).unwrap();
    let container = AppContainer::new(db_path_str).unwrap();

    // Create a test project
    let project = container
        .project_service
        .create_project("E-Commerce App", Some("Full featured e-commerce platform"), None)
        .await
        .unwrap();

    // Create business rules
    let rule1 = container
        .context_crud_service
        .create_business_rule(
            &project.id,
            "Payment Processing",
            Some("All payments must be processed securely"),
            Some("payments"),
        )
        .await
        .unwrap();

    let rule2 = container
        .context_crud_service
        .create_business_rule(
            &project.id,
            "Inventory Management",
            Some("Track stock levels in real-time"),
            Some("inventory"),
        )
        .await
        .unwrap();

    // Create architectural decisions
    let decision1 = container
        .context_crud_service
        .create_architectural_decision(
            &project.id,
            "Microservices Architecture",
            Some("Need scalability and independent deployment"),
            Some("Use microservices for different domains"),
        )
        .await
        .unwrap();

    // Create performance requirements
    let perf_1 = container
        .context_crud_service
        .create_performance_requirement(
            &project.id,
            Some("Checkout Process"),
            Some("response_time"),
            Some("< 200ms"),
            None,
            None,
        )
        .await
        .unwrap();

    // Create security policies
    let policy1 = container
        .context_crud_service
        .create_security_policy(
            &project.id,
            "Payment Data Protection",
            Some("data_storage"),
        )
        .await
        .unwrap();

    // Create development phases
    let phase1 = container
        .development_phase_service
        .create_phase(&project.id, "Design Phase", 1, None)
        .await
        .unwrap();

    let phase2 = container
        .development_phase_service
        .create_phase(&project.id, "Implementation", 2, None)
        .await
        .unwrap();

    // Verify all items were created
    let all_rules = container
        .context_crud_service
        .list_business_rules(&project.id)
        .await
        .unwrap();
    assert_eq!(all_rules.len(), 2, "Should have two business rules");

    let all_decisions = container
        .context_crud_service
        .list_architectural_decisions(&project.id)
        .await
        .unwrap();
    assert_eq!(all_decisions.len(), 1, "Should have one architectural decision");

    let all_perfs = container
        .context_crud_service
        .list_performance_requirements(&project.id)
        .await
        .unwrap();
    assert_eq!(all_perfs.len(), 1, "Should have one performance requirement");

    let all_policies = container
        .context_crud_service
        .list_security_policies(&project.id)
        .await
        .unwrap();
    assert_eq!(all_policies.len(), 1, "Should have one security policy");

    let all_phases = container
        .development_phase_service
        .list_phases(&project.id)
        .await
        .unwrap();
    assert_eq!(all_phases.len(), 2, "Should have two development phases");

    // Update some items
    let mut updated_rule = rule1.clone();
    updated_rule.implementation_pattern = Some("Use Stripe payment gateway".to_string());
    container
        .context_crud_service
        .update_business_rule(&updated_rule)
        .await
        .unwrap();

    let mut updated_phase = phase1.clone();
    updated_phase.description = Some("Design database schema and API contracts".to_string());
    container
        .development_phase_service
        .update_phase(&updated_phase)
        .await
        .unwrap();

    // Delete some items
    let deleted_rule = container
        .context_crud_service
        .delete_business_rule(&rule2.id)
        .await
        .unwrap();
    assert!(deleted_rule, "Rule should be deleted");

    let remaining_rules = container
        .context_crud_service
        .list_business_rules(&project.id)
        .await
        .unwrap();
    assert_eq!(remaining_rules.len(), 1, "Should have one business rule remaining");

    // Clean up project
    let deleted_project = container.project_service.delete_project(&project.id).await.unwrap();
    assert!(deleted_project, "Project should be deleted");
}
