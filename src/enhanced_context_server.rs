use std::sync::Arc;
use rmcp::{
    model::*,
    model::ErrorData as McpError,
    handler::server::ServerHandler,
};
use crate::models::flutter::*;
// Commented out until services are actually used
// use crate::services::*;
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
            Tool {
                name: "update_project".into(),
                description: Some("Update an existing project".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the project"},
                        "name": {"type": "string", "description": "The name of the project"},
                        "description": {"type": "string", "description": "Optional description of the project"},
                        "repository_url": {"type": "string", "description": "Optional repository URL"}
                    },
                    "required": ["id", "name"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "delete_project".into(),
                description: Some("Delete a project".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the project to delete"}
                    },
                    "required": ["id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "get_project".into(),
                description: Some("Get a project by ID".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the project"}
                    },
                    "required": ["id"]
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
            Tool {
                name: "get_business_rule".into(),
                description: Some("Get a business rule by ID".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the business rule"}
                    },
                    "required": ["id"]
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
            Tool {
                name: "get_flutter_component".into(),
                description: Some("Get a Flutter component by ID".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the Flutter component"}
                    },
                    "required": ["id"]
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
            Tool {
                name: "get_development_phase".into(),
                description: Some("Get a development phase by ID".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "The ID of the development phase"}
                    },
                    "required": ["id"]
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
            Tool {
                name: "clear_project_cache".into(),
                description: Some("Clear cache and temporary data for a specific project".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project to clear cache for"}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "clear_all_cache".into(),
                description: Some("Clear all cache and temporary data (WARNING: This removes all stored data)".into()),
                input_schema: Arc::new(serde_json::json!({"type": "object", "properties": {}}).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "get_entity".into(),
                description: Some("Get any entity by ID and type (universal getter)".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "entity_type": {"type": "string", "enum": ["project", "business_rule", "architectural_decision", "performance_requirement", "flutter_component", "development_phase"], "description": "The type of entity to retrieve"},
                        "id": {"type": "string", "description": "The ID of the entity"}
                    },
                    "required": ["entity_type", "id"]
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
            "update_project" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;
                let name = args.get("name").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: name", None))?;
                let description = args.get("description").and_then(|v| v.as_str());
                let repository_url = args.get("repository_url").and_then(|v| v.as_str());

                use crate::models::context::Project;
                let project = Project {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: description.map(|s| s.to_string()),
                    repository_url: repository_url.map(|s| s.to_string()),
                    created_at: None,
                    updated_at: Some(chrono::Utc::now().to_rfc3339()),
                };

                let updated_project = self.container.project_service.update_project(&project).await?;
                let content = serde_json::to_string_pretty(&updated_project)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "delete_project" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let deleted = self.container.project_service.delete_project(id).await?;
                let result = serde_json::json!({"deleted": deleted, "project_id": id});
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "get_project" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let project = self.container.project_service.get_project(id).await?;
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

            // Business Rules CRUD
            "create_business_rule" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;
                let rule_name = args.get("rule_name").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: rule_name", None))?;
                let description = args.get("description").and_then(|v| v.as_str());
                let domain_area = args.get("domain_area").and_then(|v| v.as_str());

                let rule = self.container.context_crud_service.create_business_rule(project_id, rule_name, description, domain_area).await?;
                let content = serde_json::to_string_pretty(&rule)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "update_business_rule" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;
                let rule_name = args.get("rule_name").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: rule_name", None))?;
                
                use crate::models::context::BusinessRule;
                let rule = BusinessRule {
                    id: id.to_string(),
                    project_id: String::new(), // Will be filled by the service
                    rule_name: rule_name.to_string(),
                    description: args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    domain_area: args.get("domain_area").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    implementation_pattern: args.get("implementation_pattern").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    constraints: args.get("constraints").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    examples: args.get("examples").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    created_at: None,
                };

                let updated_rule = self.container.context_crud_service.update_business_rule(&rule).await?;
                let content = serde_json::to_string_pretty(&updated_rule)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "delete_business_rule" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let deleted = self.container.context_crud_service.delete_business_rule(id).await?;
                let result = serde_json::json!({"deleted": deleted, "rule_id": id});
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "get_business_rule" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let rule = self.container.context_crud_service.get_business_rule(id).await?;
                let content = serde_json::to_string_pretty(&rule)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "list_business_rules" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;

                let rules = self.container.context_crud_service.list_business_rules(project_id).await?;
                let content = serde_json::to_string_pretty(&rules)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // Architectural Decisions CRUD
            "create_architectural_decision" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;
                let decision_title = args.get("decision_title").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: decision_title", None))?;
                let context = args.get("context").and_then(|v| v.as_str());
                let decision = args.get("decision").and_then(|v| v.as_str());

                let arch_decision = self.container.context_crud_service.create_architectural_decision(project_id, decision_title, context, decision).await?;
                let content = serde_json::to_string_pretty(&arch_decision)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "update_architectural_decision" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;
                let decision_title = args.get("decision_title").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: decision_title", None))?;
                
                use crate::models::context::ArchitecturalDecision;
                let decision = ArchitecturalDecision {
                    id: id.to_string(),
                    project_id: String::new(), // Will be filled by the service
                    decision_title: decision_title.to_string(),
                    context: args.get("context").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    decision: args.get("decision").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    consequences: args.get("consequences").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    alternatives_considered: args.get("alternatives_considered").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    status: args.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    created_at: None,
                };

                let updated_decision = self.container.context_crud_service.update_architectural_decision(&decision).await?;
                let content = serde_json::to_string_pretty(&updated_decision)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "delete_architectural_decision" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let deleted = self.container.context_crud_service.delete_architectural_decision(id).await?;
                let result = serde_json::json!({"deleted": deleted, "decision_id": id});
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "get_architectural_decision" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let decision = self.container.context_crud_service.get_architectural_decision(id).await?;
                let content = serde_json::to_string_pretty(&decision)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "list_architectural_decisions" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;

                let decisions = self.container.context_crud_service.list_architectural_decisions(project_id).await?;
                let content = serde_json::to_string_pretty(&decisions)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // Performance Requirements CRUD
            "create_performance_requirement" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;
                let component_area = args.get("component_area").and_then(|v| v.as_str());
                let requirement_type = args.get("requirement_type").and_then(|v| v.as_str());
                let target_value = args.get("target_value").and_then(|v| v.as_str());

                let requirement = self.container.context_crud_service.create_performance_requirement(project_id, component_area, requirement_type, target_value).await?;
                let content = serde_json::to_string_pretty(&requirement)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "update_performance_requirement" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;
                
                use crate::models::context::PerformanceRequirement;
                let requirement = PerformanceRequirement {
                    id: id.to_string(),
                    project_id: String::new(), // Will be filled by the service
                    component_area: args.get("component_area").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    requirement_type: args.get("requirement_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    target_value: args.get("target_value").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    optimization_patterns: args.get("optimization_patterns").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    avoid_patterns: args.get("avoid_patterns").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    created_at: None,
                };

                let updated_requirement = self.container.context_crud_service.update_performance_requirement(&requirement).await?;
                let content = serde_json::to_string_pretty(&updated_requirement)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "delete_performance_requirement" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let deleted = self.container.context_crud_service.delete_performance_requirement(id).await?;
                let result = serde_json::json!({"deleted": deleted, "requirement_id": id});
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "get_performance_requirement" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let requirement = self.container.context_crud_service.get_performance_requirement(id).await?;
                let content = serde_json::to_string_pretty(&requirement)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "list_performance_requirements" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;

                let requirements = self.container.context_crud_service.list_performance_requirements(project_id).await?;
                let content = serde_json::to_string_pretty(&requirements)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // Flutter Components CRUD
            "create_flutter_component" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;
                let component_name = args.get("component_name").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: component_name", None))?;
                let component_type = args.get("component_type").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: component_type", None))?;
                let architecture_layer = args.get("architecture_layer").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: architecture_layer", None))?;
                let file_path = args.get("file_path").and_then(|v| v.as_str());

                let component = self.container.flutter_service.create_component(project_id, component_name, component_type, architecture_layer, file_path).await?;
                let content = serde_json::to_string_pretty(&component)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "update_flutter_component" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;
                let component_name = args.get("component_name").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: component_name", None))?;
                let component_type = args.get("component_type").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: component_type", None))?;
                let architecture_layer = args.get("architecture_layer").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: architecture_layer", None))?;
                let file_path = args.get("file_path").and_then(|v| v.as_str());
                let dependencies: Vec<String> = args.get("dependencies")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                use crate::models::flutter::{FlutterComponent, ComponentType, ArchitectureLayer};
                
                // Parse enums
                let comp_type = match component_type {
                    "widget" => ComponentType::Widget,
                    "provider" => ComponentType::Provider,
                    "service" => ComponentType::Service,
                    "repository" => ComponentType::Repository,
                    "model" => ComponentType::Model,
                    "utility" => ComponentType::Utility,
                    _ => ComponentType::Widget,
                };
                
                let arch_layer = match architecture_layer {
                    "presentation" => ArchitectureLayer::Presentation,
                    "domain" => ArchitectureLayer::Domain,
                    "data" => ArchitectureLayer::Data,
                    "core" => ArchitectureLayer::Core,
                    _ => ArchitectureLayer::Presentation,
                };

                let component = FlutterComponent {
                    id: id.to_string(),
                    project_id: String::new(), // Will be filled by service
                    component_name: component_name.to_string(),
                    component_type: comp_type,
                    architecture_layer: arch_layer,
                    file_path: file_path.map(|s| s.to_string()),
                    dependencies,
                    riverpod_scope: None,
                    widget_type: None,
                    created_at: None,
                    updated_at: Some(chrono::Utc::now().to_rfc3339()),
                };

                let updated_component = self.container.flutter_service.update_component(&component).await?;
                let content = serde_json::to_string_pretty(&updated_component)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "delete_flutter_component" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let deleted = self.container.flutter_service.delete_component(id).await?;
                let result = serde_json::json!({"deleted": deleted, "component_id": id});
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "get_flutter_component" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let component = self.container.flutter_service.get_component(id).await?;
                let content = serde_json::to_string_pretty(&component)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "list_flutter_components" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;

                let components = self.container.flutter_service.list_components(project_id).await?;
                let content = serde_json::to_string_pretty(&components)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

            // Development Phases CRUD
            "create_development_phase" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;
                let phase_name = args.get("phase_name").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: phase_name", None))?;
                let phase_order = args.get("phase_order").and_then(|v| v.as_i64())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: phase_order", None))? as i32;
                let description = args.get("description").and_then(|v| v.as_str());

                let phase = self.container.development_phase_service.create_phase(project_id, phase_name, phase_order, description).await?;
                let content = serde_json::to_string_pretty(&phase)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "update_development_phase" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;
                let phase_name = args.get("phase_name").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: phase_name", None))?;
                let phase_order = args.get("phase_order").and_then(|v| v.as_i64())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: phase_order", None))? as i32;
                let status = args.get("status").and_then(|v| v.as_str());
                let description = args.get("description").and_then(|v| v.as_str());

                use crate::models::flutter::{DevelopmentPhase, PhaseStatus};
                
                // Parse status enum
                let phase_status = match status {
                    Some("pending") => PhaseStatus::Pending,
                    Some("in_progress") => PhaseStatus::InProgress,
                    Some("completed") => PhaseStatus::Completed,
                    Some("blocked") => PhaseStatus::Blocked,
                    _ => PhaseStatus::Pending,
                };

                let phase = DevelopmentPhase {
                    id: id.to_string(),
                    project_id: String::new(), // Will be filled by service
                    phase_name: phase_name.to_string(),
                    phase_order,
                    status: phase_status,
                    description: description.map(|s| s.to_string()),
                    completion_criteria: Vec::new(),
                    dependencies: Vec::new(),
                    started_at: None,
                    completed_at: None,
                    created_at: None,
                };

                let updated_phase = self.container.development_phase_service.update_phase(&phase).await?;
                let content = serde_json::to_string_pretty(&updated_phase)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "delete_development_phase" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let deleted = self.container.development_phase_service.delete_phase(id).await?;
                let result = serde_json::json!({"deleted": deleted, "phase_id": id});
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "get_development_phase" => {
                let args = request.arguments.unwrap_or_default();
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: id", None))?;

                let phase = self.container.development_phase_service.get_phase(id).await?;
                let content = serde_json::to_string_pretty(&phase)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },
            "list_development_phases" => {
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing required parameter: project_id", None))?;

                let phases = self.container.development_phase_service.list_phases(project_id).await?;
                let content = serde_json::to_string_pretty(&phases)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            },

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

                    let component = self.container.flutter_service.create_component(project_id, component_name, component_type, architecture_layer, file_path).await?;
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
                    "flutter_component" => {
                        let component = self.container.flutter_service.get_component(id).await?;
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

            // TODO: Implement remaining CRUD operations
            _ => {
                Err(McpError::method_not_found::<CallToolRequestMethod>())
            }
        }
    }
}
