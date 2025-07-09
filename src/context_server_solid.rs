use rmcp::{
    model::*,
    model::ErrorData as McpError,
    handler::server::ServerHandler,
};
use crate::models::flutter::*;
use crate::container::AppContainer;
use anyhow::Result;

/// MCP Context Server following SOLID principles
/// 
/// This server now delegates all business logic to appropriate services,
/// following the Single Responsibility Principle (SRP)
pub struct ContextMcpServer {
    container: AppContainer,
}

impl ContextMcpServer {
    /// Create a new ContextMcpServer with dependency injection
    /// Unused - kept for reference only
    #[allow(dead_code)]
    pub fn new(db_path: &str) -> Result<Self> {
        let container = AppContainer::new(db_path)?;
        Ok(Self { container })
    }

    /// Get server capabilities and available features
    async fn get_server_capabilities(&self) -> Result<ServerCapabilitiesInfo, McpError> {
        Ok(ServerCapabilitiesInfo {
            server_info: ServerMetadata {
                name: "context-server-rs".to_string(),
                version: "0.1.0".to_string(),
                description: "SOLID-architected Flutter-specific MCP Context Server for AI-assisted development".to_string(),
                config_directory: "~/config/context-server-rs/".to_string(),
            },
            features: vec![
                FeatureInfo {
                    name: "SOLID Architecture Implementation".to_string(),
                    description: "Follows Single Responsibility, Open/Closed, Liskov Substitution, Interface Segregation, and Dependency Inversion principles".to_string(),
                    status: FeatureStatus::Implemented,
                    tools: vec!["dependency_injection".to_string(), "service_layer".to_string(), "repository_pattern".to_string()],
                },
                FeatureInfo {
                    name: "Flutter Clean Architecture Enforcement".to_string(),
                    description: "Track components by architecture layer and validate dependency rules".to_string(),
                    status: FeatureStatus::Implemented,
                    tools: vec!["create_flutter_component".to_string(), "list_flutter_components".to_string(), "validate_architecture".to_string()],
                },
                FeatureInfo {
                    name: "Development Phase Tracking".to_string(),
                    description: "Manage project phases with order, dependencies, and completion criteria".to_string(),
                    status: FeatureStatus::Implemented,
                    tools: vec!["create_development_phase".to_string(), "list_development_phases".to_string()],
                },
                FeatureInfo {
                    name: "Context Query Service".to_string(),
                    description: "Extensible context querying with proper separation of concerns".to_string(),
                    status: FeatureStatus::Implemented,
                    tools: vec!["query_context".to_string()],
                },
            ],
            database_tables: vec![
                TableInfo {
                    name: "projects".to_string(),
                    description: "Core project information and metadata".to_string(),
                    primary_fields: vec!["id".to_string(), "name".to_string(), "description".to_string()],
                    example_use: "Store basic project info for LocalChat Flutter app".to_string(),
                },
                TableInfo {
                    name: "flutter_components".to_string(),
                    description: "Flutter widgets, providers, services tracked by architecture layer".to_string(),
                    primary_fields: vec!["component_name".to_string(), "component_type".to_string(), "architecture_layer".to_string()],
                    example_use: "Track ChatScreen widget in presentation layer".to_string(),
                },
                TableInfo {
                    name: "development_phases".to_string(),
                    description: "Project phases with order, status, and completion criteria".to_string(),
                    primary_fields: vec!["phase_name".to_string(), "phase_order".to_string(), "status".to_string()],
                    example_use: "Track Setup → Chat UI → Model Management → Polish phases".to_string(),
                },
            ],
            mcp_tools: vec![
                ToolInfo {
                    name: "query_context".to_string(),
                    description: "Query project context by feature area and task type using service layer".to_string(),
                    category: "Core".to_string(),
                    required_params: vec!["project_id".to_string(), "feature_area".to_string(), "task_type".to_string()],
                    example_use: "Get authentication context for implementation".to_string(),
                },
                ToolInfo {
                    name: "create_flutter_component".to_string(),
                    description: "Create Flutter component using service layer with proper validation".to_string(),
                    category: "Flutter".to_string(),
                    required_params: vec!["project_id".to_string(), "component_name".to_string(), "component_type".to_string(), "architecture_layer".to_string()],
                    example_use: "Create ChatScreen widget in presentation layer".to_string(),
                },
                ToolInfo {
                    name: "validate_architecture".to_string(),
                    description: "Check for Clean Architecture dependency violations using dedicated service".to_string(),
                    category: "Flutter".to_string(),
                    required_params: vec!["project_id".to_string()],
                    example_use: "Ensure presentation layer doesn't import data layer".to_string(),
                },
            ],
            usage_examples: vec![
                UsageExample {
                    scenario: "SOLID Architecture Benefits".to_string(),
                    steps: vec![
                        "Single Responsibility: Each service handles one concern".to_string(),
                        "Open/Closed: Easy to extend with new validators or repositories".to_string(),
                        "Liskov Substitution: Can swap repository implementations".to_string(),
                        "Interface Segregation: Focused, minimal interfaces".to_string(),
                        "Dependency Inversion: High-level modules don't depend on low-level modules".to_string(),
                    ],
                },
            ],
            recommended_workflow: vec![
                "1. Services are injected via AppContainer (DI)".to_string(),
                "2. Each operation delegates to appropriate service".to_string(),
                "3. Services use repository interfaces (not concrete implementations)".to_string(),
                "4. Easy to test, extend, and maintain".to_string(),
            ],
        })
    }
}

