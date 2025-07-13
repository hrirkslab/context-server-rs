use tempfile::tempdir;

use context_server_rs::db::init::init_db;
use context_server_rs::container::AppContainer;

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
    assert!(container_result.is_ok(), "AppContainer creation should succeed");
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
    let project = container.project_service
        .create_project("Test Project", Some("A test project"), None)
        .await;
    
    assert!(project.is_ok(), "Project creation should succeed");
    let project = project.unwrap();
    assert_eq!(project.name, "Test Project");
    assert_eq!(project.description, Some("A test project".to_string()));
    
    // Test project retrieval
    let retrieved = container.project_service
        .get_project(&project.id)
        .await;
    
    assert!(retrieved.is_ok(), "Project retrieval should succeed");
    let retrieved = retrieved.unwrap();
    assert!(retrieved.is_some(), "Project should exist");
    
    // Test project deletion
    let deleted = container.project_service
        .delete_project(&project.id)
        .await;
    
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
    let project = container.project_service
        .create_project("Test Project", Some("A test project"), None)
        .await
        .unwrap();
    
    // Test component creation using framework_service (was component_service)
    let component = container.framework_service
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
    let retrieved = container.framework_service
        .get_component(&component.id)
        .await;
    
    assert!(retrieved.is_ok(), "Component retrieval should succeed");
    assert!(retrieved.unwrap().is_some(), "Component should exist");
    
    // Test listing components by project using framework_service (was component_service)
    let components = container.framework_service
        .list_components(&project.id)
        .await;
    
    assert!(components.is_ok(), "Component listing should succeed");
    assert_eq!(components.unwrap().len(), 1, "Should have one component");
}
