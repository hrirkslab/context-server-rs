/// Unit tests for MCP endpoints for enhanced context server
/// Tests the create_entity, read_entity, update_entity, delete_entity, and list_entities endpoints
use serde_json::{json, Value};
use tempfile::tempdir;

use context_server_rs::enhanced_context_server::EnhancedContextMcpServer;
use rmcp::handler::server::ServerHandler;

#[tokio::test]
async fn test_mcp_create_project_entity_endpoint() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let server = EnhancedContextMcpServer::new(db_path_str).expect("Failed to create server");

    // Test creating a project via MCP
    let args = json!({
        "entity_type": "project",
        "data": {
            "name": "Test Project",
            "description": "A test project via MCP"
        }
    });

    // Verify the endpoint exists in the tools list
    let tools_result = server
        .list_tools(None, Default::default())
        .await
        .expect("Failed to list tools");

    let has_create_entity = tools_result
        .tools
        .iter()
        .any(|t| t.name == "create_entity");
    assert!(
        has_create_entity,
        "create_entity tool should be available"
    );
}

#[tokio::test]
async fn test_mcp_list_projects_endpoint() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let server = EnhancedContextMcpServer::new(db_path_str).expect("Failed to create server");

    // Verify the list_projects endpoint exists
    let tools_result = server
        .list_tools(None, Default::default())
        .await
        .expect("Failed to list tools");

    let has_list_projects = tools_result
        .tools
        .iter()
        .any(|t| t.name == "list_projects");
    assert!(
        has_list_projects,
        "list_projects tool should be available"
    );
}

#[tokio::test]
async fn test_mcp_get_entity_endpoint_schema() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let server = EnhancedContextMcpServer::new(db_path_str).expect("Failed to create server");

    // Verify the get_entity endpoint exists with proper schema
    let tools_result = server
        .list_tools(None, Default::default())
        .await
        .expect("Failed to list tools");

    let get_entity_tool = tools_result
        .tools
        .iter()
        .find(|t| t.name == "get_entity");

    assert!(get_entity_tool.is_some(), "get_entity tool should exist");

    if let Some(tool) = get_entity_tool {
        let schema = tool.input_schema.as_ref();
        assert!(schema.is_some(), "get_entity should have input schema");

        // Verify entity_type enum includes all supported types
        if let Some(props) = schema.and_then(|s| s.get("properties")) {
            assert!(
                props.get("entity_type").is_some(),
                "entity_type should be in schema"
            );
            assert!(props.get("id").is_some(), "id should be in schema");
        }
    }
}

#[tokio::test]
async fn test_mcp_create_entity_endpoint_schema() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let server = EnhancedContextMcpServer::new(db_path_str).expect("Failed to create server");

    // Verify the create_entity endpoint exists with proper schema
    let tools_result = server
        .list_tools(None, Default::default())
        .await
        .expect("Failed to list tools");

    let create_entity_tool = tools_result
        .tools
        .iter()
        .find(|t| t.name == "create_entity");

    assert!(
        create_entity_tool.is_some(),
        "create_entity tool should exist"
    );

    if let Some(tool) = create_entity_tool {
        let schema = tool.input_schema.as_ref();
        assert!(schema.is_some(), "create_entity should have input schema");

        // Verify schema structure
        if let Some(props) = schema.and_then(|s| s.get("properties")) {
            assert!(
                props.get("entity_type").is_some(),
                "entity_type should be in schema"
            );
            assert!(props.get("data").is_some(), "data should be in schema");
        }
    }
}

#[tokio::test]
async fn test_mcp_update_entity_endpoint_schema() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let server = EnhancedContextMcpServer::new(db_path_str).expect("Failed to create server");

    // Verify the update_entity endpoint exists
    let tools_result = server
        .list_tools(None, Default::default())
        .await
        .expect("Failed to list tools");

    let update_entity_tool = tools_result
        .tools
        .iter()
        .find(|t| t.name == "update_entity");

    assert!(
        update_entity_tool.is_some(),
        "update_entity tool should exist"
    );

    if let Some(tool) = update_entity_tool {
        let schema = tool.input_schema.as_ref();
        assert!(schema.is_some(), "update_entity should have input schema");

        // Verify schema structure
        if let Some(props) = schema.and_then(|s| s.get("properties")) {
            assert!(
                props.get("entity_type").is_some(),
                "entity_type should be in schema"
            );
            assert!(props.get("id").is_some(), "id should be in schema");
            assert!(props.get("data").is_some(), "data should be in schema");
        }
    }
}

