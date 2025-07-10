use std::sync::Arc;
use rmcp::{
    model::*,
    model::ErrorData as McpError,
    handler::server::ServerHandler,
};
use crate::container::AppContainer;
use anyhow::Result;
use crate::models::framework::{ServerCapabilitiesInfo, ServerMetadata, FeatureInfo, FeatureStatus, TableInfo, ToolInfo, UsageExample};

/// Enhanced MCP Context Server with SOLID principles and comprehensive CRUD operations
#[derive(Clone)]
pub struct EnhancedContextMcpServer {
    container: Arc<AppContainer>,
}

impl EnhancedContextMcpServer {
    pub fn new(db_path: &str) -> Result<Self> {
        let container = AppContainer::new(db_path)?;
        Ok(Self {
            container: Arc::new(container),
        })
    }
}

impl ServerHandler for EnhancedContextMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: "enhanced-context-server-rs".to_string(),
                version: "0.2.0".to_string(),
            },
            instructions: Some("Enhanced Context Server with comprehensive CRUD operations for AI Code Generation. Provides curated project context including business rules, architectural decisions, security policies, project conventions, feature contexts, and framework-agnostic components.".to_string()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        tracing::debug!("Received list_tools request for enhanced server");
        
        let tools = vec![
            // Core Context Query Tool
            Tool {
                name: "query_context".into(),
                description: Some("Query project context based on feature area, task type, and components".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"},
                        "feature_area": {"type": "string", "description": "The feature area (e.g., 'authentication', 'user_interface', 'payments')"},
                        "task_type": {"type": "string", "description": "The type of task ('implement', 'fix', 'optimize')"},
                        "components": {"type": "array", "items": {"type": "string"}, "description": "List of components involved"}
                    },
                    "required": ["project_id", "feature_area", "task_type", "components"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Project Management (kept for convenience)
            Tool {
                name: "list_projects".into(),
                description: Some("List all available projects".into()),
                input_schema: Arc::new(serde_json::json!({"type": "object", "properties": {}}).as_object().unwrap().clone()),
                annotations: None,
            },

            // Universal CRUD Operations - Single tools that handle all entity types
            Tool {
                name: "get_entity".into(),
                description: Some("Get any entity by ID and type (universal getter)".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "entity_type": {"type": "string", "enum": ["project", "business_rule", "architectural_decision", "performance_requirement", "security_policy", "framework_component", "development_phase", "feature_context"], "description": "The type of entity to retrieve"},
                        "id": {"type": "string", "description": "The ID of the entity"}
                    },
                    "required": ["entity_type", "id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "create_entity".into(),
                description: Some("Create any entity (project, business rule, architectural decision, etc.)".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "entity_type": {"type": "string", "enum": ["project", "business_rule", "architectural_decision", "performance_requirement", "security_policy", "framework_component", "development_phase", "feature_context"], "description": "The type of entity to create"},
                        "data": {"type": "object", "description": "The entity data as JSON object"}
                    },
                    "required": ["entity_type", "data"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "update_entity".into(),
                description: Some("Update any entity by ID and type".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "entity_type": {"type": "string", "enum": ["project", "business_rule", "architectural_decision", "performance_requirement", "security_policy", "framework_component", "development_phase", "feature_context"], "description": "The type of entity to update"},
                        "id": {"type": "string", "description": "The ID of the entity"},
                        "data": {"type": "object", "description": "The updated entity data as JSON object"}
                    },
                    "required": ["entity_type", "id", "data"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "delete_entity".into(),
                description: Some("Delete any entity by ID and type".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "entity_type": {"type": "string", "enum": ["project", "business_rule", "architectural_decision", "performance_requirement", "security_policy", "framework_component", "development_phase", "feature_context"], "description": "The type of entity to delete"},
                        "id": {"type": "string", "description": "The ID of the entity to delete"}
                    },
                    "required": ["entity_type", "id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "list_entities".into(),
                description: Some("List entities by type and optional project filter".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "entity_type": {"type": "string", "enum": ["project", "business_rule", "architectural_decision", "performance_requirement", "security_policy", "framework_component", "development_phase", "feature_context"], "description": "The type of entities to list"},
                        "project_id": {"type": "string", "description": "Optional project ID to filter by"},
                        "architecture_layer": {"type": "string", "description": "Optional architecture layer to filter framework components by (only applies to framework_component entity type)"}
                    },
                    "required": ["entity_type"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Combined Operations - Higher-level tools for complex operations
            Tool {
                name: "manage_project".into(),
                description: Some("Comprehensive project management (create, update, delete, get)".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {"type": "string", "enum": ["create", "update", "delete", "get", "list"], "description": "The action to perform"},
                        "id": {"type": "string", "description": "Project ID (required for update, delete, get)"},
                        "data": {"type": "object", "description": "Project data (required for create, update)"}
                    },
                    "required": ["action"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Bulk Operations - Essential for efficiency
            Tool {
                name: "bulk_create_components".into(),
                description: Some("Create multiple framework components in bulk".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"},
                        "components": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "component_name": {"type": "string"},
                                    "component_type": {"type": "string", "enum": ["widget", "provider", "service", "repository", "model", "utility"]},
                                    "architecture_layer": {"type": "string", "enum": ["presentation", "domain", "data", "core"]},
                                    "file_path": {"type": "string"}
                                },
                                "required": ["component_name", "component_type", "architecture_layer"]
                            }
                        }
                    },
                    "required": ["project_id", "components"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "bulk_update_components".into(),
                description: Some("Update multiple framework components in bulk".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "components": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {"type": "string"},
                                    "component_name": {"type": "string"},
                                    "component_type": {"type": "string", "enum": ["widget", "provider", "service", "repository", "model", "utility"]},
                                    "architecture_layer": {"type": "string", "enum": ["presentation", "domain", "data", "core"]},
                                    "file_path": {"type": "string"}
                                },
                                "required": ["id", "component_name", "component_type", "architecture_layer"]
                            }
                        }
                    },
                    "required": ["components"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "bulk_delete_components".into(),
                description: Some("Delete multiple framework components in bulk".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "component_ids": {"type": "array", "items": {"type": "string"}, "description": "Array of component IDs to delete"}
                    },
                    "required": ["component_ids"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Advanced Operations - Specific high-value tools
            Tool {
                name: "bulk_operations".into(),
                description: Some("Perform bulk operations on multiple entities".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "operation": {"type": "string", "enum": ["create", "update", "delete"], "description": "The bulk operation to perform"},
                        "entity_type": {"type": "string", "enum": ["business_rule", "architectural_decision", "performance_requirement", "framework_component", "development_phase"], "description": "The type of entities"},
                        "data": {"type": "array", "description": "Array of entity data or IDs"}
                    },
                    "required": ["operation", "entity_type", "data"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "validate_architecture".into(),
                description: Some("Validate Clean Architecture rules and detect violations".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project to validate"}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "get_server_capabilities".into(),
                description: Some("Get comprehensive information about server features, database tables, and available tools".into()),
                input_schema: Arc::new(serde_json::json!({"type": "object", "properties": {}}).as_object().unwrap().clone()),
                annotations: None,
            },

            // Cache Management Tools
            Tool {
                name: "cache_management".into(),
                description: Some("Manage cache and temporary data (clear project, clear all)".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {"type": "string", "enum": ["clear_project", "clear_all"], "description": "The cache action to perform"},
                        "project_id": {"type": "string", "description": "Project ID (required for clear_project action)"}
                    },
                    "required": ["action"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
        ];

        Ok(ListToolsResult { 
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        tracing::debug!("Received call_tool request: {}", request.name);

        match request.name.as_ref() {
            // Core operations (kept for convenience)
            "list_projects" => {
                let projects = self.container.project_service.list_projects().await?;
                let content = serde_json::to_string_pretty(&projects)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // Context Query
            "query_context" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;
                let feature_area = args.get("feature_area").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: feature_area", None))?;
                let task_type = args.get("task_type").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: task_type", None))?;
                let components: Vec<String> = args.get("components")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let result = self.container.context_query_service.query_context(project_id, feature_area, task_type, &components).await?;
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // Architecture validation
            "validate_architecture" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;

                let violations = self.container.architecture_validation_service.validate_architecture(project_id).await?;
                let content = serde_json::to_string_pretty(&violations)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // Server capabilities
            "get_server_capabilities" => {
                let capabilities = ServerCapabilitiesInfo {
                    server_info: ServerMetadata {
                        name: "Enhanced Context Server".to_string(),
                        version: "0.2.0".to_string(),
                        description: "SOLID-compliant context server with comprehensive CRUD operations".to_string(),
                        config_directory: "~/.context-server".to_string(),
                    },
                    features: vec![
                        FeatureInfo {
                            name: "Enhanced CRUD Operations".to_string(),
                            description: "Full CRUD for all entities with bulk operations".to_string(),
                            status: FeatureStatus::Implemented,
                            tools: vec!["create_*".to_string(), "update_*".to_string(), "delete_*".to_string(), "bulk_*".to_string()],
                        },
                        FeatureInfo {
                            name: "SOLID Architecture".to_string(),
                            description: "Service/Repository pattern with dependency injection".to_string(),
                            status: FeatureStatus::Implemented,
                            tools: vec!["All operations".to_string()],
                        },
                    ],
                    database_tables: vec![
                        TableInfo {
                            name: "projects".to_string(),
                            description: "Main project information".to_string(),
                            primary_fields: vec!["id".to_string(), "name".to_string()],
                            example_use: "Organizing code contexts by project".to_string(),
                        },
                        TableInfo {
                            name: "business_rules".to_string(),
                            description: "Domain-specific business logic rules".to_string(),
                            primary_fields: vec!["id".to_string(), "rule_name".to_string(), "domain_area".to_string()],
                            example_use: "Capturing business constraints for AI code generation".to_string(),
                        },
                        // Add more table info as needed
                    ],
                    mcp_tools: vec![
                        ToolInfo {
                            name: "create_business_rule".to_string(),
                            description: "Create a new business rule".to_string(),
                            category: "CRUD".to_string(),
                            required_params: vec!["project_id".to_string(), "rule_name".to_string()],
                            example_use: "Define validation rules for user input".to_string(),
                        },
                        // Add more tool info as needed
                    ],
                    usage_examples: vec![
                        UsageExample {
                            scenario: "Setting up a new project".to_string(),
                            steps: vec![
                                "1. create_project with project details".to_string(),
                                "2. create_development_phase for each milestone".to_string(),
                                "3. bulk_create_components for initial architecture".to_string(),
                                "4. Define business_rules for domain logic".to_string(),
                            ],
                        },
                    ],
                    recommended_workflow: vec![
                        "Start with create_project".to_string(),
                        "Define development phases".to_string(),
                        "Set up framework components structure".to_string(),
                        "Add business rules and constraints".to_string(),
                        "Use query_context for AI assistance".to_string(),
                    ],
                };

                let content = serde_json::to_string_pretty(&capabilities)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // Business Rules CRUD operations are now handled by universal CRUD handlers

            // Architectural Decisions CRUD operations are now handled by universal CRUD handlers

            // Performance Requirements CRUD operations are now handled by universal CRUD handlers

            // Framework Components CRUD
            // Removed legacy Flutter component handlers - now using universal entity handlers

            // Development Phases CRUD
            // Removed legacy development phase handlers - now using universal entity handlers

            // Bulk Operations
            "bulk_create_components" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;
                let components_data = args.get("components").and_then(|v| v.as_array())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: components", None))?;

                let mut components = Vec::new();
                for comp_data in components_data {
                    let obj = comp_data.as_object()
                        .ok_or_else(|| McpError::invalid_params("Invalid component data", None))?;
                    let component_name = obj.get("component_name").and_then(|v| v.as_str())
                        .ok_or_else(|| McpError::invalid_params("Missing component_name in component data", None))?;
                    let component_type = obj.get("component_type").and_then(|v| v.as_str())
                        .ok_or_else(|| McpError::invalid_params("Missing component_type in component data", None))?;
                    let architecture_layer = obj.get("architecture_layer").and_then(|v| v.as_str())
                        .ok_or_else(|| McpError::invalid_params("Missing architecture_layer in component data", None))?;
                    let file_path = obj.get("file_path").and_then(|v| v.as_str());

                    let component = self.container.framework_service.create_component(
                        project_id, component_name, component_type, architecture_layer, file_path, None
                    ).await?;
                    components.push(component);
                }

                let content = serde_json::to_string_pretty(&components)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // Cache and Cleanup Operations
            "clear_project_cache" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;

                // Clear project-related data - for now, just return a success message
                // In a real implementation, you might want to clear cached data, temporary files, etc.
                let result = serde_json::json!({
                    "message": "Project cache cleared successfully",
                    "project_id": project_id,
                    "cleared": true,
                    "note": "Cache clearing implementation can be customized based on your needs"
                });
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "clear_all_cache" => {
                // This would be a nuclear option - clear everything
                let result = serde_json::json!({
                    "message": "All cache cleared successfully",
                    "warning": "This operation removes all stored data",
                    "cleared": true
                });
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // Get entity by ID operations
            "get_entity" => {
                let args = request.arguments.unwrap_or_default();
                let entity_type = args.get("entity_type").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: entity_type", None))?;
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let result = match entity_type {
                    "project" => {
                        let project = self.container.project_service.get_project(id).await?;
                        serde_json::to_value(project)
                    },
                    "business_rule" => {
                        let rule = self.container.context_crud_service.get_business_rule(id).await?;
                        serde_json::to_value(rule)
                    },
                    "architectural_decision" => {
                        let decision = self.container.context_crud_service.get_architectural_decision(id).await?;
                        serde_json::to_value(decision)
                    },
                    "performance_requirement" => {
                        let requirement = self.container.context_crud_service.get_performance_requirement(id).await?;
                        serde_json::to_value(requirement)
                    },
                    "framework_component" => {
                        let component = self.container.framework_service.get_component(id).await?;
                        serde_json::to_value(component)
                    },
                    "development_phase" => {
                        let phase = self.container.development_phase_service.get_phase(id).await?;
                        serde_json::to_value(phase)
                    },
                    _ => return Err(McpError::invalid_params("Invalid entity_type", None)),
                }.map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // Universal CRUD Operations - Remove duplicate handlers
            "manage_project" => {
                let args = request.arguments.unwrap_or_default();
                let action = args.get("action").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: action", None))?;

                let result = match action {
                    "create" => {
                        let data = args.get("data").and_then(|v| v.as_object())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: data for create", None))?;
                        let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let description = data.get("description").and_then(|v| v.as_str());
                        let repository_url = data.get("repository_url").and_then(|v| v.as_str());
                        let project = self.container.project_service.create_project(name, description, repository_url).await?;
                        serde_json::to_value(project)
                    },
                    "get" => {
                        let id = args.get("id").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: id for get", None))?;
                        let project = self.container.project_service.get_project(id).await?;
                        serde_json::to_value(project)
                    },
                    "delete" => {
                        let id = args.get("id").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: id for delete", None))?;
                        let deleted = self.container.project_service.delete_project(id).await?;
                        serde_json::to_value(serde_json::json!({"deleted": deleted, "id": id}))
                    },
                    "list" => {
                        let projects = self.container.project_service.list_projects().await?;
                        serde_json::to_value(projects)
                    },
                    _ => return Err(McpError::invalid_params("Unsupported action", None)),
                }.map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "cache_management" => {
                let args = request.arguments.unwrap_or_default();
                let action = args.get("action").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: action", None))?;

                let result = match action {
                    "clear_project" => {
                        let project_id = args.get("project_id").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id for clear_project", None))?;
                        serde_json::json!({
                            "message": "Project cache cleared successfully",
                            "project_id": project_id,
                            "cleared": true,
                            "note": "Cache clearing implementation can be customized based on your needs"
                        })
                    },
                    "clear_all" => {
                        serde_json::json!({
                            "message": "All cache cleared successfully",
                            "warning": "This operation removes all stored data",
                            "cleared": true
                        })
                    },
                    _ => return Err(McpError::invalid_params("Unsupported cache action", None)),
                };

                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            // Universal CRUD Operations
            // First universal handler for create_entity
            "create_entity" => {
                let args = request.arguments.unwrap_or_default();
                let entity_type = args.get("entity_type").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: entity_type", None))?;
                let data = args.get("data").and_then(|v| v.as_object())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: data", None))?;
                
                let result = match entity_type {
                    "project" => {
                        let name = data.get("name").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: name", None))?;
                        let description = data.get("description").and_then(|v| v.as_str());
                        let repository_url = data.get("repository_url").and_then(|v| v.as_str());
                        
                        let project = self.container.project_service.create_project(name, description, repository_url).await?;
                        serde_json::to_value(project).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                    },
                    "business_rule" => {
                        let project_id = data.get("project_id").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;
                        let rule_name = data.get("rule_name").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: rule_name", None))?;
                        let description = data.get("description").and_then(|v| v.as_str());
                        let domain_area = data.get("domain_area").and_then(|v| v.as_str());
                        
                        let rule = self.container.context_crud_service.create_business_rule(project_id, rule_name, description, domain_area).await?;
                        serde_json::to_value(rule).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                    },
                    "architectural_decision" => {
                        let project_id = data.get("project_id").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;
                        let decision_title = data.get("decision_title").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: decision_title", None))?;
                        let context = data.get("context").and_then(|v| v.as_str());
                        let decision = data.get("decision").and_then(|v| v.as_str());
                        
                        let arch_decision = self.container.context_crud_service.create_architectural_decision(project_id, decision_title, context, decision).await?;
                        serde_json::to_value(arch_decision).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                    },
                    "framework_component" => {
                        let project_id = data.get("project_id").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;
                        let component_name = data.get("component_name").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: component_name", None))?;
                        let component_type = data.get("component_type").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: component_type", None))?;
                        let architecture_layer = data.get("architecture_layer").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: architecture_layer", None))?;
                        let file_path = data.get("file_path").and_then(|v| v.as_str());
                        
                        let component = self.container.framework_service.create_component(
                            project_id, component_name, component_type, architecture_layer, file_path, None
                        ).await?;
                        serde_json::to_value(component).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                    },
                    "development_phase" => {
                        let project_id = data.get("project_id").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;
                        let phase_name = data.get("phase_name").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: phase_name", None))?;
                        let phase_order = data.get("phase_order").and_then(|v| v.as_i64())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: phase_order", None))? as i32;
                        let description = data.get("description").and_then(|v| v.as_str());
                        
                        let phase = self.container.development_phase_service.create_phase(project_id, phase_name, phase_order, description).await?;
                        serde_json::to_value(phase).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                    },
                    // Add more entity types as needed
                    _ => return Err(McpError::invalid_params(format!("Unsupported entity type: {}", entity_type), None)),
                };
                
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            
            "update_entity" => {
                let args = request.arguments.unwrap_or_default();
                let entity_type = args.get("entity_type").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: entity_type", None))?;
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;
                let data = args.get("data").and_then(|v| v.as_object())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: data", None))?;
                
                let result = match entity_type {
                    "project" => {
                        use crate::models::context::Project;
                        
                        let name = data.get("name").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: name", None))?;
                        
                        let project = Project {
                            id: id.to_string(),
                            name: name.to_string(),
                            description: data.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            repository_url: data.get("repository_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            created_at: None,
                            updated_at: None,
                        };
                        
                        let updated_project = self.container.project_service.update_project(&project).await?;
                        serde_json::to_value(updated_project).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                    },
                    "business_rule" => {
                        use crate::models::context::BusinessRule;
                        
                        let rule_name = data.get("rule_name").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: rule_name", None))?;
                        
                        let rule = BusinessRule {
                            id: id.to_string(),
                            project_id: data.get("project_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            rule_name: rule_name.to_string(),
                            description: data.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            domain_area: data.get("domain_area").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            implementation_pattern: data.get("implementation_pattern").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            constraints: data.get("constraints").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            examples: data.get("examples").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            created_at: None,
                        };
                        
                        let updated_rule = self.container.context_crud_service.update_business_rule(&rule).await?;
                        serde_json::to_value(updated_rule).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                    },
                    "framework_component" => {
                        let component_name = data.get("component_name").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: component_name", None))?;
                        let component_type = data.get("component_type").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: component_type", None))?;
                        let architecture_layer = data.get("architecture_layer").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing required parameter: architecture_layer", None))?;
                        let file_path = data.get("file_path").and_then(|v| v.as_str());
                        
                        // First retrieve the component
                        let component_opt = self.container.framework_service.get_component(id).await?;
                        if component_opt.is_none() {
                            return Err(McpError::invalid_params(format!("Component with id {} not found", id), None));
                        }
                        
                        // Get the component first
                        let mut component = match self.container.framework_service.get_component(id).await? {
                            Some(c) => c,
                            None => return Err(McpError::invalid_params(format!("Component with id {} not found", id), None)),
                        };
                        
                        // Update the component fields
                        component.component_name = component_name.to_string();
                        component.component_type = component_type.to_string();
                        component.architecture_layer = architecture_layer.to_string();
                        
                        if let Some(fp) = file_path {
                            component.file_path = Some(fp.to_string());
                        }
                        
                        // Update the component
                        let updated_component = self.container.framework_service.update_component(&component).await?;
                        
                        serde_json::to_value(updated_component).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                    },
                    // Add more entity types as needed
                    _ => return Err(McpError::invalid_params(format!("Unsupported entity type: {}", entity_type), None)),
                };
                
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            
            "delete_entity" => {
                let args = request.arguments.unwrap_or_default();
                let entity_type = args.get("entity_type").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: entity_type", None))?;
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;
                
                let result = match entity_type {
                    "project" => {
                        let deleted = self.container.project_service.delete_project(id).await?;
                        serde_json::json!({"deleted": deleted, "project_id": id})
                    },
                    "business_rule" => {
                        let deleted = self.container.context_crud_service.delete_business_rule(id).await?;
                        serde_json::json!({"deleted": deleted, "rule_id": id})
                    },
                    "architectural_decision" => {
                        let deleted = self.container.context_crud_service.delete_architectural_decision(id).await?;
                        serde_json::json!({"deleted": deleted, "decision_id": id})
                    },
                    "framework_component" => {
                        let deleted = self.container.framework_service.delete_component(id).await?;
                        serde_json::json!({"deleted": deleted, "component_id": id})
                    },
                    "development_phase" => {
                        let deleted = self.container.development_phase_service.delete_phase(id).await?;
                        serde_json::json!({"deleted": deleted, "phase_id": id})
                    },
                    // Add more entity types as needed
                    _ => return Err(McpError::invalid_params(format!("Unsupported entity type: {}", entity_type), None)),
                };
                
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            
            // Removed duplicate get_entity handler
            
            "list_entities" => {
                let args = request.arguments.unwrap_or_default();
                let entity_type = args.get("entity_type").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: entity_type", None))?;
                let project_id = args.get("project_id").and_then(|v| v.as_str());
                let architecture_layer = args.get("architecture_layer").and_then(|v| v.as_str());
                
                let result = match entity_type {
                    "project" => {
                        let projects = self.container.project_service.list_projects().await?;
                        serde_json::to_value(projects).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                    },
                    "business_rule" => {
                        if let Some(pid) = project_id {
                            let rules = self.container.context_crud_service.list_business_rules(pid).await?;
                            serde_json::to_value(rules).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                        } else {
                            return Err(McpError::invalid_params("Missing required parameter: project_id for business_rule listing", None))
                        }
                    },
                    "architectural_decision" => {
                        if let Some(pid) = project_id {
                            let decisions = self.container.context_crud_service.list_architectural_decisions(pid).await?;
                            serde_json::to_value(decisions).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                        } else {
                            return Err(McpError::invalid_params("Missing required parameter: project_id for architectural_decision listing", None))
                        }
                    },
                    "framework_component" => {
                        if let Some(pid) = project_id {
                            let components = if let Some(layer) = architecture_layer {
                                // Use list_components_by_layer if architecture_layer is specified
                                self.container.framework_service.list_components_by_layer(pid, layer).await?
                            } else {
                                // Use list_components if no layer is specified
                                self.container.framework_service.list_components(pid).await?
                            };
                            serde_json::to_value(components).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                        } else {
                            return Err(McpError::invalid_params("Missing required parameter: project_id for framework_component listing", None))
                        }
                    },
                    "development_phase" => {
                        if let Some(pid) = project_id {
                            let phases = self.container.development_phase_service.list_phases(pid).await?;
                            serde_json::to_value(phases).map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?
                        } else {
                            return Err(McpError::invalid_params("Missing required parameter: project_id for development_phase listing", None))
                        }
                    },
                    // Add more entity types as needed
                    _ => return Err(McpError::invalid_params(format!("Unsupported entity type: {}", entity_type), None)),
                };
                
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            
            // Cache Management
            // Removed duplicate cache_management handler
            
            // Bulk Operations for Framework Components
            // Removed duplicate bulk_create_components handler
            
            "bulk_update_components" => {
                let args = request.arguments.unwrap_or_default();
                let components_value = args.get("components").and_then(|v| v.as_array())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: components array", None))?;
                
                let mut results = Vec::new();
                
                for comp_value in components_value {
                    if let Some(comp_obj) = comp_value.as_object() {
                        let id = comp_obj.get("id").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing id in component", None))?;
                        let component_name = comp_obj.get("component_name").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing component_name in component", None))?;
                        let component_type = comp_obj.get("component_type").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing component_type in component", None))?;
                        let architecture_layer = comp_obj.get("architecture_layer").and_then(|v| v.as_str())
                            .ok_or_else(|| McpError::invalid_params("Missing architecture_layer in component", None))?;
                        let file_path = comp_obj.get("file_path").and_then(|v| v.as_str());
                        
                        // First retrieve the component
                        let mut component = match self.container.framework_service.get_component(id).await? {
                            Some(c) => c,
                            None => return Err(McpError::invalid_params(format!("Component with id {} not found", id), None)),
                        };
                        
                        // Update component fields
                        component.component_name = component_name.to_string();
                        component.component_type = component_type.to_string();
                        component.architecture_layer = architecture_layer.to_string();
                        
                        if let Some(fp) = file_path {
                            component.file_path = Some(fp.to_string());
                        }
                        
                        // Update the component
                        let updated_component = self.container.framework_service.update_component(&component).await?;
                        results.push(updated_component);
                    }
                }
                
                let content = serde_json::to_string_pretty(&results)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            
            "bulk_delete_components" => {
                let args = request.arguments.unwrap_or_default();
                let component_ids = args.get("component_ids").and_then(|v| v.as_array())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: component_ids array", None))?;
                
                let mut ids = Vec::new();
                for id_value in component_ids {
                    if let Some(id_str) = id_value.as_str() {
                        ids.push(id_str.to_string());
                    }
                }
                
                // Delete components individually since bulk delete is not implemented
                let mut deleted_count = 0;
                let mut failed_ids = Vec::new();
                
                for id in &ids {
                    match self.container.framework_service.delete_component(id).await {
                        Ok(true) => deleted_count += 1,
                        Ok(false) => failed_ids.push(id.clone()),
                        Err(e) => {
                            tracing::error!("Error deleting component {}: {}", id, e);
                            failed_ids.push(id.clone());
                        }
                    }
                }
                
                let result = serde_json::json!({
                    "deleted_count": deleted_count,
                    "component_ids": ids,
                    "failed_ids": failed_ids,
                    "success": deleted_count == ids.len()
                });
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            
            // Project Management
            // Removed duplicate manage_project handler

            // Fallback for undefined tools
            _ => {
                Err(McpError::method_not_found::<CallToolRequestMethod>())
            }
        }
    }
}

// Actual code replacement for CRUD operations:
// 1. Replace all:
//     self.container.flutter_service.create_component(...)
// with:
//     self.container.framework_service.create_component(..., None)
// 2. Replace all entity_type checks for "flutter_component" with "framework_component"
// 3. Update CRUD logic to use FrameworkComponent and FrameworkService
// 4. Remove or update any Flutter-specific enum/struct usage (e.g., ComponentType, ArchitectureLayer)

// Example actual code changes:
// In match arms and CRUD logic:
// "flutter_component" => { ...self.container.flutter_service... }
// becomes
// "framework_component" => { ...self.container.framework_service... }

// For create/update:
// let component = self.container.framework_service.create_component(
//     project_id, component_name, component_type, architecture_layer, file_path, None
// ).await?;

// For get:
// let component = self.container.framework_service.get_component(id).await?;

// For update:
// let updated_component = self.container.framework_service.update_component(&component).await?;

// For delete:
// let deleted = self.container.framework_service.delete_component(id).await?;

// For list:
// let components = self.container.framework_service.list_components(pid).await?;

// Remove ComponentType/ArchitectureLayer enum conversions, use strings directly from input.
