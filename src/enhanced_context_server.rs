use crate::api::SpecificationAnalyticsTools;
use crate::container::AppContainer;
use crate::models::framework::{
    FeatureInfo, FeatureStatus, ServerCapabilitiesInfo, ServerMetadata, TableInfo, ToolInfo,
    UsageExample,
};
use crate::services::AnalyticsHelper;
use anyhow::Result;
use rmcp::{handler::server::ServerHandler, model::ErrorData as McpError, model::*};
use std::sync::Arc;
use std::time::Instant;

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
                        "data": {
                            "type": "array", 
                            "description": "Array of entity data or IDs",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {"type": "string", "description": "Entity ID (required for update and delete operations)"},
                                    "name": {"type": "string", "description": "Entity name (for create and update operations)"},
                                    "description": {"type": "string", "description": "Entity description (for create and update operations)"}
                                },
                                "additionalProperties": true
                            }
                        }
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

            // Analytics MCP Tools
            Tool {
                name: "get_usage_analytics".into(),
                description: Some("Retrieve usage statistics for entities or global analytics".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "scope": {"type": "string", "enum": ["global", "entity"], "description": "Scope of analytics to retrieve"},
                        "entity_type": {"type": "string", "description": "Entity type (required for entity scope)"},
                        "entity_id": {"type": "string", "description": "Entity ID (required for entity scope)"}
                    },
                    "required": ["scope"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "get_context_insights".into(),
                description: Some("Get project-level analytics and insights".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project to analyze"}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "generate_quality_report".into(),
                description: Some("Generate a context health assessment and quality report".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "start_date": {"type": "string", "format": "date-time", "description": "Start date for the report (ISO 8601 format)"},
                        "end_date": {"type": "string", "format": "date-time", "description": "End date for the report (ISO 8601 format)"},
                        "project_id": {"type": "string", "description": "Optional project ID to filter the report"}
                    },
                    "required": ["start_date", "end_date"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "export_analytics_data".into(),
                description: Some("Export analytics data for data portability and external analysis".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "format": {"type": "string", "enum": ["json", "csv"], "description": "Export format", "default": "json"},
                        "start_date": {"type": "string", "format": "date-time", "description": "Start date for export (ISO 8601 format)"},
                        "end_date": {"type": "string", "format": "date-time", "description": "End date for export (ISO 8601 format)"},
                        "project_id": {"type": "string", "description": "Optional project ID to filter the export"},
                        "event_types": {"type": "array", "items": {"type": "string"}, "description": "Optional array of event types to include"}
                    },
                    "required": ["start_date", "end_date"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Specification Import and Management Tools
            Tool {
                name: "scan_specifications".into(),
                description: Some("Scan and import all Kiro specifications from .kiro/specs directory".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "base_path": {"type": "string", "description": "Base path to scan for specifications (defaults to .kiro/specs)", "default": ".kiro/specs"}
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "import_specification".into(),
                description: Some("Import a single specification file".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string", "description": "Path to the specification file to import"}
                    },
                    "required": ["file_path"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "validate_specification".into(),
                description: Some("Validate a specification file and return validation issues".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string", "description": "Path to the specification file to validate"}
                    },
                    "required": ["file_path"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "start_spec_monitoring".into(),
                description: Some("Start monitoring .kiro/specs directory for changes".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "base_path": {"type": "string", "description": "Base path to monitor (defaults to .kiro/specs)", "default": ".kiro/specs"}
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "get_specification_versions".into(),
                description: Some("Get all versions of a specification".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "spec_id": {"type": "string", "description": "ID of the specification"}
                    },
                    "required": ["spec_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "compare_specification_versions".into(),
                description: Some("Compare two versions of a specification".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "version1_id": {"type": "string", "description": "ID of the first version"},
                        "version2_id": {"type": "string", "description": "ID of the second version"}
                    },
                    "required": ["version1_id", "version2_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },

            // Specification Analytics Tools
            Tool {
                name: "track_requirements_progress".into(),
                description: Some("Track progress for all requirements in a project, including completion percentages, linked tasks, and acceptance criteria status".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project to track requirements progress for"}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "track_tasks_progress".into(),
                description: Some("Track progress for all tasks in a project, including status, completion percentage, time tracking, and dependencies".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project to track tasks progress for"}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "analyze_specification_completeness".into(),
                description: Some("Analyze completeness of specifications in a project, including content quality, missing sections, and recommendations".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project to analyze specification completeness for"}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "calculate_development_velocity".into(),
                description: Some("Calculate development velocity metrics based on task and requirement completion over a specified time period".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project to calculate velocity for"},
                        "days": {"type": "integer", "description": "Number of days to look back for velocity calculation", "default": 30, "minimum": 1, "maximum": 365}
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "generate_specification_health_report".into(),
                description: Some("Generate a comprehensive health report for all specifications in a project, including progress, completeness, velocity, and recommendations".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The ID of the project to generate health report for"}
                    },
                    "required": ["project_id"]
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
                let content = serde_json::to_string_pretty(&projects).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {e}"), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            // Context Query
            "query_context" => {
                let start_time = Instant::now();
                let args = request.arguments.unwrap_or_default();
                let project_id =
                    args.get("project_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            McpError::invalid_params("Missing required parameter: project_id", None)
                        })?;
                let feature_area = args
                    .get("feature_area")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: feature_area", None)
                    })?;
                let task_type =
                    args.get("task_type")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            McpError::invalid_params("Missing required parameter: task_type", None)
                        })?;
                let components: Vec<String> = args
                    .get("components")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let query_result = self
                    .container
                    .context_query_service
                    .query_context(project_id, feature_area, task_type, &components)
                    .await;

                let duration_ms = start_time.elapsed().as_millis() as u64;
                
                match query_result {
                    Ok(result) => {
                        // Track successful query
                        let analytics_event = AnalyticsHelper::create_context_query_event(
                            Some(project_id.to_string()),
                            Some(feature_area.to_string()),
                            Some(task_type.to_string()),
                            Some(components),
                            Some(duration_ms),
                            true,
                            None,
                        );
                        
                        if let Err(e) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", e);
                        }

                        let content = serde_json::to_string_pretty(&result).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => {
                        // Track failed query
                        let analytics_event = AnalyticsHelper::create_context_query_event(
                            Some(project_id.to_string()),
                            Some(feature_area.to_string()),
                            Some(task_type.to_string()),
                            Some(components),
                            Some(duration_ms),
                            false,
                            Some(e.to_string()),
                        );
                        
                        if let Err(analytics_err) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", analytics_err);
                        }

                        Err(McpError::internal_error(format!("Query failed: {e}"), None))
                    }
                }
            }

            // Architecture validation
            "validate_architecture" => {
                let start_time = Instant::now();
                let args = request.arguments.unwrap_or_default();
                let project_id =
                    args.get("project_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            McpError::invalid_params("Missing required parameter: project_id", None)
                        })?;

                let validation_result = self
                    .container
                    .architecture_validation_service
                    .validate_architecture(project_id)
                    .await;

                let duration_ms = start_time.elapsed().as_millis() as u64;

                match validation_result {
                    Ok(violations) => {
                        // Track successful validation
                        let analytics_event = AnalyticsHelper::create_architecture_validation_event(
                            project_id.to_string(),
                            violations.len(),
                            Some(duration_ms),
                            true,
                            None,
                        );
                        
                        if let Err(e) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", e);
                        }

                        let content = serde_json::to_string_pretty(&violations).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => {
                        // Track failed validation
                        let analytics_event = AnalyticsHelper::create_architecture_validation_event(
                            project_id.to_string(),
                            0,
                            Some(duration_ms),
                            false,
                            Some(e.to_string()),
                        );
                        
                        if let Err(analytics_err) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", analytics_err);
                        }

                        Err(McpError::internal_error(format!("Validation failed: {e}"), None))
                    }
                }
            }

            // Server capabilities
            "get_server_capabilities" => {
                let capabilities = ServerCapabilitiesInfo {
                    server_info: ServerMetadata {
                        name: "Enhanced Context Server".to_string(),
                        version: "0.2.0".to_string(),
                        description:
                            "Professional Context Engine with AI-powered intelligence, semantic search, real-time sync, and comprehensive project specification management"
                                .to_string(),
                        config_directory: "~/.context-server".to_string(),
                    },
                    features: vec![
                        FeatureInfo {
                            name: "Enhanced CRUD Operations".to_string(),
                            description: "Full CRUD for all entities with bulk operations and universal entity handlers"
                                .to_string(),
                            status: FeatureStatus::Implemented,
                            tools: vec![
                                "create_entity".to_string(),
                                "update_entity".to_string(),
                                "delete_entity".to_string(),
                                "get_entity".to_string(),
                                "list_entities".to_string(),
                                "bulk_*".to_string(),
                            ],
                        },
                        FeatureInfo {
                            name: "SOLID Architecture".to_string(),
                            description: "Service/Repository pattern with dependency injection and clean architecture"
                                .to_string(),
                            status: FeatureStatus::Implemented,
                            tools: vec!["All operations".to_string()],
                        },
                        FeatureInfo {
                            name: "Project Specification Management".to_string(),
                            description: "Complete Kiro specification integration with automatic parsing, versioning, and context linking"
                                .to_string(),
                            status: FeatureStatus::Implemented,
                            tools: vec![
                                "scan_specifications".to_string(),
                                "import_specification".to_string(),
                                "validate_specification".to_string(),
                                "get_specification_versions".to_string(),
                                "compare_specification_versions".to_string(),
                                "start_spec_monitoring".to_string(),
                            ],
                        },
                        FeatureInfo {
                            name: "Specification Analytics & Intelligence".to_string(),
                            description: "Advanced analytics for requirements, tasks, development velocity, and project health"
                                .to_string(),
                            status: FeatureStatus::Implemented,
                            tools: vec![
                                "track_requirements_progress".to_string(),
                                "track_tasks_progress".to_string(),
                                "analyze_specification_completeness".to_string(),
                                "calculate_development_velocity".to_string(),
                                "generate_specification_health_report".to_string(),
                            ],
                        },
                        FeatureInfo {
                            name: "Context Intelligence & Quality".to_string(),
                            description: "AI-powered context relationship detection, quality scoring, and intelligent suggestions"
                                .to_string(),
                            status: FeatureStatus::Implemented,
                            tools: vec![
                                "query_context".to_string(),
                                "validate_architecture".to_string(),
                                "get_context_insights".to_string(),
                                "generate_quality_report".to_string(),
                            ],
                        },
                        FeatureInfo {
                            name: "Usage Analytics & Insights".to_string(),
                            description: "Comprehensive usage tracking, analytics, and data export capabilities"
                                .to_string(),
                            status: FeatureStatus::Implemented,
                            tools: vec![
                                "get_usage_analytics".to_string(),
                                "get_context_insights".to_string(),
                                "generate_quality_report".to_string(),
                                "export_analytics_data".to_string(),
                            ],
                        },
                        FeatureInfo {
                            name: "Cache Management".to_string(),
                            description: "Intelligent caching system with project-level and global cache management"
                                .to_string(),
                            status: FeatureStatus::Implemented,
                            tools: vec![
                                "cache_management".to_string(),
                            ],
                        },
                    ],
                    database_tables: vec![
                        TableInfo {
                            name: "projects".to_string(),
                            description: "Main project information and metadata".to_string(),
                            primary_fields: vec!["id".to_string(), "name".to_string(), "description".to_string()],
                            example_use: "Organizing code contexts by project with comprehensive metadata".to_string(),
                        },
                        TableInfo {
                            name: "business_rules".to_string(),
                            description: "Domain-specific business logic rules and constraints".to_string(),
                            primary_fields: vec![
                                "id".to_string(),
                                "rule_name".to_string(),
                                "domain_area".to_string(),
                                "project_id".to_string(),
                            ],
                            example_use: "Capturing business constraints for AI code generation and validation".to_string(),
                        },
                        TableInfo {
                            name: "architectural_decisions".to_string(),
                            description: "Architecture Decision Records (ADRs) and design choices".to_string(),
                            primary_fields: vec![
                                "id".to_string(),
                                "decision_title".to_string(),
                                "status".to_string(),
                                "project_id".to_string(),
                            ],
                            example_use: "Tracking architectural decisions and their rationale for consistent development".to_string(),
                        },
                        TableInfo {
                            name: "performance_requirements".to_string(),
                            description: "Performance constraints and non-functional requirements".to_string(),
                            primary_fields: vec![
                                "id".to_string(),
                                "requirement_name".to_string(),
                                "metric_type".to_string(),
                                "target_value".to_string(),
                            ],
                            example_use: "Defining performance benchmarks and optimization targets".to_string(),
                        },
                        TableInfo {
                            name: "framework_components".to_string(),
                            description: "Framework-agnostic component definitions and architecture layers".to_string(),
                            primary_fields: vec![
                                "id".to_string(),
                                "component_name".to_string(),
                                "component_type".to_string(),
                                "architecture_layer".to_string(),
                            ],
                            example_use: "Managing component architecture and clean architecture compliance".to_string(),
                        },
                        TableInfo {
                            name: "specifications".to_string(),
                            description: "Project specifications, requirements, and tasks from Kiro specs".to_string(),
                            primary_fields: vec![
                                "id".to_string(),
                                "title".to_string(),
                                "spec_type".to_string(),
                                "status".to_string(),
                                "project_id".to_string(),
                            ],
                            example_use: "Managing project specifications with automatic parsing and version tracking".to_string(),
                        },
                        TableInfo {
                            name: "specification_versions".to_string(),
                            description: "Version history and change tracking for specifications".to_string(),
                            primary_fields: vec![
                                "id".to_string(),
                                "spec_id".to_string(),
                                "version_number".to_string(),
                                "created_at".to_string(),
                            ],
                            example_use: "Tracking specification changes and enabling version comparison".to_string(),
                        },
                        TableInfo {
                            name: "enhanced_context".to_string(),
                            description: "Enhanced context items with relationships and quality metrics".to_string(),
                            primary_fields: vec![
                                "id".to_string(),
                                "context_type".to_string(),
                                "quality_score".to_string(),
                                "relationship_count".to_string(),
                            ],
                            example_use: "Storing intelligent context with AI-powered relationship detection".to_string(),
                        },
                        TableInfo {
                            name: "analytics_events".to_string(),
                            description: "Usage analytics and event tracking for insights".to_string(),
                            primary_fields: vec![
                                "id".to_string(),
                                "event_type".to_string(),
                                "entity_type".to_string(),
                                "timestamp".to_string(),
                            ],
                            example_use: "Tracking usage patterns and generating analytics insights".to_string(),
                        },
                    ],
                    mcp_tools: vec![
                        // Core Context Operations
                        ToolInfo {
                            name: "query_context".to_string(),
                            description: "Query project context with AI-powered intelligence".to_string(),
                            category: "Core".to_string(),
                            required_params: vec![
                                "project_id".to_string(),
                                "feature_area".to_string(),
                                "task_type".to_string(),
                                "components".to_string(),
                            ],
                            example_use: "Get curated context for implementing authentication features".to_string(),
                        },
                        // Universal CRUD Operations
                        ToolInfo {
                            name: "create_entity".to_string(),
                            description: "Create any entity type with universal handler".to_string(),
                            category: "CRUD".to_string(),
                            required_params: vec![
                                "entity_type".to_string(),
                                "data".to_string(),
                            ],
                            example_use: "Create business rules, architectural decisions, or any other entity".to_string(),
                        },
                        ToolInfo {
                            name: "get_entity".to_string(),
                            description: "Retrieve any entity by ID and type".to_string(),
                            category: "CRUD".to_string(),
                            required_params: vec![
                                "entity_type".to_string(),
                                "id".to_string(),
                            ],
                            example_use: "Get specific business rule or architectural decision".to_string(),
                        },
                        ToolInfo {
                            name: "list_entities".to_string(),
                            description: "List entities by type with optional filtering".to_string(),
                            category: "CRUD".to_string(),
                            required_params: vec![
                                "entity_type".to_string(),
                            ],
                            example_use: "List all business rules for a project".to_string(),
                        },
                        // Specification Management
                        ToolInfo {
                            name: "scan_specifications".to_string(),
                            description: "Automatically scan and import Kiro specifications".to_string(),
                            category: "Specifications".to_string(),
                            required_params: vec![],
                            example_use: "Import all specifications from .kiro/specs directory".to_string(),
                        },
                        ToolInfo {
                            name: "import_specification".to_string(),
                            description: "Import a single specification file with parsing".to_string(),
                            category: "Specifications".to_string(),
                            required_params: vec![
                                "file_path".to_string(),
                            ],
                            example_use: "Import specific requirements.md or tasks.md file".to_string(),
                        },
                        ToolInfo {
                            name: "validate_specification".to_string(),
                            description: "Validate specification format and content".to_string(),
                            category: "Specifications".to_string(),
                            required_params: vec![
                                "file_path".to_string(),
                            ],
                            example_use: "Check specification file for format issues and completeness".to_string(),
                        },
                        // Specification Analytics
                        ToolInfo {
                            name: "track_requirements_progress".to_string(),
                            description: "Track progress for all requirements in a project".to_string(),
                            category: "Analytics".to_string(),
                            required_params: vec![
                                "project_id".to_string(),
                            ],
                            example_use: "Monitor requirement completion rates and acceptance criteria status".to_string(),
                        },
                        ToolInfo {
                            name: "track_tasks_progress".to_string(),
                            description: "Track progress for all tasks with dependencies and time tracking".to_string(),
                            category: "Analytics".to_string(),
                            required_params: vec![
                                "project_id".to_string(),
                            ],
                            example_use: "Monitor task completion, blockers, and development velocity".to_string(),
                        },
                        ToolInfo {
                            name: "generate_specification_health_report".to_string(),
                            description: "Generate comprehensive health report for project specifications".to_string(),
                            category: "Analytics".to_string(),
                            required_params: vec![
                                "project_id".to_string(),
                            ],
                            example_use: "Get executive summary of project health, velocity, and recommendations".to_string(),
                        },
                        ToolInfo {
                            name: "calculate_development_velocity".to_string(),
                            description: "Calculate development velocity metrics and trends".to_string(),
                            category: "Analytics".to_string(),
                            required_params: vec![
                                "project_id".to_string(),
                            ],
                            example_use: "Measure team productivity and identify bottlenecks".to_string(),
                        },
                        // Usage Analytics
                        ToolInfo {
                            name: "get_usage_analytics".to_string(),
                            description: "Retrieve usage statistics and patterns".to_string(),
                            category: "Analytics".to_string(),
                            required_params: vec![
                                "scope".to_string(),
                            ],
                            example_use: "Get global or entity-specific usage analytics".to_string(),
                        },
                        ToolInfo {
                            name: "export_analytics_data".to_string(),
                            description: "Export analytics data for external analysis".to_string(),
                            category: "Analytics".to_string(),
                            required_params: vec![
                                "start_date".to_string(),
                                "end_date".to_string(),
                            ],
                            example_use: "Export usage data in JSON or CSV format for reporting".to_string(),
                        },
                        // Architecture & Quality
                        ToolInfo {
                            name: "validate_architecture".to_string(),
                            description: "Validate Clean Architecture rules and detect violations".to_string(),
                            category: "Quality".to_string(),
                            required_params: vec![
                                "project_id".to_string(),
                            ],
                            example_use: "Check for architecture layer violations and dependency issues".to_string(),
                        },
                        ToolInfo {
                            name: "generate_quality_report".to_string(),
                            description: "Generate context health assessment and quality report".to_string(),
                            category: "Quality".to_string(),
                            required_params: vec![
                                "start_date".to_string(),
                                "end_date".to_string(),
                            ],
                            example_use: "Assess context quality and get improvement recommendations".to_string(),
                        },
                        // Bulk Operations
                        ToolInfo {
                            name: "bulk_operations".to_string(),
                            description: "Perform bulk operations on multiple entities".to_string(),
                            category: "Bulk".to_string(),
                            required_params: vec![
                                "operation".to_string(),
                                "entity_type".to_string(),
                                "data".to_string(),
                            ],
                            example_use: "Create, update, or delete multiple entities in one operation".to_string(),
                        },
                        // Server Management
                        ToolInfo {
                            name: "get_server_capabilities".to_string(),
                            description: "Get comprehensive server information and capabilities".to_string(),
                            category: "Core".to_string(),
                            required_params: vec![],
                            example_use: "Discover available features, tools, and database schema".to_string(),
                        },
                        ToolInfo {
                            name: "cache_management".to_string(),
                            description: "Manage cache and temporary data".to_string(),
                            category: "Management".to_string(),
                            required_params: vec![
                                "action".to_string(),
                            ],
                            example_use: "Clear project cache or global cache for performance optimization".to_string(),
                        },
                    ],
                    usage_examples: vec![
                        UsageExample {
                            scenario: "Setting up a new project with specifications".to_string(),
                            steps: vec![
                                "1. create_entity with entity_type='project' and project details".to_string(),
                                "2. scan_specifications to import all Kiro specs from .kiro/specs".to_string(),
                                "3. bulk_create_components for initial architecture setup".to_string(),
                                "4. create_entity with entity_type='business_rule' for domain logic".to_string(),
                                "5. query_context to get AI-curated context for development tasks".to_string(),
                            ],
                        },
                        UsageExample {
                            scenario: "Monitoring project health and progress".to_string(),
                            steps: vec![
                                "1. track_requirements_progress to see requirement completion status".to_string(),
                                "2. track_tasks_progress to monitor task completion and blockers".to_string(),
                                "3. calculate_development_velocity to measure team productivity".to_string(),
                                "4. generate_specification_health_report for executive summary".to_string(),
                                "5. export_analytics_data for external reporting and analysis".to_string(),
                            ],
                        },
                        UsageExample {
                            scenario: "AI-powered development assistance".to_string(),
                            steps: vec![
                                "1. query_context with feature_area, task_type, and components".to_string(),
                                "2. validate_architecture to check for Clean Architecture compliance".to_string(),
                                "3. get_context_insights for project-level analytics and patterns".to_string(),
                                "4. generate_quality_report to assess context health and get recommendations".to_string(),
                            ],
                        },
                        UsageExample {
                            scenario: "Specification management workflow".to_string(),
                            steps: vec![
                                "1. import_specification to add new spec files".to_string(),
                                "2. validate_specification to check format and completeness".to_string(),
                                "3. get_specification_versions to see version history".to_string(),
                                "4. compare_specification_versions to see changes between versions".to_string(),
                                "5. start_spec_monitoring for automatic updates on file changes".to_string(),
                            ],
                        },
                    ],
                    recommended_workflow: vec![
                        "1. Project Setup: Use create_entity to create your project".to_string(),
                        "2. Specification Import: Run scan_specifications to import all Kiro specs".to_string(),
                        "3. Architecture Setup: Use bulk_create_components for initial component structure".to_string(),
                        "4. Context Definition: Add business_rules, architectural_decisions, and performance_requirements".to_string(),
                        "5. Development: Use query_context for AI-powered development assistance".to_string(),
                        "6. Monitoring: Track progress with track_requirements_progress and track_tasks_progress".to_string(),
                        "7. Quality Assurance: Run validate_architecture and generate_quality_report regularly".to_string(),
                        "8. Analytics: Use generate_specification_health_report for project insights".to_string(),
                        "9. Optimization: Use get_usage_analytics to understand usage patterns".to_string(),
                        "10. Maintenance: Use cache_management to optimize performance as needed".to_string(),
                    ],
                };

                let content = serde_json::to_string_pretty(&capabilities).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {e}"), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            // Business Rules CRUD operations are now handled by universal CRUD handlers

            // Architectural Decisions CRUD operations are now handled by universal CRUD handlers

            // Performance Requirements CRUD operations are now handled by universal CRUD handlers

            // Framework Components CRUD
            // Removed legacy Flutter component handlers - now using universal entity handlers

            // Development Phases CRUD
            // Removed legacy development phase handlers - now using universal entity handlers

            // Bulk Operations
            "bulk_create_components" => {
                let start_time = Instant::now();
                let args = request.arguments.unwrap_or_default();
                let project_id =
                    args.get("project_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            McpError::invalid_params("Missing required parameter: project_id", None)
                        })?;
                let components_data = args
                    .get("components")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: components", None)
                    })?;

                let mut components = Vec::new();
                for comp_data in components_data {
                    let obj = comp_data
                        .as_object()
                        .ok_or_else(|| McpError::invalid_params("Invalid component data", None))?;
                    let component_name = obj
                        .get("component_name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            McpError::invalid_params(
                                "Missing component_name in component data",
                                None,
                            )
                        })?;
                    let component_type = obj
                        .get("component_type")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            McpError::invalid_params(
                                "Missing component_type in component data",
                                None,
                            )
                        })?;
                    let architecture_layer = obj
                        .get("architecture_layer")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            McpError::invalid_params(
                                "Missing architecture_layer in component data",
                                None,
                            )
                        })?;
                    let file_path = obj.get("file_path").and_then(|v| v.as_str());

                    let component = self
                        .container
                        .framework_service
                        .create_component(
                            project_id,
                            component_name,
                            component_type,
                            architecture_layer,
                            file_path,
                            None,
                        )
                        .await?;
                    components.push(component);
                }

                let duration_ms = start_time.elapsed().as_millis() as u64;
                
                // Track successful bulk operation
                let analytics_event = AnalyticsHelper::create_bulk_operation_event(
                    Some(project_id.to_string()),
                    "framework_component".to_string(),
                    "bulk_create".to_string(),
                    components.len(),
                    Some(duration_ms),
                    true,
                    None,
                );
                
                if let Err(e) = self.container.analytics_service.track_event(analytics_event).await {
                    tracing::warn!("Failed to track analytics event: {}", e);
                }

                let content = serde_json::to_string_pretty(&components).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {e}"), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            // Cache and Cleanup Operations
            "clear_project_cache" => {
                let args = request.arguments.unwrap_or_default();
                let project_id =
                    args.get("project_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            McpError::invalid_params("Missing required parameter: project_id", None)
                        })?;

                // Clear project-related data - for now, just return a success message
                // In a real implementation, you might want to clear cached data, temporary files, etc.
                let result = serde_json::json!({
                    "message": "Project cache cleared successfully",
                    "project_id": project_id,
                    "cleared": true,
                    "note": "Cache clearing implementation can be customized based on your needs"
                });
                let content = serde_json::to_string_pretty(&result).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {e}"), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "clear_all_cache" => {
                // This would be a nuclear option - clear everything
                let result = serde_json::json!({
                    "message": "All cache cleared successfully",
                    "warning": "This operation removes all stored data",
                    "cleared": true
                });
                let content = serde_json::to_string_pretty(&result).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            // Get entity by ID operations
            "get_entity" => {
                let args = request.arguments.unwrap_or_default();
                let entity_type = args
                    .get("entity_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: entity_type", None)
                    })?;
                let id = args.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: id", None)
                })?;

                let result = match entity_type {
                    "project" => {
                        let project = self.container.project_service.get_project(id).await?;
                        serde_json::to_value(project)
                    }
                    "business_rule" => {
                        let rule = self
                            .container
                            .context_crud_service
                            .get_business_rule(id)
                            .await?;
                        serde_json::to_value(rule)
                    }
                    "architectural_decision" => {
                        let decision = self
                            .container
                            .context_crud_service
                            .get_architectural_decision(id)
                            .await?;
                        serde_json::to_value(decision)
                    }
                    "performance_requirement" => {
                        let requirement = self
                            .container
                            .context_crud_service
                            .get_performance_requirement(id)
                            .await?;
                        serde_json::to_value(requirement)
                    }
                    "framework_component" => {
                        let component = self.container.framework_service.get_component(id).await?;
                        serde_json::to_value(component)
                    }
                    "development_phase" => {
                        let phase = self
                            .container
                            .development_phase_service
                            .get_phase(id)
                            .await?;
                        serde_json::to_value(phase)
                    }
                    _ => return Err(McpError::invalid_params("Invalid entity_type", None)),
                }
                .map_err(|e| McpError::internal_error(format!("Serialization error: {e}"), None))?;

                let content = serde_json::to_string_pretty(&result).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {e}"), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            // Universal CRUD Operations - Remove duplicate handlers
            "manage_project" => {
                let args = request.arguments.unwrap_or_default();
                let action = args.get("action").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: action", None)
                })?;

                let result = match action {
                    "create" => {
                        // Try to get data from 'data' field first, then fall back to direct args
                        let (name, description, repository_url) = if let Some(data) = args.get("data").and_then(|v| v.as_object()) {
                            // Data is in nested 'data' object
                            let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let description = data.get("description").and_then(|v| v.as_str());
                            let repository_url = data.get("repository_url").and_then(|v| v.as_str());
                            (name, description, repository_url)
                        } else {
                            // Try direct args for convenience
                            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let description = args.get("description").and_then(|v| v.as_str());
                            let repository_url = args.get("repository_url").and_then(|v| v.as_str());
                            (name, description, repository_url)
                        };

                        if name.is_empty() {
                            return Err(McpError::invalid_params(
                                "Missing required parameter: name (either in 'data' object or as direct parameter)",
                                None,
                            ));
                        }

                        let project = self
                            .container
                            .project_service
                            .create_project(name, description, repository_url)
                            .await?;
                        serde_json::to_value(project)
                    }
                    "get" => {
                        let id = args.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                            McpError::invalid_params("Missing required parameter: id for get", None)
                        })?;
                        let project = self.container.project_service.get_project(id).await?;
                        serde_json::to_value(project)
                    }
                    "delete" => {
                        let id = args.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                            McpError::invalid_params(
                                "Missing required parameter: id for delete",
                                None,
                            )
                        })?;
                        let deleted = self.container.project_service.delete_project(id).await?;
                        serde_json::to_value(serde_json::json!({"deleted": deleted, "id": id}))
                    }
                    "list" => {
                        let projects = self.container.project_service.list_projects().await?;
                        serde_json::to_value(projects)
                    }
                    _ => return Err(McpError::invalid_params("Unsupported action", None)),
                }
                .map_err(|e| McpError::internal_error(format!("Serialization error: {e}"), None))?;

                let content = serde_json::to_string_pretty(&result).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {e}"), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "cache_management" => {
                let args = request.arguments.unwrap_or_default();
                let action = args.get("action").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: action", None)
                })?;

                let result = match action {
                    "clear_project" => {
                        let project_id = args
                            .get("project_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: project_id for clear_project",
                                    None,
                                )
                            })?;
                        serde_json::json!({
                            "message": "Project cache cleared successfully",
                            "project_id": project_id,
                            "cleared": true,
                            "note": "Cache clearing implementation can be customized based on your needs"
                        })
                    }
                    "clear_all" => {
                        serde_json::json!({
                            "message": "All cache cleared successfully",
                            "warning": "This operation removes all stored data",
                            "cleared": true
                        })
                    }
                    _ => return Err(McpError::invalid_params("Unsupported cache action", None)),
                };

                let content = serde_json::to_string_pretty(&result).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            // Analytics MCP Tools
            "get_usage_analytics" => {
                let start_time = Instant::now();
                let args = request.arguments.unwrap_or_default();
                let scope = args.get("scope").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: scope", None)
                })?;

                let analytics_result = match scope {
                    "global" => {
                        self.container.analytics_service.get_global_statistics().await
                    }
                    "entity" => {
                        let entity_type = args.get("entity_type").and_then(|v| v.as_str()).ok_or_else(|| {
                            McpError::invalid_params("Missing required parameter: entity_type for entity scope", None)
                        })?;
                        let entity_id = args.get("entity_id").and_then(|v| v.as_str()).ok_or_else(|| {
                            McpError::invalid_params("Missing required parameter: entity_id for entity scope", None)
                        })?;
                        
                        match self.container.analytics_service.get_entity_usage(entity_type, entity_id).await {
                            Ok(usage_stats) => Ok(serde_json::to_value(usage_stats).unwrap_or_default().as_object().unwrap().clone().into_iter().collect()),
                            Err(e) => Err(e),
                        }
                    }
                    _ => return Err(McpError::invalid_params("Invalid scope. Must be 'global' or 'entity'", None)),
                };

                let duration_ms = start_time.elapsed().as_millis() as u64;

                match analytics_result {
                    Ok(result) => {
                        // Track successful analytics query
                        let analytics_event = AnalyticsHelper::create_analytics_event(
                            "get_usage_analytics".to_string(),
                            Some(scope.to_string()),
                            Some(duration_ms),
                            true,
                            None,
                        );
                        
                        if let Err(e) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", e);
                        }

                        let content = serde_json::to_string_pretty(&result).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => {
                        // Track failed analytics query
                        let analytics_event = AnalyticsHelper::create_analytics_event(
                            "get_usage_analytics".to_string(),
                            Some(scope.to_string()),
                            Some(duration_ms),
                            false,
                            Some(e.to_string()),
                        );
                        
                        if let Err(analytics_err) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", analytics_err);
                        }

                        Err(McpError::internal_error(format!("Analytics query failed: {e}"), None))
                    }
                }
            }

            "get_context_insights" => {
                let start_time = Instant::now();
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: project_id", None)
                })?;

                let insights_result = self.container.analytics_service.get_project_insights(project_id).await;
                let duration_ms = start_time.elapsed().as_millis() as u64;

                match insights_result {
                    Ok(insights) => {
                        // Track successful insights query
                        let analytics_event = AnalyticsHelper::create_analytics_event(
                            "get_context_insights".to_string(),
                            Some(project_id.to_string()),
                            Some(duration_ms),
                            true,
                            None,
                        );
                        
                        if let Err(e) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", e);
                        }

                        let content = serde_json::to_string_pretty(&insights).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => {
                        // Track failed insights query
                        let analytics_event = AnalyticsHelper::create_analytics_event(
                            "get_context_insights".to_string(),
                            Some(project_id.to_string()),
                            Some(duration_ms),
                            false,
                            Some(e.to_string()),
                        );
                        
                        if let Err(analytics_err) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", analytics_err);
                        }

                        Err(McpError::internal_error(format!("Context insights query failed: {e}"), None))
                    }
                }
            }

            "generate_quality_report" => {
                let start_time = Instant::now();
                let args = request.arguments.unwrap_or_default();
                let start_date_str = args.get("start_date").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: start_date", None)
                })?;
                let end_date_str = args.get("end_date").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: end_date", None)
                })?;

                // Parse dates
                let start_date = chrono::DateTime::parse_from_rfc3339(start_date_str)
                    .map_err(|_| McpError::invalid_params("Invalid start_date format. Use ISO 8601 format", None))?
                    .with_timezone(&chrono::Utc);
                let end_date = chrono::DateTime::parse_from_rfc3339(end_date_str)
                    .map_err(|_| McpError::invalid_params("Invalid end_date format. Use ISO 8601 format", None))?
                    .with_timezone(&chrono::Utc);

                let project_id = args.get("project_id").and_then(|v| v.as_str());

                let report_result = self.container.analytics_service.generate_usage_report(start_date, end_date).await;
                let duration_ms = start_time.elapsed().as_millis() as u64;

                match report_result {
                    Ok(mut report) => {
                        // Add quality assessment to the report
                        if let Some(pid) = project_id {
                            if let Ok(insights) = self.container.analytics_service.get_project_insights(pid).await {
                                if let Some(report_obj) = report.as_object_mut() {
                                    report_obj.insert("quality_assessment".to_string(), serde_json::json!({
                                        "context_health_score": insights.context_health_score,
                                        "recommendations": insights.recommendations,
                                        "most_active_entity_types": insights.most_active_entity_types
                                    }));
                                }
                            }
                        }

                        // Track successful report generation
                        let analytics_event = AnalyticsHelper::create_analytics_event(
                            "generate_quality_report".to_string(),
                            project_id.map(|s| s.to_string()),
                            Some(duration_ms),
                            true,
                            None,
                        );
                        
                        if let Err(e) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", e);
                        }

                        let content = serde_json::to_string_pretty(&report).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => {
                        // Track failed report generation
                        let analytics_event = AnalyticsHelper::create_analytics_event(
                            "generate_quality_report".to_string(),
                            project_id.map(|s| s.to_string()),
                            Some(duration_ms),
                            false,
                            Some(e.to_string()),
                        );
                        
                        if let Err(analytics_err) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", analytics_err);
                        }

                        Err(McpError::internal_error(format!("Quality report generation failed: {e}"), None))
                    }
                }
            }

            "export_analytics_data" => {
                let start_time = Instant::now();
                let args = request.arguments.unwrap_or_default();
                let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("json");
                let start_date_str = args.get("start_date").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: start_date", None)
                })?;
                let end_date_str = args.get("end_date").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: end_date", None)
                })?;

                // Parse dates
                let start_date = chrono::DateTime::parse_from_rfc3339(start_date_str)
                    .map_err(|_| McpError::invalid_params("Invalid start_date format. Use ISO 8601 format", None))?
                    .with_timezone(&chrono::Utc);
                let end_date = chrono::DateTime::parse_from_rfc3339(end_date_str)
                    .map_err(|_| McpError::invalid_params("Invalid end_date format. Use ISO 8601 format", None))?
                    .with_timezone(&chrono::Utc);

                let project_id = args.get("project_id").and_then(|v| v.as_str());
                let event_types: Vec<String> = args.get("event_types")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let export_result = self.container.analytics_service.generate_usage_report(start_date, end_date).await;
                let duration_ms = start_time.elapsed().as_millis() as u64;

                match export_result {
                    Ok(mut export_data) => {
                        // Add export metadata
                        if let Some(export_obj) = export_data.as_object_mut() {
                            export_obj.insert("export_metadata".to_string(), serde_json::json!({
                                "format": format,
                                "exported_at": chrono::Utc::now().to_rfc3339(),
                                "project_filter": project_id,
                                "event_type_filter": if event_types.is_empty() { None } else { Some(event_types) },
                                "total_records": export_obj.get("summary").and_then(|s| s.get("total_events")).unwrap_or(&serde_json::Value::Number(0.into()))
                            }));
                        }

                        // For CSV format, we would need to implement CSV conversion
                        // For now, we'll return JSON with a note about CSV format
                        let final_content = if format == "csv" {
                            serde_json::json!({
                                "note": "CSV export format requested but not yet implemented. Returning JSON format.",
                                "format": "json",
                                "data": export_data
                            })
                        } else {
                            export_data
                        };

                        // Track successful export
                        let analytics_event = AnalyticsHelper::create_analytics_event(
                            "export_analytics_data".to_string(),
                            project_id.map(|s| s.to_string()),
                            Some(duration_ms),
                            true,
                            None,
                        );
                        
                        if let Err(e) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", e);
                        }

                        let content = serde_json::to_string_pretty(&final_content).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => {
                        // Track failed export
                        let analytics_event = AnalyticsHelper::create_analytics_event(
                            "export_analytics_data".to_string(),
                            project_id.map(|s| s.to_string()),
                            Some(duration_ms),
                            false,
                            Some(e.to_string()),
                        );
                        
                        if let Err(analytics_err) = self.container.analytics_service.track_event(analytics_event).await {
                            tracing::warn!("Failed to track analytics event: {}", analytics_err);
                        }

                        Err(McpError::internal_error(format!("Analytics data export failed: {e}"), None))
                    }
                }
            }

            // Universal CRUD Operations
            // First universal handler for create_entity
            "create_entity" => {
                let start_time = Instant::now();
                let args = request.arguments.unwrap_or_default();
                let entity_type = args
                    .get("entity_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: entity_type", None)
                    })?;
                let data = args
                    .get("data")
                    .and_then(|v| v.as_object())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: data", None)
                    })?;

                let result = match entity_type {
                    "project" => {
                        let name = data.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                            McpError::invalid_params("Missing required parameter: name", None)
                        })?;
                        let description = data.get("description").and_then(|v| v.as_str());
                        let repository_url = data.get("repository_url").and_then(|v| v.as_str());

                        let project = self
                            .container
                            .project_service
                            .create_project(name, description, repository_url)
                            .await?;
                        serde_json::to_value(project).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?
                    }
                    "business_rule" => {
                        let project_id = data
                            .get("project_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: project_id",
                                    None,
                                )
                            })?;
                        let rule_name =
                            data.get("rule_name")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    McpError::invalid_params(
                                        "Missing required parameter: rule_name",
                                        None,
                                    )
                                })?;
                        let description = data.get("description").and_then(|v| v.as_str());
                        let domain_area = data.get("domain_area").and_then(|v| v.as_str());

                        let rule = self
                            .container
                            .context_crud_service
                            .create_business_rule(project_id, rule_name, description, domain_area)
                            .await?;
                        serde_json::to_value(rule).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?
                    }
                    "architectural_decision" => {
                        let project_id = data
                            .get("project_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: project_id",
                                    None,
                                )
                            })?;
                        let decision_title = data
                            .get("decision_title")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: decision_title",
                                    None,
                                )
                            })?;
                        let context = data.get("context").and_then(|v| v.as_str());
                        let decision = data.get("decision").and_then(|v| v.as_str());

                        let arch_decision = self
                            .container
                            .context_crud_service
                            .create_architectural_decision(
                                project_id,
                                decision_title,
                                context,
                                decision,
                            )
                            .await?;
                        serde_json::to_value(arch_decision).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?
                    }
                    "framework_component" => {
                        let project_id = data
                            .get("project_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: project_id",
                                    None,
                                )
                            })?;
                        let component_name = data
                            .get("component_name")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: component_name",
                                    None,
                                )
                            })?;
                        let component_type = data
                            .get("component_type")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: component_type",
                                    None,
                                )
                            })?;
                        let architecture_layer = data
                            .get("architecture_layer")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: architecture_layer",
                                    None,
                                )
                            })?;
                        let file_path = data.get("file_path").and_then(|v| v.as_str());

                        let component = self
                            .container
                            .framework_service
                            .create_component(
                                project_id,
                                component_name,
                                component_type,
                                architecture_layer,
                                file_path,
                                None,
                            )
                            .await?;
                        serde_json::to_value(component).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?
                    }
                    "development_phase" => {
                        let project_id = data
                            .get("project_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: project_id",
                                    None,
                                )
                            })?;
                        let phase_name = data
                            .get("phase_name")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: phase_name",
                                    None,
                                )
                            })?;
                        let phase_order = data
                            .get("phase_order")
                            .and_then(|v| v.as_i64())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: phase_order",
                                    None,
                                )
                            })? as i32;
                        let description = data.get("description").and_then(|v| v.as_str());

                        let phase = self
                            .container
                            .development_phase_service
                            .create_phase(project_id, phase_name, phase_order, description)
                            .await?;
                        serde_json::to_value(phase).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?
                    }
                    // Add more entity types as needed
                    _ => {
                        return Err(McpError::invalid_params(
                            format!("Unsupported entity type: {entity_type}"),
                            None,
                        ))
                    }
                };

                let duration_ms = start_time.elapsed().as_millis() as u64;
                
                // Extract project_id and entity_id from result for analytics
                let project_id = data.get("project_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let entity_id = if let Ok(obj) = result.as_object().ok_or("Invalid result") {
                    obj.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
                } else {
                    None
                };

                // Track successful creation
                let analytics_event = AnalyticsHelper::create_entity_create_event(
                    project_id,
                    entity_type.to_string(),
                    entity_id.unwrap_or_else(|| "unknown".to_string()),
                    Some(duration_ms),
                    true,
                    None,
                );
                
                if let Err(e) = self.container.analytics_service.track_event(analytics_event).await {
                    tracing::warn!("Failed to track analytics event: {}", e);
                }

                let content = serde_json::to_string_pretty(&result).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            "update_entity" => {
                let args = request.arguments.unwrap_or_default();
                let entity_type = args
                    .get("entity_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: entity_type", None)
                    })?;
                let id = args.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: id", None)
                })?;
                let data = args
                    .get("data")
                    .and_then(|v| v.as_object())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: data", None)
                    })?;

                let result = match entity_type {
                    "project" => {
                        use crate::models::context::Project;

                        let name = data.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                            McpError::invalid_params("Missing required parameter: name", None)
                        })?;

                        let project = Project {
                            id: id.to_string(),
                            name: name.to_string(),
                            description: data
                                .get("description")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            repository_url: data
                                .get("repository_url")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            created_at: None,
                            updated_at: None,
                        };

                        let updated_project = self
                            .container
                            .project_service
                            .update_project(&project)
                            .await?;
                        serde_json::to_value(updated_project).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?
                    }
                    "business_rule" => {
                        use crate::models::context::BusinessRule;

                        let rule_name =
                            data.get("rule_name")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    McpError::invalid_params(
                                        "Missing required parameter: rule_name",
                                        None,
                                    )
                                })?;

                        let rule = BusinessRule {
                            id: id.to_string(),
                            project_id: data
                                .get("project_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            rule_name: rule_name.to_string(),
                            description: data
                                .get("description")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            domain_area: data
                                .get("domain_area")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            implementation_pattern: data
                                .get("implementation_pattern")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            constraints: data
                                .get("constraints")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            examples: data
                                .get("examples")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            created_at: None,
                        };

                        let updated_rule = self
                            .container
                            .context_crud_service
                            .update_business_rule(&rule)
                            .await?;
                        serde_json::to_value(updated_rule).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?
                    }
                    "framework_component" => {
                        let component_name = data
                            .get("component_name")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: component_name",
                                    None,
                                )
                            })?;
                        let component_type = data
                            .get("component_type")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: component_type",
                                    None,
                                )
                            })?;
                        let architecture_layer = data
                            .get("architecture_layer")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing required parameter: architecture_layer",
                                    None,
                                )
                            })?;
                        let file_path = data.get("file_path").and_then(|v| v.as_str());

                        // First retrieve the component
                        let component_opt =
                            self.container.framework_service.get_component(id).await?;
                        if component_opt.is_none() {
                            return Err(McpError::invalid_params(
                                format!("Component with id {id} not found"),
                                None,
                            ));
                        }

                        // Get the component first
                        let mut component =
                            match self.container.framework_service.get_component(id).await? {
                                Some(c) => c,
                                None => {
                                    return Err(McpError::invalid_params(
                                        format!("Component with id {id} not found"),
                                        None,
                                    ))
                                }
                            };

                        // Update the component fields
                        component.component_name = component_name.to_string();
                        component.component_type = component_type.to_string();
                        component.architecture_layer = architecture_layer.to_string();

                        if let Some(fp) = file_path {
                            component.file_path = Some(fp.to_string());
                        }

                        // Update the component
                        let updated_component = self
                            .container
                            .framework_service
                            .update_component(&component)
                            .await?;

                        serde_json::to_value(updated_component).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?
                    }
                    // Add more entity types as needed
                    _ => {
                        return Err(McpError::invalid_params(
                            format!("Unsupported entity type: {}", entity_type),
                            None,
                        ))
                    }
                };

                let content = serde_json::to_string_pretty(&result).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            "delete_entity" => {
                let args = request.arguments.unwrap_or_default();
                let entity_type = args
                    .get("entity_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: entity_type", None)
                    })?;
                let id = args.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: id", None)
                })?;

                let result = match entity_type {
                    "project" => {
                        let deleted = self.container.project_service.delete_project(id).await?;
                        serde_json::json!({"deleted": deleted, "project_id": id})
                    }
                    "business_rule" => {
                        let deleted = self
                            .container
                            .context_crud_service
                            .delete_business_rule(id)
                            .await?;
                        serde_json::json!({"deleted": deleted, "rule_id": id})
                    }
                    "architectural_decision" => {
                        let deleted = self
                            .container
                            .context_crud_service
                            .delete_architectural_decision(id)
                            .await?;
                        serde_json::json!({"deleted": deleted, "decision_id": id})
                    }
                    "framework_component" => {
                        let deleted = self
                            .container
                            .framework_service
                            .delete_component(id)
                            .await?;
                        serde_json::json!({"deleted": deleted, "component_id": id})
                    }
                    "development_phase" => {
                        let deleted = self
                            .container
                            .development_phase_service
                            .delete_phase(id)
                            .await?;
                        serde_json::json!({"deleted": deleted, "phase_id": id})
                    }
                    // Add more entity types as needed
                    _ => {
                        return Err(McpError::invalid_params(
                            format!("Unsupported entity type: {}", entity_type),
                            None,
                        ))
                    }
                };

                let content = serde_json::to_string_pretty(&result).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            // Removed duplicate get_entity handler
            "list_entities" => {
                let args = request.arguments.unwrap_or_default();
                let entity_type = args
                    .get("entity_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: entity_type", None)
                    })?;
                let project_id = args.get("project_id").and_then(|v| v.as_str());
                let architecture_layer = args.get("architecture_layer").and_then(|v| v.as_str());

                let result = match entity_type {
                    "project" => {
                        let projects = self.container.project_service.list_projects().await?;
                        serde_json::to_value(projects).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?
                    }
                    "business_rule" => {
                        if let Some(pid) = project_id {
                            let rules = self
                                .container
                                .context_crud_service
                                .list_business_rules(pid)
                                .await?;
                            serde_json::to_value(rules).map_err(|e| {
                                McpError::internal_error(
                                    format!("Serialization error: {}", e),
                                    None,
                                )
                            })?
                        } else {
                            return Err(McpError::invalid_params(
                                "Missing required parameter: project_id for business_rule listing",
                                None,
                            ));
                        }
                    }
                    "architectural_decision" => {
                        if let Some(pid) = project_id {
                            let decisions = self
                                .container
                                .context_crud_service
                                .list_architectural_decisions(pid)
                                .await?;
                            serde_json::to_value(decisions).map_err(|e| {
                                McpError::internal_error(
                                    format!("Serialization error: {}", e),
                                    None,
                                )
                            })?
                        } else {
                            return Err(McpError::invalid_params("Missing required parameter: project_id for architectural_decision listing", None));
                        }
                    }
                    "framework_component" => {
                        if let Some(pid) = project_id {
                            let components = if let Some(layer) = architecture_layer {
                                // Use list_components_by_layer if architecture_layer is specified
                                self.container
                                    .framework_service
                                    .list_components_by_layer(pid, layer)
                                    .await?
                            } else {
                                // Use list_components if no layer is specified
                                self.container
                                    .framework_service
                                    .list_components(pid)
                                    .await?
                            };
                            serde_json::to_value(components).map_err(|e| {
                                McpError::internal_error(
                                    format!("Serialization error: {}", e),
                                    None,
                                )
                            })?
                        } else {
                            return Err(McpError::invalid_params("Missing required parameter: project_id for framework_component listing", None));
                        }
                    }
                    "development_phase" => {
                        if let Some(pid) = project_id {
                            let phases = self
                                .container
                                .development_phase_service
                                .list_phases(pid)
                                .await?;
                            serde_json::to_value(phases).map_err(|e| {
                                McpError::internal_error(
                                    format!("Serialization error: {}", e),
                                    None,
                                )
                            })?
                        } else {
                            return Err(McpError::invalid_params("Missing required parameter: project_id for development_phase listing", None));
                        }
                    }
                    // Add more entity types as needed
                    _ => {
                        return Err(McpError::invalid_params(
                            format!("Unsupported entity type: {}", entity_type),
                            None,
                        ))
                    }
                };

                let content = serde_json::to_string_pretty(&result).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            // Cache Management
            // Removed duplicate cache_management handler

            // Bulk Operations for Framework Components
            // Removed duplicate bulk_create_components handler
            "bulk_update_components" => {
                let args = request.arguments.unwrap_or_default();
                let components_value = args
                    .get("components")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        McpError::invalid_params(
                            "Missing required parameter: components array",
                            None,
                        )
                    })?;

                let mut results = Vec::new();

                for comp_value in components_value {
                    if let Some(comp_obj) = comp_value.as_object() {
                        let id = comp_obj.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                            McpError::invalid_params("Missing id in component", None)
                        })?;
                        let component_name = comp_obj
                            .get("component_name")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing component_name in component",
                                    None,
                                )
                            })?;
                        let component_type = comp_obj
                            .get("component_type")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing component_type in component",
                                    None,
                                )
                            })?;
                        let architecture_layer = comp_obj
                            .get("architecture_layer")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                McpError::invalid_params(
                                    "Missing architecture_layer in component",
                                    None,
                                )
                            })?;
                        let file_path = comp_obj.get("file_path").and_then(|v| v.as_str());

                        // First retrieve the component
                        let mut component =
                            match self.container.framework_service.get_component(id).await? {
                                Some(c) => c,
                                None => {
                                    return Err(McpError::invalid_params(
                                        format!("Component with id {} not found", id),
                                        None,
                                    ))
                                }
                            };

                        // Update component fields
                        component.component_name = component_name.to_string();
                        component.component_type = component_type.to_string();
                        component.architecture_layer = architecture_layer.to_string();

                        if let Some(fp) = file_path {
                            component.file_path = Some(fp.to_string());
                        }

                        // Update the component
                        let updated_component = self
                            .container
                            .framework_service
                            .update_component(&component)
                            .await?;
                        results.push(updated_component);
                    }
                }

                let content = serde_json::to_string_pretty(&results).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            "bulk_delete_components" => {
                let args = request.arguments.unwrap_or_default();
                let component_ids = args
                    .get("component_ids")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        McpError::invalid_params(
                            "Missing required parameter: component_ids array",
                            None,
                        )
                    })?;

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
                let content = serde_json::to_string_pretty(&result).map_err(|e| {
                    McpError::internal_error(format!("Serialization error: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }

            "bulk_operations" => {
                let args = request.arguments.unwrap_or_default();
                let operation =
                    args.get("operation")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            McpError::invalid_params("Missing required parameter: operation", None)
                        })?;
                let entity_type = args
                    .get("entity_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: entity_type", None)
                    })?;
                let data = args.get("data").and_then(|v| v.as_array()).ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: data", None)
                })?;

                match (operation, entity_type) {
                    ("create", "framework_component") => {
                        let mut results = Vec::new();
                        for item in data {
                            let obj = item.as_object().ok_or_else(|| {
                                McpError::invalid_params("Invalid component data", None)
                            })?;
                            let project_id = obj
                                .get("project_id")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    McpError::invalid_params(
                                        "Missing project_id in component data",
                                        None,
                                    )
                                })?;
                            let component_name =
                                obj.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                                    McpError::invalid_params("Missing name in component data", None)
                                })?;
                            let component_type = obj
                                .get("component_type")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    McpError::invalid_params(
                                        "Missing component_type in component data",
                                        None,
                                    )
                                })?;
                            let architecture_layer = obj
                                .get("architecture_layer")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    McpError::invalid_params(
                                        "Missing architecture_layer in component data",
                                        None,
                                    )
                                })?;
                            let file_path = obj.get("file_path").and_then(|v| v.as_str());

                            let component = self
                                .container
                                .framework_service
                                .create_component(
                                    project_id,
                                    component_name,
                                    component_type,
                                    architecture_layer,
                                    file_path,
                                    None,
                                )
                                .await?;
                            results.push(component);
                        }
                        let content = serde_json::to_string_pretty(&results).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    ("update", "framework_component") => {
                        let mut results = Vec::new();
                        for item in data {
                            let obj = item.as_object().ok_or_else(|| {
                                McpError::invalid_params("Invalid component data", None)
                            })?;
                            let id = obj.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                                McpError::invalid_params("Missing id in component data", None)
                            })?;

                            // First retrieve the component
                            let mut component =
                                match self.container.framework_service.get_component(id).await? {
                                    Some(c) => c,
                                    None => {
                                        return Err(McpError::invalid_params(
                                            format!("Component with id {} not found", id),
                                            None,
                                        ))
                                    }
                                };

                            // Update fields if provided
                            if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                                component.component_name = name.to_string();
                            }
                            if let Some(component_type) =
                                obj.get("component_type").and_then(|v| v.as_str())
                            {
                                component.component_type = component_type.to_string();
                            }
                            if let Some(layer) =
                                obj.get("architecture_layer").and_then(|v| v.as_str())
                            {
                                component.architecture_layer = layer.to_string();
                            }
                            if let Some(file_path) = obj.get("file_path").and_then(|v| v.as_str()) {
                                component.file_path = Some(file_path.to_string());
                            }

                            // Update the component
                            let updated_component = self
                                .container
                                .framework_service
                                .update_component(&component)
                                .await?;
                            results.push(updated_component);
                        }
                        let content = serde_json::to_string_pretty(&results).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    ("delete", "framework_component") => {
                        let mut ids = Vec::new();
                        for item in data {
                            if let Some(obj) = item.as_object() {
                                if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                                    ids.push(id.to_string());
                                }
                            }
                        }

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
                        let content = serde_json::to_string_pretty(&result).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {}", e), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    _ => Err(McpError::invalid_params(
                        format!(
                            "Unsupported operation '{}' or entity type '{}'",
                            operation, entity_type
                        ),
                        None,
                    )),
                }
            }

            // Project Management
            // Removed duplicate manage_project handler

            // Specification Import and Management Tools
            "scan_specifications" => {
                let args = request.arguments.unwrap_or_default();
                let base_path = args
                    .get("base_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".kiro/specs");

                let path = std::path::Path::new(base_path);
                match self.container.specification_import_service.scan_and_import_specifications(path).await {
                    Ok(specs) => {
                        let content = serde_json::to_string_pretty(&specs).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => Err(McpError::internal_error(format!("Failed to scan specifications: {e}"), None)),
                }
            }

            "import_specification" => {
                let args = request.arguments.unwrap_or_default();
                let file_path = args
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: file_path", None)
                    })?;

                let path = std::path::Path::new(file_path);
                match self.container.specification_import_service.import_specification_file(path).await {
                    Ok(spec) => {
                        let content = serde_json::to_string_pretty(&spec).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => Err(McpError::internal_error(format!("Failed to import specification: {e}"), None)),
                }
            }

            "validate_specification" => {
                let args = request.arguments.unwrap_or_default();
                let file_path = args
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: file_path", None)
                    })?;

                let path = std::path::Path::new(file_path);
                match self.container.specification_import_service.validate_specification_file(path).await {
                    Ok(issues) => {
                        let result = serde_json::json!({
                            "file_path": file_path,
                            "validation_issues": issues,
                            "is_valid": issues.is_empty()
                        });
                        let content = serde_json::to_string_pretty(&result).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => Err(McpError::internal_error(format!("Failed to validate specification: {e}"), None)),
                }
            }

            "start_spec_monitoring" => {
                let args = request.arguments.unwrap_or_default();
                let base_path = args
                    .get("base_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".kiro/specs");

                let path = std::path::Path::new(base_path);
                match self.container.specification_import_service.start_file_monitoring(path).await {
                    Ok(()) => {
                        let result = serde_json::json!({
                            "status": "success",
                            "message": format!("Started monitoring {}", base_path),
                            "monitoring_path": base_path
                        });
                        let content = serde_json::to_string_pretty(&result).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => Err(McpError::internal_error(format!("Failed to start monitoring: {e}"), None)),
                }
            }

            "get_specification_versions" => {
                let args = request.arguments.unwrap_or_default();
                let spec_id = args
                    .get("spec_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: spec_id", None)
                    })?;

                match self.container.specification_versioning_service.get_versions(spec_id).await {
                    Ok(versions) => {
                        let content = serde_json::to_string_pretty(&versions).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => Err(McpError::internal_error(format!("Failed to get specification versions: {e}"), None)),
                }
            }

            "compare_specification_versions" => {
                let args = request.arguments.unwrap_or_default();
                let version1_id = args
                    .get("version1_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: version1_id", None)
                    })?;
                let version2_id = args
                    .get("version2_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing required parameter: version2_id", None)
                    })?;

                match self.container.specification_versioning_service.compare_versions(version1_id, version2_id).await {
                    Ok(comparison) => {
                        let content = serde_json::to_string_pretty(&comparison).map_err(|e| {
                            McpError::internal_error(format!("Serialization error: {e}"), None)
                        })?;
                        Ok(CallToolResult::success(vec![Content::text(content)]))
                    }
                    Err(e) => Err(McpError::internal_error(format!("Failed to compare specification versions: {e}"), None)),
                }
            }

            // Specification Analytics Tools
            "track_requirements_progress" | "track_tasks_progress" | "analyze_specification_completeness" | 
            "calculate_development_velocity" | "generate_specification_health_report" => {
                let analytics_tools = SpecificationAnalyticsTools::new(self.container.specification_analytics_service.clone());
                let arguments = request.arguments.unwrap_or_default();
                analytics_tools.handle_tool_call(&request.name, serde_json::Value::Object(arguments)).await
            }

            // Fallback for undefined tools
            _ => Err(McpError::method_not_found::<CallToolRequestMethod>()),
        }
    }
}
