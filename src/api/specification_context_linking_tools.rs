use crate::services::specification_context_linking_service::SpecificationContextLinkingService;
use rmcp::model::{CallToolRequest, CallToolResult, Content, ErrorData as McpError};
use std::sync::Arc;

/// MCP tools for specification-context linking operations
pub struct SpecificationContextLinkingTools {
    service: Arc<dyn SpecificationContextLinkingService>,
}

impl SpecificationContextLinkingTools {
    pub fn new(service: Arc<dyn SpecificationContextLinkingService>) -> Self {
        Self { service }
    }

    /// Handle MCP tool calls for specification context linking
    pub async fn handle_tool_call(
        &self,
        request: CallToolRequest,
    ) -> Result<CallToolResult, McpError> {
        match request.params.name.as_ref() {
            "link_requirements_to_context" => {
                let spec_id = request
                    .params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("spec_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing or invalid spec_id parameter", None)
                    })?;

                let links = self.service.link_requirements_to_context(spec_id).await?;

                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Successfully linked {} context items to requirements in specification {}",
                    links.len(),
                    spec_id
                ))]))
            }

            "link_requirement_to_context" => {
                let requirement_id = request
                    .params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("requirement_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params(
                            "Missing or invalid requirement_id parameter",
                            None,
                        )
                    })?;

                let links = self
                    .service
                    .link_requirement_to_context(requirement_id)
                    .await?;

                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Successfully linked {} context items to requirement {}",
                    links.len(),
                    requirement_id
                ))]))
            }

            "update_task_progress" => {
                let task_id = request
                    .params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("task_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing or invalid task_id parameter", None)
                    })?;

                let progress = request
                    .params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("progress"))
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing or invalid progress parameter", None)
                    })?;

                self.service.update_task_progress(task_id, progress).await?;

                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Successfully updated task {} progress to {:.1}%",
                    task_id,
                    progress * 100.0
                ))]))
            }

            "analyze_context_impact" => {
                let context_id = request
                    .params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("context_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing or invalid context_id parameter", None)
                    })?;

                let analysis = self.service.analyze_context_impact(context_id).await?;

                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Context impact analysis completed for {}. Found {} affected specifications.",
                    context_id,
                    analysis.affected_specifications.len()
                ))]))
            }

            "sync_specification_context" => {
                let spec_id = request
                    .params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("spec_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing or invalid spec_id parameter", None)
                    })?;

                let result = self.service.sync_specification_context(spec_id).await?;

                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Synchronization completed for specification {}. {} changes applied.",
                    spec_id,
                    result.changes_applied.len()
                ))]))
            }

            "suggest_context_links" => {
                let requirement_id = request
                    .params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("requirement_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params(
                            "Missing or invalid requirement_id parameter",
                            None,
                        )
                    })?;

                let suggestions = self.service.suggest_context_links(requirement_id).await?;

                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Found {} context link suggestions for requirement {}",
                    suggestions.len(),
                    requirement_id
                ))]))
            }

            "track_specification_changes" => {
                let spec_id = request
                    .params
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("spec_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing or invalid spec_id parameter", None)
                    })?;

                // For this tool, we'll create a dummy changes object since the method requires it
                let changes = crate::services::specification_context_linking_service::SpecificationChanges {
                    spec_id: spec_id.to_string(),
                    change_type: crate::services::specification_context_linking_service::SpecChangeType::StatusChanged,
                    changes: vec![],
                    changed_at: chrono::Utc::now(),
                    changed_by: None,
                };

                self.service
                    .track_specification_changes(spec_id, changes)
                    .await?;

                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Successfully tracked changes for specification {}.",
                    spec_id
                ))]))
            }

            _ => Err(McpError::method_not_found::<
                rmcp::model::CallToolRequestMethod,
            >()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::specification_context_linking_service::{
        ContextImpactAnalysis, ContextLink, ContextSuggestion, SpecificationChanges,
        SpecificationContextLinkingService, SyncResult,
    };
    use async_trait::async_trait;
    use rmcp::model::ErrorData as McpError;

    struct MockSpecificationContextLinkingService;

    #[async_trait]
    impl SpecificationContextLinkingService for MockSpecificationContextLinkingService {
        async fn link_requirements_to_context(
            &self,
            _spec_id: &str,
        ) -> Result<Vec<ContextLink>, McpError> {
            Ok(vec![])
        }

        async fn link_requirement_to_context(
            &self,
            _requirement_id: &str,
        ) -> Result<Vec<ContextLink>, McpError> {
            Ok(vec![])
        }

        async fn update_task_progress(
            &self,
            _task_id: &str,
            _progress: f64,
        ) -> Result<
            crate::services::specification_context_linking_service::TaskProgressUpdate,
            McpError,
        > {
            Ok(
                crate::services::specification_context_linking_service::TaskProgressUpdate {
                    task_id: _task_id.to_string(),
                    old_progress: 0.0,
                    new_progress: _progress,
                    updated_at: chrono::Utc::now(),
                    context_updates: vec![],
                    related_tasks_affected: vec![],
                },
            )
        }

        async fn analyze_context_impact(
            &self,
            _context_id: &str,
        ) -> Result<ContextImpactAnalysis, McpError> {
            Ok(ContextImpactAnalysis {
                context_id: _context_id.to_string(),
                affected_specifications: vec![],
                affected_requirements: vec![],
                affected_tasks: vec![],
                risk_level: crate::services::specification_context_linking_service::RiskLevel::Low,
                recommendations: vec![],
                analyzed_at: chrono::Utc::now(),
            })
        }

        async fn sync_specification_context(&self, _spec_id: &str) -> Result<SyncResult, McpError> {
            Ok(SyncResult {
                spec_id: "test".to_string(),
                sync_type:
                    crate::services::specification_context_linking_service::SyncType::Bidirectional,
                changes_applied: vec![],
                conflicts_detected: vec![],
                sync_status:
                    crate::services::specification_context_linking_service::SyncStatus::Success,
                synced_at: chrono::Utc::now(),
            })
        }

        async fn suggest_context_links(
            &self,
            _requirement_id: &str,
        ) -> Result<Vec<ContextSuggestion>, McpError> {
            Ok(vec![])
        }

        async fn track_specification_changes(
            &self,
            _spec_id: &str,
            _changes: SpecificationChanges,
        ) -> Result<(), McpError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_link_requirements_to_context() {
        let service = Arc::new(MockSpecificationContextLinkingService);
        let tools = SpecificationContextLinkingTools::new(service);

        let request = CallToolRequest {
            method: rmcp::model::CallToolRequestMethod,
            params: rmcp::model::CallToolRequestParam {
                name: "link_requirements_to_context".into(),
                arguments: Some(
                    serde_json::json!({
                        "spec_id": "test-spec"
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            },
            extensions: rmcp::model::Extensions::default(),
        };

        let result = tools.handle_tool_call(request).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
        assert!(!result.content.is_empty());
    }
}
