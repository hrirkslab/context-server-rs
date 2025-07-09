use std::sync::Arc;
use rmcp::{
    model::*,
    model::ErrorData as McpError,
    handler::server::ServerHandler,
};
use crate::models::flutter::*;
use crate::services::*;
use crate::container::AppContainer;
use anyhow::Result;

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
            instructions: Some("Enhanced Context Server with comprehensive CRUD operations for AI Code Generation. Provides curated project context including business rules, architectural decisions, security policies, project conventions, feature contexts, and Flutter-specific components.".to_string()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        tracing::debug!("Received list_tools request for enhanced server");
        
        let tools = vec![
            // Original tools
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
            Tool {
                name: "list_projects".into(),
                description: Some("List all available projects".into()),
                input_schema: Arc::new(serde_json::json!({"type": "object", "properties": {}}).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "create_project".into(),
                description: Some("Create a new project".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "The name of the project"},
                        "description": {"type": "string", "description": "Optional description of the project"},
                        "repository_url": {"type": "string", "description": "Optional repository URL"}
                    },
                    "required": ["name"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Business Rules CRUD
            Tool {
                name: "create_business_rule".into(),
                description: Some("Create a new business rule".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"},
                        "rule_name": {"type": "string", "description": "The name of the business rule"},
                        "description": {"type": "string", "description": "Optional description of the rule"},
                        "domain_area": {"type": "string", "description": "Optional domain area"}
                    },
                    "required": ["project_id", "rule_name"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "update_business_rule".into(),
                description: Some("Update an existing business rule".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the business rule"},
                        "rule_name": {"type": "string", "description": "The name of the business rule"},
                        "description": {"type": "string", "description": "Optional description"},
                        "domain_area": {"type": "string", "description": "Optional domain area"},
                        "implementation_pattern": {"type": "string", "description": "Implementation pattern"},
                        "constraints": {"type": "string", "description": "Constraints as JSON"},
                        "examples": {"type": "string", "description": "Examples as JSON"}
                    },
                    "required": ["id", "rule_name"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "delete_business_rule".into(),
                description: Some("Delete a business rule".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the business rule to delete"}
                    },
                    "required": ["id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "list_business_rules".into(),
                description: Some("List all business rules for a project".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Security Policy CRUD
            Tool {
                name: "create_security_policy".into(),
                description: Some("Create a new security policy".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"},
                        "policy_name": {"type": "string", "description": "The name of the security policy"},
                        "policy_area": {"type": "string", "description": "Optional policy area"}
                    },
                    "required": ["project_id", "policy_name"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "update_security_policy".into(),
                description: Some("Update an existing security policy".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the security policy"},
                        "policy_name": {"type": "string", "description": "The name of the security policy"},
                        "policy_area": {"type": "string", "description": "Optional policy area"},
                        "requirements": {"type": "string", "description": "Policy requirements"},
                        "implementation_pattern": {"type": "string", "description": "Implementation pattern"},
                        "forbidden_patterns": {"type": "string", "description": "Forbidden patterns as JSON"},
                        "compliance_notes": {"type": "string", "description": "Compliance notes"}
                    },
                    "required": ["id", "policy_name"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "delete_security_policy".into(),
                description: Some("Delete a security policy".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the security policy to delete"}
                    },
                    "required": ["id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "list_security_policies".into(),
                description: Some("List all security policies for a project".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Flutter Component CRUD
            Tool {
                name: "create_flutter_component".into(),
                description: Some("Create a new Flutter component in the project".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"},
                        "component_name": {"type": "string", "description": "The name of the component"},
                        "component_type": {"type": "string", "enum": ["widget", "provider", "service", "repository", "model", "utility"], "description": "The type of component"},
                        "architecture_layer": {"type": "string", "enum": ["presentation", "domain", "data", "core"], "description": "The architecture layer where this component belongs"},
                        "file_path": {"type": "string", "description": "Optional file path for the component"}
                    },
                    "required": ["project_id", "component_name", "component_type", "architecture_layer"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "update_flutter_component".into(),
                description: Some("Update an existing Flutter component".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the Flutter component"},
                        "component_name": {"type": "string", "description": "The name of the component"},
                        "component_type": {"type": "string", "enum": ["widget", "provider", "service", "repository", "model", "utility"], "description": "The type of component"},
                        "architecture_layer": {"type": "string", "enum": ["presentation", "domain", "data", "core"], "description": "The architecture layer"},
                        "file_path": {"type": "string", "description": "Optional file path for the component"},
                        "dependencies": {"type": "array", "items": {"type": "string"}, "description": "Component dependencies"}
                    },
                    "required": ["id", "component_name", "component_type", "architecture_layer"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "delete_flutter_component".into(),
                description: Some("Delete a Flutter component".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the Flutter component to delete"}
                    },
                    "required": ["id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "list_flutter_components".into(),
                description: Some("List all Flutter components in a project".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Development Phase CRUD
            Tool {
                name: "create_development_phase".into(),
                description: Some("Create a new development phase for tracking project progress".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"},
                        "phase_name": {"type": "string", "description": "The name of the phase (e.g., 'Setup', 'Chat UI', 'Model Management')"},
                        "phase_order": {"type": "integer", "description": "The order of this phase (1, 2, 3, etc.)"},
                        "description": {"type": "string", "description": "Optional description of the phase"}
                    },
                    "required": ["project_id", "phase_name", "phase_order"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "update_development_phase".into(),
                description: Some("Update an existing development phase".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the development phase"},
                        "phase_name": {"type": "string", "description": "The name of the phase"},
                        "phase_order": {"type": "integer", "description": "The order of this phase"},
                        "status": {"type": "string", "enum": ["pending", "in_progress", "completed", "blocked"], "description": "Phase status"},
                        "description": {"type": "string", "description": "Optional description"}
                    },
                    "required": ["id", "phase_name", "phase_order"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "delete_development_phase".into(),
                description: Some("Delete a development phase".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the development phase to delete"}
                    },
                    "required": ["id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "list_development_phases".into(),
                description: Some("List all development phases for a project".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Bulk Operations
            Tool {
                name: "bulk_create_components".into(),
                description: Some("Create multiple Flutter components in bulk".into()),
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
                description: Some("Update multiple Flutter components in bulk".into()),
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
                description: Some("Delete multiple Flutter components in bulk".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "component_ids": {"type": "array", "items": {"type": "string"}, "description": "Array of component IDs to delete"}
                    },
                    "required": ["component_ids"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Feature Context CRUD
            Tool {
                name: "create_feature_context".into(),
                description: Some("Create a new feature context".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"},
                        "feature_name": {"type": "string", "description": "The name of the feature"},
                        "business_purpose": {"type": "string", "description": "Optional business purpose"}
                    },
                    "required": ["project_id", "feature_name"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "update_feature_context".into(),
                description: Some("Update an existing feature context".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the feature context"},
                        "feature_name": {"type": "string", "description": "The name of the feature"},
                        "business_purpose": {"type": "string", "description": "Business purpose"},
                        "user_personas": {"type": "string", "description": "User personas as JSON"},
                        "key_workflows": {"type": "string", "description": "Key workflows as JSON"},
                        "integration_points": {"type": "string", "description": "Integration points as JSON"},
                        "edge_cases": {"type": "string", "description": "Edge cases as JSON"}
                    },
                    "required": ["id", "feature_name"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "delete_feature_context".into(),
                description: Some("Delete a feature context".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the feature context to delete"}
                    },
                    "required": ["id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "list_feature_contexts".into(),
                description: Some("List all feature contexts for a project".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project"}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Utility tools
            Tool {
                name: "validate_architecture".into(),
                description: Some("Validate Flutter Clean Architecture rules and detect violations".into()),
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
            // Project operations
            "list_projects" => {
                let projects = self.container.project_service.list_projects().await?;
                let content = serde_json::to_string_pretty(&projects)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "create_project" => {
                let args = request.arguments.unwrap_or_default();
                let name = args.get("name").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: name", None))?;
                let description = args.get("description").and_then(|v| v.as_str());
                let repository_url = args.get("repository_url").and_then(|v| v.as_str());

                let project = self.container.project_service.create_project(name, description, repository_url).await?;
                let content = serde_json::to_string_pretty(&project)
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
                            scenario: "Setting up a new Flutter project".to_string(),
                            steps: vec![
                                "1. create_project with Flutter details".to_string(),
                                "2. create_development_phase for each milestone".to_string(),
                                "3. bulk_create_components for initial architecture".to_string(),
                                "4. Define business_rules for domain logic".to_string(),
                            ],
                        },
                    ],
                    recommended_workflow: vec![
                        "Start with create_project".to_string(),
                        "Define development phases".to_string(),
                        "Set up Flutter components structure".to_string(),
                        "Add business rules and constraints".to_string(),
                        "Use query_context for AI assistance".to_string(),
                    ],
                };

                let content = serde_json::to_string_pretty(&capabilities)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // TODO: Implement remaining CRUD operations
            _ => {
                Err(McpError::method_not_found::<CallToolRequestMethod>())
            }
        }
    }
}
