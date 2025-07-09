use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use rmcp::{
    model::*,
    model::ErrorData as McpError,
    handler::server::ServerHandler,
};
use crate::models::context::*;
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
            _ => {
                tracing::warn!("Unknown tool requested: {}", request.name);
                Err(McpError::method_not_found::<CallToolRequestMethod>())
            }
        }
    }
}