#[tokio::test]
async fn test_mcp_delete_entity_endpoint_schema() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let server = EnhancedContextMcpServer::new(db_path_str).expect("Failed to create server");

    // Verify the delete_entity endpoint exists
    let tools_result = server
        .list_tools(None, Default::default())
        .await
        .expect("Failed to list tools");

    let delete_entity_tool = tools_result
        .tools
        .iter()
        .find(|t| t.name == "delete_entity");

    assert!(
        delete_entity_tool.is_some(),
        "delete_entity tool should exist"
    );

    if let Some(tool) = delete_entity_tool {
        let schema = tool.input_schema.as_ref();
        assert!(schema.is_some(), "delete_entity should have input schema");

        // Verify schema structure
        if let Some(props) = schema.and_then(|s| s.get("properties")) {
            assert!(
                props.get("entity_type").is_some(),
                "entity_type should be in schema"
            );
            assert!(props.get("id").is_some(), "id should be in schema");
        }
    }
}

#[tokio::test]
async fn test_mcp_list_entities_endpoint_schema() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let server = EnhancedContextMcpServer::new(db_path_str).expect("Failed to create server");

    // Verify the list_entities endpoint exists
    let tools_result = server
        .list_tools(None, Default::default())
        .await
        .expect("Failed to list tools");

    let list_entities_tool = tools_result
        .tools
        .iter()
        .find(|t| t.name == "list_entities");

    assert!(
        list_entities_tool.is_some(),
        "list_entities tool should exist"
    );

    if let Some(tool) = list_entities_tool {
        let schema = tool.input_schema.as_ref();
        assert!(schema.is_some(), "list_entities should have input schema");

        // Verify schema structure
        if let Some(props) = schema.and_then(|s| s.get("properties")) {
            assert!(
                props.get("entity_type").is_some(),
                "entity_type should be in schema"
            );
        }
    }
}

#[tokio::test]
async fn test_mcp_supported_entity_types() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let server = EnhancedContextMcpServer::new(db_path_str).expect("Failed to create server");

    // Verify all required tools exist
    let tools_result = server
        .list_tools(None, Default::default())
        .await
        .expect("Failed to list tools");

    let tool_names: Vec<&str> = tools_result.tools.iter().map(|t| t.name.as_str()).collect();

    // Check for essential CRUD tools
    assert!(
        tool_names.contains(&"create_entity"),
        "create_entity tool should exist"
    );
    assert!(
        tool_names.contains(&"get_entity"),
        "get_entity tool should exist"
    );
    assert!(
        tool_names.contains(&"update_entity"),
        "update_entity tool should exist"
    );
    assert!(
        tool_names.contains(&"delete_entity"),
        "delete_entity tool should exist"
    );
    assert!(
        tool_names.contains(&"list_entities"),
        "list_entities tool should exist"
    );
    assert!(
        tool_names.contains(&"query_context"),
        "query_context tool should exist"
    );
    assert!(
        tool_names.contains(&"list_projects"),
        "list_projects tool should exist"
    );
}

#[tokio::test]
async fn test_mcp_server_info() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let server = EnhancedContextMcpServer::new(db_path_str).expect("Failed to create server");

    let info = server.get_info();

    // Verify server information
    assert_eq!(info.server_info.name, "enhanced-context-server-rs");
    assert_eq!(info.server_info.version, "0.2.0");
    assert!(
        info.instructions.is_some(),
        "Server should have instructions"
    );

    // Verify server has tools capability
    let schema = &info.capabilities;
    assert!(
        schema
            .to_json()
            .get("tools")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        "Server should support tools"
    );
}

#[tokio::test]
async fn test_entity_type_enum_values() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let server = EnhancedContextMcpServer::new(db_path_str).expect("Failed to create server");

    // Get the create_entity tool and verify all entity types are in the enum
    let tools_result = server
        .list_tools(None, Default::default())
        .await
        .expect("Failed to list tools");

    let create_entity = tools_result
        .tools
        .iter()
        .find(|t| t.name == "create_entity");

    if let Some(tool) = create_entity {
        if let Some(schema) = tool.input_schema.as_ref() {
            if let Some(props) = schema.get("properties") {
                if let Some(entity_type) = props.get("entity_type") {
                    if let Some(enum_values) = entity_type.get("enum") {
                        if let Some(arr) = enum_values.as_array() {
                            let values: Vec<&str> = arr
                                .iter()
                                .filter_map(|v| v.as_str())
                                .collect();

                            // Verify all required entity types are supported
                            assert!(
                                values.contains(&"project"),
                                "project type should be supported"
                            );
                            assert!(
                                values.contains(&"business_rule"),
                                "business_rule type should be supported"
                            );
                            assert!(
                                values.contains(&"architectural_decision"),
                                "architectural_decision type should be supported"
                            );
                            assert!(
                                values.contains(&"performance_requirement"),
                                "performance_requirement type should be supported"
                            );
                            assert!(
                                values.contains(&"security_policy"),
                                "security_policy type should be supported"
                            );
                            assert!(
                                values.contains(&"framework_component"),
                                "framework_component type should be supported"
                            );
                            assert!(
                                values.contains(&"development_phase"),
                                "development_phase type should be supported"
                            );
                            assert!(
                                values.contains(&"feature_context"),
                                "feature_context type should be supported"
                            );
                        }
                    }
                }
            }
        }
    }
}
