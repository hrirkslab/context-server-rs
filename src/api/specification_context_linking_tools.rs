use crate::services::specification_context_linking_service::{
    SpecificationContextLinkingService, ContextLink, TaskProgressUpdate, ContextImpactAnalysis,
    SyncResult, ContextSuggestion, SpecificationChanges
};
use rmcp::model::{CallToolRequest, CallToolResult, ErrorData as McpError, ToolInfo};
use serde_json::{json, Value};
use std::sync::Arc;

/// MCP tools for specification-context linking operations
pub struct SpecificationContextLinkingTools {
    service: Arc<dyn SpecificationContextLinkingService>,
}

impl SpecificationContextLinkingTools {
    pub fn new(service: Arc<dyn SpecificationContextLinkingService>) -> Self {
        Self { service }
    }

    /// Get tool definitions for MCP
    pub fn get_tool_definitions() -> Vec<ToolInfo> {
        vec![
            ToolInfo {
                name: "link_requirements_to_context".to_string(),
                description: "Automatically link all requirements in a specification to relevant context items".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "spec_id": {
                            "type": "string",
                            "description": "ID of the specification containing requirements to link"
                        }
                    },
                    "required": ["spec_id"]
                }),
            },
            ToolInfo {
                name: "link_requirement_to_context".to_string(),
                description: "Link a specific requirement to relevant context items based on content analysis".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "requirement_id": {
                            "type": "string",
                            "description": "ID of the requirement to link to context"
                        }
                    },
                    "required": ["requirement_id"]
                }),
            },
            ToolInfo {
                name: "update_task_progress".to_string(),
                description: "Update task progress and sync with related context items".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "ID of the task to update"
                        },
                        "progress": {
                            "type": "number",
                            "minimum": 0.0,
                            "maximum": 1.0,
                            "description": "Progress value between 0.0 and 1.0"
                        }
                    },
                    "required": ["task_id", "progress"]
                }),
            },
            ToolInfo {
                name: "analyze_context_impact".to_string(),
                description: "Analyze the impact of context changes on specifications, requirements, and tasks".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "context_id": {
                            "type": "string",
                            "description": "ID of the context item to analyze"
                        }
                    },
                    "required": ["context_id"]
                }),
            },
            ToolInfo {
                name: "sync_specification_context".to_string(),
                description: "Perform bidirectional synchronization between specifications and context".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "spec_id": {
                            "type": "string",
                            "description": "ID of the specification to synchronize"
                        }
                    },
                    "required": ["spec_id"]
                }),
            },
            ToolInfo {
                name: "suggest_context_links".to_string(),
                description: "Get suggestions for linking context items to a requirement based on semantic similarity".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "requirement_id": {
                            "type": "string",
                            "description": "ID of the requirement to get context suggestions for"
                        }
                    },
                    "required": ["requirement_id"]
                }),
            },
            ToolInfo {
                name: "track_specification_changes".to_string(),
                description: "Track changes in specifications and update related context items".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "spec_id": {
                            "type": "string",
                            "description": "ID of the specification that changed"
                        },
                        "changes": {
                            "type": "object",
                            "description": "Details of the changes made to the specification",
                            "properties": {
                                "change_type": {
                                    "type": "string",
                                    "enum": ["RequirementAdded", "RequirementModified", "RequirementRemoved", "TaskAdded", "TaskModified", "TaskRemoved", "StatusChanged", "ContentUpdated", "MetadataChanged"]
                                },
                                "changes": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "field": {"type": "string"},
                                            "old_value": {},
                                            "new_value": {},
                                            "description": {"type": "string"}
                                        },
                                        "required": ["field", "description"]
                                    }
                                },
                                "changed_by": {"type": "string"}
                            },
                            "required": ["change_type", "changes"]
                        }
                    },
                    "required": ["spec_id", "changes"]
                }),
            },
        ]
    }

    /// Handle MCP tool calls
    pub async fn handle_tool_call(&self, request: CallToolRequest) -> Result<CallToolResult, McpError> {
        match request.params.name.as_str() {
            "link_requirements_to_context" => self.handle_link_requirements_to_context(request).await,
            "link_requirement_to_context" => self.handle_link_requirement_to_context(request).await,
            "update_task_progress" => self.handle_update_task_progress(request).await,
            "analyze_context_impact" => self.handle_analyze_context_impact(request).await,
            "sync_specification_context" => self.handle_sync_specification_context(request).await,
            "suggest_context_links" => self.handle_suggest_context_links(request).await,
            "track_specification_changes" => self.handle_track_specification_changes(request).await,
            _ => Err(McpError::method_not_found(
                format!("Unknown tool: {}", request.params.name),
                None,
            )),
        }
    }

    async fn handle_link_requirements_to_context(&self, request: CallToolRequest) -> Result<CallToolResult, McpError> {
        let spec_id = request.params.arguments
            .get("spec_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid spec_id parameter", None))?;

        let links = self.service.link_requirements_to_context(spec_id).await?;

        Ok(CallToolResult {
            content: vec![rmcp::model::TextContent {
                type_: "text".to_string(),
                text: json!({
                    "message": format!("Successfully linked {} context items to requirements in specification {}", links.len(), spec_id),
                    "links": links,
                    "summary": {
                        "total_links": links.len(),
                        "auto_detected": links.iter().filter(|l| l.auto_detected).count(),
                        "manual": links.iter().filter(|l| !l.auto_detected).count(),
                        "average_confidence": if links.is_empty() { 0.0 } else { 
                            links.iter().map(|l| l.confidence).sum::<f64>() / links.len() as f64 
                        }
                    }
                }).to_string(),
            }],
            is_error: Some(false),
        })
    }

    async fn handle_link_requirement_to_context(&self, request: CallToolRequest) -> Result<CallToolResult, McpError> {
        let requirement_id = request.params.arguments
            .get("requirement_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid requirement_id parameter", None))?;

        let links = self.service.link_requirement_to_context(requirement_id).await?;

        Ok(CallToolResult {
            content: vec![rmcp::model::TextContent {
                type_: "text".to_string(),
                text: json!({
                    "message": format!("Successfully linked {} context items to requirement {}", links.len(), requirement_id),
                    "links": links,
                    "summary": {
                        "total_links": links.len(),
                        "link_types": {
                            "implements": links.iter().filter(|l| l.link_type == crate::services::specification_context_linking_service::LinkType::Implements).count(),
                            "validates": links.iter().filter(|l| l.link_type == crate::services::specification_context_linking_service::LinkType::Validates).count(),
                            "references": links.iter().filter(|l| l.link_type == crate::services::specification_context_linking_service::LinkType::References).count(),
                            "constrains": links.iter().filter(|l| l.link_type == crate::services::specification_context_linking_service::LinkType::Constrains).count(),
                        },
                        "average_confidence": if links.is_empty() { 0.0 } else { 
                            links.iter().map(|l| l.confidence).sum::<f64>() / links.len() as f64 
                        }
                    }
                }).to_string(),
            }],
            is_error: Some(false),
        })
    }

    async fn handle_update_task_progress(&self, request: CallToolRequest) -> Result<CallToolResult, McpError> {
        let task_id = request.params.arguments
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid task_id parameter", None))?;

        let progress = request.params.arguments
            .get("progress")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid progress parameter", None))?;

        if progress < 0.0 || progress > 1.0 {
            return Err(McpError::invalid_params("Progress must be between 0.0 and 1.0", None));
        }

        let update = self.service.update_task_progress(task_id, progress).await?;

        Ok(CallToolResult {
            content: vec![rmcp::model::TextContent {
                type_: "text".to_string(),
                text: json!({
                    "message": format!("Successfully updated task {} progress from {:.1}% to {:.1}%", 
                        task_id, update.old_progress * 100.0, update.new_progress * 100.0),
                    "update": update,
                    "summary": {
                        "progress_change": (update.new_progress - update.old_progress) * 100.0,
                        "context_updates": update.context_updates.len(),
                        "related_tasks_affected": update.related_tasks_affected.len(),
                    }
                }).to_string(),
            }],
            is_error: Some(false),
        })
    }

    async fn handle_analyze_context_impact(&self, request: CallToolRequest) -> Result<CallToolResult, McpError> {
        let context_id = request.params.arguments
            .get("context_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid context_id parameter", None))?;

        let analysis = self.service.analyze_context_impact(context_id).await?;

        Ok(CallToolResult {
            content: vec![rmcp::model::TextContent {
                type_: "text".to_string(),
                text: json!({
                    "message": format!("Context impact analysis completed for {}", context_id),
                    "analysis": analysis,
                    "summary": {
                        "risk_level": analysis.risk_level,
                        "affected_specifications": analysis.affected_specifications.len(),
                        "affected_requirements": analysis.affected_requirements.len(),
                        "affected_tasks": analysis.affected_tasks.len(),
                        "recommendations_count": analysis.recommendations.len(),
                    }
                }).to_string(),
            }],
            is_error: Some(false),
        })
    }

    async fn handle_sync_specification_context(&self, request: CallToolRequest) -> Result<CallToolResult, McpError> {
        let spec_id = request.params.arguments
            .get("spec_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid spec_id parameter", None))?;

        let sync_result = self.service.sync_specification_context(spec_id).await?;

        Ok(CallToolResult {
            content: vec![rmcp::model::TextContent {
                type_: "text".to_string(),
                text: json!({
                    "message": format!("Synchronization completed for specification {}", spec_id),
                    "sync_result": sync_result,
                    "summary": {
                        "sync_status": sync_result.sync_status,
                        "changes_applied": sync_result.changes_applied.len(),
                        "conflicts_detected": sync_result.conflicts_detected.len(),
                        "sync_type": sync_result.sync_type,
                    }
                }).to_string(),
            }],
            is_error: Some(false),
        })
    }

    async fn handle_suggest_context_links(&self, request: CallToolRequest) -> Result<CallToolResult, McpError> {
        let requirement_id = request.params.arguments
            .get("requirement_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid requirement_id parameter", None))?;

        let suggestions = self.service.suggest_context_links(requirement_id).await?;

        Ok(CallToolResult {
            content: vec![rmcp::model::TextContent {
                type_: "text".to_string(),
                text: json!({
                    "message": format!("Found {} context link suggestions for requirement {}", suggestions.len(), requirement_id),
                    "suggestions": suggestions,
                    "summary": {
                        "total_suggestions": suggestions.len(),
                        "high_relevance": suggestions.iter().filter(|s| s.relevance_score > 0.7).count(),
                        "medium_relevance": suggestions.iter().filter(|s| s.relevance_score > 0.4 && s.relevance_score <= 0.7).count(),
                        "low_relevance": suggestions.iter().filter(|s| s.relevance_score <= 0.4).count(),
                        "context_types": {
                            "business_rule": suggestions.iter().filter(|s| s.context_type == crate::models::enhanced_context::ContextType::BusinessRule).count(),
                            "architectural_decision": suggestions.iter().filter(|s| s.context_type == crate::models::enhanced_context::ContextType::ArchitecturalDecision).count(),
                            "performance_requirement": suggestions.iter().filter(|s| s.context_type == crate::models::enhanced_context::ContextType::PerformanceRequirement).count(),
                            "security_policy": suggestions.iter().filter(|s| s.context_type == crate::models::enhanced_context::ContextType::SecurityPolicy).count(),
                        }
                    }
                }).to_string(),
            }],
            is_error: Some(false),
        })
    }

    async fn handle_track_specification_changes(&self, request: CallToolRequest) -> Result<CallToolResult, McpError> {
        let spec_id = request.params.arguments
            .get("spec_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid spec_id parameter", None))?;

        let changes_value = request.params.arguments
            .get("changes")
            .ok_or_else(|| McpError::invalid_params("Missing changes parameter", None))?;

        let changes: SpecificationChanges = serde_json::from_value(changes_value.clone())
            .map_err(|e| McpError::invalid_params(format!("Invalid changes format: {}", e), None))?;

        self.service.track_specification_changes(spec_id, changes).await?;

        Ok(CallToolResult {
            content: vec![rmcp::model::TextContent {
                type_: "text".to_string(),
                text: json!({
                    "message": format!("Successfully tracked changes for specification {}", spec_id),
                    "spec_id": spec_id,
                    "changes_tracked": true
                }).to_string(),
            }],
            is_error: Some(false),
        })
    }
}