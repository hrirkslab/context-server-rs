use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use rmcp::{
    model::*,
    model::ErrorData as McpError,
    handler::server::ServerHandler,
};
use crate::models::context::*;
use crate::models::flutter::*;
use anyhow::Result;
use uuid::Uuid;

/// MCP Context Server that provides curated project context to AI agents
#[derive(Clone)]
pub struct ContextMcpServer {
    db: Arc<Mutex<Connection>>,
}

impl ContextMcpServer {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
        })
    }

    /// Query context based on feature area and task type
    async fn query_context(&self, project_id: &str, feature_area: &str, _task_type: &str, _components: &[String]) -> Result<ContextQueryResult, McpError> {
        let db = self.db.lock().unwrap();
        
        // Query business rules
        let mut business_rules = Vec::new();
        let mut stmt = db.prepare("SELECT id, project_id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at FROM business_rules WHERE project_id = ? AND (domain_area = ? OR domain_area IS NULL)").map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        let rules = stmt.query_map([project_id, feature_area], |row| {
            Ok(BusinessRule {
                id: row.get(0)?,
                project_id: row.get(1)?,
                rule_name: row.get(2)?,
                description: row.get(3)?,
                domain_area: row.get(4)?,
                implementation_pattern: row.get(5)?,
                constraints: row.get(6)?,
                examples: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for rule in rules {
            match rule {
                Ok(rule) => business_rules.push(rule),
                Err(e) => tracing::warn!("Failed to parse business rule: {}", e),
            }
        }

        // Query architectural decisions
        let mut architectural_decisions = Vec::new();
        let mut stmt = db.prepare("SELECT id, project_id, decision_title, context, decision, consequences, alternatives_considered, status, created_at FROM architectural_decisions WHERE project_id = ?").map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        let decisions = stmt.query_map([project_id], |row| {
            Ok(ArchitecturalDecision {
                id: row.get(0)?,
                project_id: row.get(1)?,
                decision_title: row.get(2)?,
                context: row.get(3)?,
                decision: row.get(4)?,
                consequences: row.get(5)?,
                alternatives_considered: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for decision in decisions {
            match decision {
                Ok(decision) => architectural_decisions.push(decision),
                Err(e) => tracing::warn!("Failed to parse architectural decision: {}", e),
            }
        }

        // Query performance requirements
        let mut performance_requirements = Vec::new();
        let mut stmt = db.prepare("SELECT id, project_id, component_area, requirement_type, target_value, optimization_patterns, avoid_patterns, created_at FROM performance_requirements WHERE project_id = ?").map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        let requirements = stmt.query_map([project_id], |row| {
            Ok(PerformanceRequirement {
                id: row.get(0)?,
                project_id: row.get(1)?,
                component_area: row.get(2)?,
                requirement_type: row.get(3)?,
                target_value: row.get(4)?,
                optimization_patterns: row.get(5)?,
                avoid_patterns: row.get(6)?,
                created_at: row.get(7)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for requirement in requirements {
            match requirement {
                Ok(requirement) => performance_requirements.push(requirement),
                Err(e) => tracing::warn!("Failed to parse performance requirement: {}", e),
            }
        }

        Ok(ContextQueryResult {
            business_rules,
            architectural_decisions,
            performance_requirements,
            security_policies: Vec::new(), // TODO: Implement security policies query
            project_conventions: Vec::new(), // TODO: Implement project conventions query
        })
    }

    /// List all projects
    async fn list_projects(&self) -> Result<Vec<Project>, McpError> {
        let db = self.db.lock().unwrap();
        let mut projects = Vec::new();
        
        let mut stmt = db.prepare("SELECT id, name, description, repository_url, created_at, updated_at FROM projects").map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        let project_rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                repository_url: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for project in project_rows {
            match project {
                Ok(project) => projects.push(project),
                Err(e) => tracing::warn!("Failed to parse project: {}", e),
            }
        }

        Ok(projects)
    }

    /// Create a new project
    async fn create_project(&self, name: &str, description: Option<&str>, repository_url: Option<&str>) -> Result<Project, McpError> {
        let db = self.db.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        db.execute(
            "INSERT INTO projects (id, name, description, repository_url, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            (
                &id,
                name,
                description,
                repository_url,
                &now,
                &now,
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(Project {
            id,
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            repository_url: repository_url.map(|s| s.to_string()),
            created_at: Some(now.clone()),
            updated_at: Some(now),
        })
    }

    /// Create a Flutter component
    async fn create_flutter_component(&self, project_id: &str, component_name: &str, component_type: &str, architecture_layer: &str, file_path: Option<&str>) -> Result<FlutterComponent, McpError> {
        let db = self.db.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        db.execute(
            "INSERT INTO flutter_components (id, project_id, component_name, component_type, architecture_layer, file_path, dependencies, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                &id,
                project_id,
                component_name,
                component_type,
                architecture_layer,
                file_path,
                "[]", // Empty JSON array for dependencies
                &now,
                &now,
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        // Parse enums from strings
        let component_type_enum = match component_type {
            "widget" => ComponentType::Widget,
            "provider" => ComponentType::Provider,
            "service" => ComponentType::Service,
            "repository" => ComponentType::Repository,
            "model" => ComponentType::Model,
            "utility" => ComponentType::Utility,
            _ => ComponentType::Widget,
        };

        let architecture_layer_enum = match architecture_layer {
            "presentation" => crate::models::flutter::ArchitectureLayer::Presentation,
            "domain" => crate::models::flutter::ArchitectureLayer::Domain,
            "data" => crate::models::flutter::ArchitectureLayer::Data,
            "core" => crate::models::flutter::ArchitectureLayer::Core,
            _ => crate::models::flutter::ArchitectureLayer::Presentation,
        };

        Ok(FlutterComponent {
            id,
            project_id: project_id.to_string(),
            component_name: component_name.to_string(),
            component_type: component_type_enum,
            architecture_layer: architecture_layer_enum,
            file_path: file_path.map(|s| s.to_string()),
            dependencies: Vec::new(),
            riverpod_scope: None,
            widget_type: None,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        })
    }

    /// List Flutter components for a project
    async fn list_flutter_components(&self, project_id: &str) -> Result<Vec<FlutterComponent>, McpError> {
        let db = self.db.lock().unwrap();
        let mut components = Vec::new();
        
        let mut stmt = db.prepare("SELECT id, project_id, component_name, component_type, architecture_layer, file_path, dependencies, riverpod_scope, widget_type, created_at, updated_at FROM flutter_components WHERE project_id = ?").map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        let component_rows = stmt.query_map([project_id], |row| {
            let component_type_str: String = row.get(3)?;
            let architecture_layer_str: String = row.get(4)?;
            let dependencies_str: String = row.get(6)?;
            
            let component_type = match component_type_str.as_str() {
                "widget" => ComponentType::Widget,
                "provider" => ComponentType::Provider,
                "service" => ComponentType::Service,
                "repository" => ComponentType::Repository,
                "model" => ComponentType::Model,
                "utility" => ComponentType::Utility,
                _ => ComponentType::Widget,
            };

            let architecture_layer = match architecture_layer_str.as_str() {
                "presentation" => crate::models::flutter::ArchitectureLayer::Presentation,
                "domain" => crate::models::flutter::ArchitectureLayer::Domain,
                "data" => crate::models::flutter::ArchitectureLayer::Data,
                "core" => crate::models::flutter::ArchitectureLayer::Core,
                _ => crate::models::flutter::ArchitectureLayer::Presentation,
            };

            let dependencies: Vec<String> = serde_json::from_str(&dependencies_str).unwrap_or_default();

            Ok(FlutterComponent {
                id: row.get(0)?,
                project_id: row.get(1)?,
                component_name: row.get(2)?,
                component_type,
                architecture_layer,
                file_path: row.get(5)?,
                dependencies,
                riverpod_scope: None, // TODO: Parse from database
                widget_type: None, // TODO: Parse from database
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for component in component_rows {
            match component {
                Ok(component) => components.push(component),
                Err(e) => tracing::warn!("Failed to parse Flutter component: {}", e),
            }
        }

        Ok(components)
    }

    /// Create a development phase
    async fn create_development_phase(&self, project_id: &str, phase_name: &str, phase_order: i32, description: Option<&str>) -> Result<DevelopmentPhase, McpError> {
        let db = self.db.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        db.execute(
            "INSERT INTO development_phases (id, project_id, phase_name, phase_order, status, description, completion_criteria, dependencies, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                &id,
                project_id,
                phase_name,
                phase_order,
                "pending",
                description,
                "[]", // Empty JSON array for completion criteria
                "[]", // Empty JSON array for dependencies
                &now,
            ),
        ).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        Ok(DevelopmentPhase {
            id,
            project_id: project_id.to_string(),
            phase_name: phase_name.to_string(),
            phase_order,
            status: PhaseStatus::Pending,
            description: description.map(|s| s.to_string()),
            completion_criteria: Vec::new(),
            dependencies: Vec::new(),
            started_at: None,
            completed_at: None,
            created_at: Some(now),
        })
    }

    /// List development phases for a project
    async fn list_development_phases(&self, project_id: &str) -> Result<Vec<DevelopmentPhase>, McpError> {
        let db = self.db.lock().unwrap();
        let mut phases = Vec::new();
        
        let mut stmt = db.prepare("SELECT id, project_id, phase_name, phase_order, status, description, completion_criteria, dependencies, started_at, completed_at, created_at FROM development_phases WHERE project_id = ? ORDER BY phase_order").map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;
        let phase_rows = stmt.query_map([project_id], |row| {
            let status_str: String = row.get(4)?;
            let status = match status_str.as_str() {
                "pending" => PhaseStatus::Pending,
                "in_progress" => PhaseStatus::InProgress,
                "completed" => PhaseStatus::Completed,
                "blocked" => PhaseStatus::Blocked,
                _ => PhaseStatus::Pending,
            };

            let completion_criteria_str: String = row.get(6)?;
            let dependencies_str: String = row.get(7)?;
            let completion_criteria: Vec<String> = serde_json::from_str(&completion_criteria_str).unwrap_or_default();
            let dependencies: Vec<String> = serde_json::from_str(&dependencies_str).unwrap_or_default();

            Ok(DevelopmentPhase {
                id: row.get(0)?,
                project_id: row.get(1)?,
                phase_name: row.get(2)?,
                phase_order: row.get(3)?,
                status,
                description: row.get(5)?,
                completion_criteria,
                dependencies,
                started_at: row.get(8)?,
                completed_at: row.get(9)?,
                created_at: row.get(10)?,
            })
        }).map_err(|e| McpError::internal_error(format!("Database error: {}", e), None))?;

        for phase in phase_rows {
            match phase {
                Ok(phase) => phases.push(phase),
                Err(e) => tracing::warn!("Failed to parse development phase: {}", e),
            }
        }

        Ok(phases)
    }

    /// Validate architecture dependencies for Flutter project
    async fn validate_architecture(&self, project_id: &str) -> Result<Vec<String>, McpError> {
        let mut violations = Vec::new();
        
        // Get all components for the project
        let components = self.list_flutter_components(project_id).await?;
        
        // Check architecture layer violations
        for component in &components {
            match component.architecture_layer {
                crate::models::flutter::ArchitectureLayer::Presentation => {
                    // Presentation layer should not directly import from data layer
                    for dep in &component.dependencies {
                        if dep.contains("data/") && !dep.contains("domain/") {
                            violations.push(format!(
                                "Architecture violation: {} (presentation) directly imports from data layer: {}",
                                component.component_name, dep
                            ));
                        }
                    }
                }
                crate::models::flutter::ArchitectureLayer::Domain => {
                    // Domain layer should not import from presentation or data layers
                    for dep in &component.dependencies {
                        if dep.contains("presentation/") || dep.contains("data/") {
                            violations.push(format!(
                                "Architecture violation: {} (domain) imports from {}: {}",
                                component.component_name, 
                                if dep.contains("presentation/") { "presentation" } else { "data" },
                                dep
                            ));
                        }
                    }
                }
                _ => {} // Data and core layers have fewer restrictions
            }
        }
        
        Ok(violations)
    }

    /// Get server capabilities and available features
    async fn get_server_capabilities(&self) -> Result<ServerCapabilitiesInfo, McpError> {
        Ok(ServerCapabilitiesInfo {
            server_info: ServerMetadata {
                name: "context-server-rs".to_string(),
                version: "0.1.0".to_string(),
                description: "Flutter-specific MCP Context Server for AI-assisted development".to_string(),
                config_directory: "~/config/context-server-rs/".to_string(),
            },
            features: vec![
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
                    name: "Privacy-First Validation".to_string(),
                    description: "Monitor for external network calls and enforce local-only constraints".to_string(),
                    status: FeatureStatus::Framework,
                    tools: vec!["create_privacy_rule".to_string(), "list_privacy_violations".to_string()],
                },
                FeatureInfo {
                    name: "Code Generation Templates".to_string(),
                    description: "Generate boilerplate for widgets, providers, repositories".to_string(),
                    status: FeatureStatus::Planned,
                    tools: vec!["generate_widget_template".to_string(), "generate_provider_template".to_string()],
                },
                FeatureInfo {
                    name: "LLM Model Context".to_string(),
                    description: "Track model configurations, performance metrics, and inference patterns".to_string(),
                    status: FeatureStatus::Framework,
                    tools: vec!["create_model_context".to_string(), "list_model_metrics".to_string()],
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
                TableInfo {
                    name: "privacy_rules".to_string(),
                    description: "Rules for detecting external API calls and privacy violations".to_string(),
                    primary_fields: vec!["rule_name".to_string(), "rule_type".to_string(), "pattern".to_string()],
                    example_use: "Detect http imports or external network calls".to_string(),
                },
                TableInfo {
                    name: "business_rules".to_string(),
                    description: "Domain-specific business logic and implementation patterns".to_string(),
                    primary_fields: vec!["rule_name".to_string(), "domain_area".to_string(), "implementation_pattern".to_string()],
                    example_use: "Store chat message validation rules".to_string(),
                },
                TableInfo {
                    name: "architectural_decisions".to_string(),
                    description: "ADRs (Architecture Decision Records) for the project".to_string(),
                    primary_fields: vec!["decision_title".to_string(), "context".to_string(), "decision".to_string()],
                    example_use: "Document decision to use Riverpod for state management".to_string(),
                },
                TableInfo {
                    name: "model_context".to_string(),
                    description: "LLM model configurations, performance metrics, and settings".to_string(),
                    primary_fields: vec!["model_name".to_string(), "model_type".to_string(), "performance_metrics".to_string()],
                    example_use: "Track Llama model performance and configuration".to_string(),
                },
            ],
            mcp_tools: vec![
                ToolInfo {
                    name: "query_context".to_string(),
                    description: "Query project context by feature area and task type".to_string(),
                    category: "Core".to_string(),
                    required_params: vec!["project_id".to_string(), "feature_area".to_string(), "task_type".to_string()],
                    example_use: "Get authentication context for implementation".to_string(),
                },
                ToolInfo {
                    name: "create_flutter_component".to_string(),
                    description: "Create Flutter component with architecture layer tracking".to_string(),
                    category: "Flutter".to_string(),
                    required_params: vec!["project_id".to_string(), "component_name".to_string(), "component_type".to_string(), "architecture_layer".to_string()],
                    example_use: "Create ChatScreen widget in presentation layer".to_string(),
                },
                ToolInfo {
                    name: "list_flutter_components".to_string(),
                    description: "List all Flutter components with their architecture layers".to_string(),
                    category: "Flutter".to_string(),
                    required_params: vec!["project_id".to_string()],
                    example_use: "Get overview of all widgets, providers, services".to_string(),
                },
                ToolInfo {
                    name: "validate_architecture".to_string(),
                    description: "Check for Clean Architecture dependency violations".to_string(),
                    category: "Flutter".to_string(),
                    required_params: vec!["project_id".to_string()],
                    example_use: "Ensure presentation layer doesn't import data layer".to_string(),
                },
                ToolInfo {
                    name: "create_development_phase".to_string(),
                    description: "Create project development phase with order and dependencies".to_string(),
                    category: "Project Management".to_string(),
                    required_params: vec!["project_id".to_string(), "phase_name".to_string(), "phase_order".to_string()],
                    example_use: "Create 'Chat UI' phase after 'Setup' phase".to_string(),
                },
                ToolInfo {
                    name: "list_development_phases".to_string(),
                    description: "List all project phases in order with status".to_string(),
                    category: "Project Management".to_string(),
                    required_params: vec!["project_id".to_string()],
                    example_use: "See current project progress and next steps".to_string(),
                },
                ToolInfo {
                    name: "get_server_capabilities".to_string(),
                    description: "Get comprehensive information about server features, database tables, and available tools".to_string(),
                    category: "Core".to_string(),
                    required_params: vec![],
                    example_use: "Discover available features and tools on the server".to_string(),
                },
            ],
            usage_examples: vec! [
                UsageExample {
                    scenario: "Setting up LocalChat Flutter Project".to_string(),
                    steps: vec! [
                        "Create project: 'LocalChat Flutter App'".to_string(),
                        "Create phases: Setup → Chat UI → Model Management → Polish".to_string(),
                        "Add components: ChatScreen (widget), ChatProvider (provider), MessageRepository (repository)".to_string(),
                        "Validate architecture to ensure clean dependency flow".to_string(),
                    ],
                },
                UsageExample {
                    scenario: "AI-Assisted Component Creation".to_string(),
                    steps: vec! [
                        "Ask AI: 'Create a new chat message widget'".to_string(),
                        "AI calls create_flutter_component with proper layer".to_string(),
                        "AI queries existing components for consistency".to_string(),
                        "AI validates architecture rules before suggesting code".to_string(),
                    ],
                },
            ],
            recommended_workflow: vec! [
                "1. Create project with create_project".to_string(),
                "2. Set up development phases with create_development_phase".to_string(),
                "3. Add Flutter components as you build with create_flutter_component".to_string(),
                "4. Regularly validate architecture with validate_architecture".to_string(),
                "5. Query context when building new features with query_context".to_string(),
            ],
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ContextQueryResult {
    pub business_rules: Vec<BusinessRule>,
    pub architectural_decisions: Vec<ArchitecturalDecision>,
    pub performance_requirements: Vec<PerformanceRequirement>,
    pub security_policies: Vec<SecurityPolicy>,
    pub project_conventions: Vec<ProjectConvention>,
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
            instructions: Some("Context Server for AI Code Generation. Provides curated project context including business rules, architectural decisions, and conventions to help AI agents generate better code.".to_string()),
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
                description: Some("Query project context based on feature area, task type, and components".into()),
                input_schema: Arc::new(serde_json::json!({
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
                description: Some("List all available projects".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {}
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "create_project".into(),
                description: Some("Create a new project".into()),
                input_schema: Arc::new(serde_json::json!({
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
                description: Some("Create a new Flutter component in the project".into()),
                input_schema: Arc::new(serde_json::json!({
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
                description: Some("List all Flutter components in a project".into()),
                input_schema: Arc::new(serde_json::json!({
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
                name: "create_development_phase".into(),
                description: Some("Create a new development phase for tracking project progress".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {
                            "type": "string",
                            "description": "The ID of the project"
                        },
                        "phase_name": {
                            "type": "string",
                            "description": "The name of the phase (e.g., 'Setup', 'Chat UI', 'Model Management')"
                        },
                        "phase_order": {
                            "type": "integer",
                            "description": "The order of this phase (1, 2, 3, etc.)"
                        },
                        "description": {
                            "type": "string",
                            "description": "Optional description of the phase"
                        }
                    },
                    "required": ["project_id", "phase_name", "phase_order"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "list_development_phases".into(),
                description: Some("List all development phases for a project".into()),
                input_schema: Arc::new(serde_json::json!({
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
                description: Some("Validate Flutter Clean Architecture rules and detect violations".into()),
                input_schema: Arc::new(serde_json::json!({
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
                description: Some("Get comprehensive information about server features, database tables, and available tools".into()),
                input_schema: Arc::new(serde_json::json!({
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
                tracing::debug!("Processing query_context tool");
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

                let result = self.query_context(project_id, feature_area, task_type, &components).await?;
                let content = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("query_context completed successfully");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "list_projects" => {
                tracing::debug!("Processing list_projects tool");
                let projects = self.list_projects().await?;
                let content = serde_json::to_string_pretty(&projects)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("list_projects completed successfully");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "create_project" => {
                tracing::debug!("Processing create_project tool");
                let args = request.arguments.unwrap_or_default();
                let name = args.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("name is required", None))?;
                let description = args.get("description").and_then(|v| v.as_str());
                let repository_url = args.get("repository_url").and_then(|v| v.as_str());

                let project = self.create_project(name, description, repository_url).await?;
                let content = serde_json::to_string_pretty(&project)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("create_project completed successfully");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "create_flutter_component" => {
                tracing::debug!("Processing create_flutter_component tool");
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

                let component = self.create_flutter_component(project_id, component_name, component_type, architecture_layer, file_path).await?;
                let content = serde_json::to_string_pretty(&component)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("create_flutter_component completed successfully");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "list_flutter_components" => {
                tracing::debug!("Processing list_flutter_components tool");
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("project_id is required", None))?;

                let components = self.list_flutter_components(project_id).await?;
                let content = serde_json::to_string_pretty(&components)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("list_flutter_components completed successfully");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "create_development_phase" => {
                tracing::debug!("Processing create_development_phase tool");
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("project_id is required", None))?;
                let phase_name = args.get("phase_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("phase_name is required", None))?;
                let phase_order = args.get("phase_order")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| McpError::invalid_params("phase_order is required", None))? as i32;
                let description = args.get("description").and_then(|v| v.as_str());

                let phase = self.create_development_phase(project_id, phase_name, phase_order, description).await?;
                let content = serde_json::to_string_pretty(&phase)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("create_development_phase completed successfully");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "list_development_phases" => {
                tracing::debug!("Processing list_development_phases tool");
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("project_id is required", None))?;

                let phases = self.list_development_phases(project_id).await?;
                let content = serde_json::to_string_pretty(&phases)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("list_development_phases completed successfully");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "validate_architecture" => {
                tracing::debug!("Processing validate_architecture tool");
                let args = request.arguments.unwrap_or_default();
                let project_id = args.get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("project_id is required", None))?;

                let violations = self.validate_architecture(project_id).await?;
                let content = serde_json::to_string_pretty(&violations)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;

                tracing::debug!("validate_architecture completed successfully");
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            "get_server_capabilities" => {
                tracing::debug!("Processing get_server_capabilities tool");
                
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