impl ServerHandler for ContextMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: "context-server-rs".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: Some("SOLID-architected Context Server for AI Code Generation. Provides curated project context with proper separation of concerns, dependency injection, and extensible architecture.".to_string()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        tracing::debug!("Received list_tools request");
        
        let tools = vec![
            Tool {
                name: "query_context".into(),
                description: Some("Query project context using service layer architecture".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {
                            "type": "string",
                            "description": "The ID of the project"
                        },
                        "feature_area": {
                            "type": "string",
                            "description": "The feature area (e.g., 'authentication', 'user_interface', 'payments')"
                        },
                        "task_type": {
                            "type": "string",
                            "description": "The type of task ('implement', 'fix', 'optimize')"
                        },
                        "components": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "List of components involved"
                        }
                    },
                    "required": ["project_id", "feature_area", "task_type", "components"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "list_projects".into(),
                description: Some("List all available projects using project service".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {}
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "create_project".into(),
                description: Some("Create a new project using project service".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "The name of the project"
                        },
                        "description": {
                            "type": "string",
                            "description": "Optional description of the project"
                        },
                        "repository_url": {
                            "type": "string",
                            "description": "Optional repository URL"
                        }
                    },
                    "required": ["name"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "create_flutter_component".into(),
                description: Some("Create a new Flutter component using service layer".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {
                            "type": "string",
                            "description": "The ID of the project"
                        },
                        "component_name": {
                            "type": "string",
                            "description": "The name of the component"
                        },
                        "component_type": {
                            "type": "string",
                            "enum": ["widget", "provider", "service", "repository", "model", "utility"],
                            "description": "The type of component"
                        },
                        "architecture_layer": {
                            "type": "string",
                            "enum": ["presentation", "domain", "data", "core"],
                            "description": "The architecture layer where this component belongs"
                        },
                        "file_path": {
                            "type": "string",
                            "description": "Optional file path for the component"
                        }
                    },
                    "required": ["project_id", "component_name", "component_type", "architecture_layer"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "list_flutter_components".into(),
                description: Some("List all Flutter components using service layer".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {
                            "type": "string",
                            "description": "The ID of the project"
                        }
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "validate_architecture".into(),
                description: Some("Validate Flutter Clean Architecture rules using validation service".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {
                            "type": "string",
                            "description": "The ID of the project to validate"
                        }
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "get_server_capabilities".into(),
                description: Some("Get comprehensive information about SOLID architecture and server features".into()),
                input_schema: std::sync::Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {}
                }).as_object().unwrap().clone()),
                annotations: None,
            },
        ];

        tracing::debug!("Returning {} tools", tools.len());
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
        tracing::debug!("Received call_tool request for: {}", request.name);
        
        match request.name.as_ref() {
            "query_context" => {
                tracing::debug!("Processing query_context using ContextQueryService");
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("project_id is required", None))?;
                let feature_area = args.get("feature_area")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("feature_area is required", None))?;
                let task_type = args.get("task_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("task_type is required", None))?;
                let components = args.get("components")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
                    .unwrap_or_default();

                let result = self.container.context_query_service.query_context(project_id, feature_area, task_type, &components).await?;
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("query_context completed successfully via service layer");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "list_projects" => {
                tracing::debug!("Processing list_projects using ProjectService");
                let projects = self.container.project_service.list_projects().await?;
                let content = serde_json::to_string_pretty(&projects)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("list_projects completed successfully via service layer");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "create_project" => {
                tracing::debug!("Processing create_project using ProjectService");
                let args = request.arguments.unwrap_or_default();
                let name = args.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("name is required", None))?;
                let description = args.get("description").and_then(|v| v.as_str());
                let repository_url = args.get("repository_url").and_then(|v| v.as_str());

                let project = self.container.project_service.create_project(name, description, repository_url).await?;
                let content = serde_json::to_string_pretty(&project)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("create_project completed successfully via service layer");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "create_flutter_component" => {
                tracing::debug!("Processing create_flutter_component using FlutterService");
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("project_id is required", None))?;
                let component_name = args.get("component_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("component_name is required", None))?;
                let component_type = args.get("component_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("component_type is required", None))?;
                let architecture_layer = args.get("architecture_layer")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("architecture_layer is required", None))?;
                let file_path = args.get("file_path").and_then(|v| v.as_str());

                let component = self.container.flutter_service.create_component(project_id, component_name, component_type, architecture_layer, file_path).await?;
                let content = serde_json::to_string_pretty(&component)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("create_flutter_component completed successfully via service layer");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "list_flutter_components" => {
                tracing::debug!("Processing list_flutter_components using FlutterService");
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("project_id is required", None))?;

                let components = self.container.flutter_service.list_components(project_id).await?;
                let content = serde_json::to_string_pretty(&components)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("list_flutter_components completed successfully via service layer");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "validate_architecture" => {
                tracing::debug!("Processing validate_architecture using ArchitectureValidationService");
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("project_id is required", None))?;

                let violations = self.container.architecture_validation_service.validate_architecture(project_id).await?;
                let content = serde_json::to_string_pretty(&violations)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("validate_architecture completed successfully via service layer");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "get_server_capabilities" => {
                tracing::debug!("Processing get_server_capabilities");
                
                let capabilities = self.get_server_capabilities().await?;
                let content = serde_json::to_string_pretty(&capabilities)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("get_server_capabilities completed successfully");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            _ => {
                tracing::warn!("Unknown tool requested: {}", request.name);
                Err(McpError::method_not_found::<CallToolRequestMethod>())
            }
        }
    }
}
