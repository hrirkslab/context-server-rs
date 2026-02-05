use crate::services::specification_analytics_service::SpecificationAnalyticsService;
use rmcp::model::{ErrorData as McpError, Tool, CallToolResult, Content};
use serde_json::{json, Value};
use std::sync::Arc;

/// MCP tools for specification analytics
pub struct SpecificationAnalyticsTools {
    analytics_service: Arc<dyn SpecificationAnalyticsService>,
}

impl SpecificationAnalyticsTools {
    pub fn new(analytics_service: Arc<dyn SpecificationAnalyticsService>) -> Self {
        Self { analytics_service }
    }

    /// Get available specification analytics tools
    pub fn get_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "track_requirements_progress".into(),
                description: Some("Track progress for all requirements in a project, including completion percentages, linked tasks, and acceptance criteria status".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project_id": {
                            "type": "string",
                            "description": "The ID of the project to track requirements progress for"
                        }
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
                        "project_id": {
                            "type": "string",
                            "description": "The ID of the project to track tasks progress for"
                        }
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
                        "project_id": {
                            "type": "string",
                            "description": "The ID of the project to analyze specification completeness for"
                        }
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
                        "project_id": {
                            "type": "string",
                            "description": "The ID of the project to calculate velocity for"
                        },
                        "days": {
                            "type": "integer",
                            "description": "Number of days to look back for velocity calculation",
                            "default": 30,
                            "minimum": 1,
                            "maximum": 365
                        }
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
                        "project_id": {
                            "type": "string",
                            "description": "The ID of the project to generate health report for"
                        }
                    },
                    "required": ["project_id"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
        ]
    }

    /// Handle specification analytics tool calls
    pub async fn handle_tool_call(&self, name: &str, arguments: Value) -> Result<CallToolResult, McpError> {
        match name {
            "track_requirements_progress" => self.handle_track_requirements_progress(arguments).await,
            "track_tasks_progress" => self.handle_track_tasks_progress(arguments).await,
            "analyze_specification_completeness" => self.handle_analyze_specification_completeness(arguments).await,
            "calculate_development_velocity" => self.handle_calculate_development_velocity(arguments).await,
            "generate_specification_health_report" => self.handle_generate_specification_health_report(arguments).await,
            _ => Err(McpError::method_not_found::<rmcp::model::CallToolRequestMethod>()),
        }
    }

    async fn handle_track_requirements_progress(&self, arguments: Value) -> Result<CallToolResult, McpError> {
        let project_id = arguments.get("project_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid project_id", None))?;

        let progress = self.analytics_service.track_requirements_progress(project_id).await?;

        let result = json!({
            "project_id": project_id,
            "requirements_count": progress.len(),
            "requirements_progress": progress,
            "summary": {
                "total_requirements": progress.len(),
                "completed_requirements": progress.iter().filter(|r| r.completion_percentage >= 1.0).count(),
                "in_progress_requirements": progress.iter().filter(|r| r.completion_percentage > 0.0 && r.completion_percentage < 1.0).count(),
                "not_started_requirements": progress.iter().filter(|r| r.completion_percentage == 0.0).count(),
                "average_completion": if progress.is_empty() { 0.0 } else { 
                    progress.iter().map(|r| r.completion_percentage).sum::<f64>() / progress.len() as f64 
                },
                "stale_requirements": progress.iter().filter(|r| r.days_since_last_update > 30).count(),
                "high_priority_requirements": progress.iter().filter(|r| matches!(r.priority, crate::models::specification::Priority::Critical | crate::models::specification::Priority::High)).count(),
            }
        });

        Ok(CallToolResult::success(vec![Content::text(format!("Requirements Progress Tracking for Project: {}\n\n{}", project_id, serde_json::to_string_pretty(&result).unwrap()))]))
    }

    async fn handle_track_tasks_progress(&self, arguments: Value) -> Result<CallToolResult, McpError> {
        let project_id = arguments.get("project_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid project_id", None))?;

        let progress = self.analytics_service.track_tasks_progress(project_id).await?;

        let result = json!({
            "project_id": project_id,
            "tasks_count": progress.len(),
            "tasks_progress": progress,
            "summary": {
                "total_tasks": progress.len(),
                "completed_tasks": progress.iter().filter(|t| t.status == crate::models::specification::TaskStatus::Completed).count(),
                "in_progress_tasks": progress.iter().filter(|t| t.status == crate::models::specification::TaskStatus::InProgress).count(),
                "not_started_tasks": progress.iter().filter(|t| t.status == crate::models::specification::TaskStatus::NotStarted).count(),
                "blocked_tasks": progress.iter().filter(|t| t.is_blocked).count(),
                "average_progress": if progress.is_empty() { 0.0 } else { 
                    progress.iter().map(|t| t.progress).sum::<f64>() / progress.len() as f64 
                },
                "tasks_with_dependencies": progress.iter().filter(|t| t.dependencies_count > 0).count(),
                "tasks_with_subtasks": progress.iter().filter(|t| t.subtasks_count > 0).count(),
                "high_priority_tasks": progress.iter().filter(|t| matches!(t.priority, crate::models::specification::Priority::Critical | crate::models::specification::Priority::High)).count(),
                "complex_tasks": progress.iter().filter(|t| matches!(t.complexity, crate::models::specification::Complexity::Complex | crate::models::specification::Complexity::VeryComplex)).count(),
            }
        });

        Ok(CallToolResult::success(vec![Content::text(format!("Tasks Progress Tracking for Project: {}\n\n{}", project_id, serde_json::to_string_pretty(&result).unwrap()))]))
    }

    async fn handle_analyze_specification_completeness(&self, arguments: Value) -> Result<CallToolResult, McpError> {
        let project_id = arguments.get("project_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid project_id", None))?;

        let completeness = self.analytics_service.analyze_specification_completeness(project_id).await?;

        let result = json!({
            "project_id": project_id,
            "specifications_count": completeness.len(),
            "specifications_completeness": completeness,
            "summary": {
                "total_specifications": completeness.len(),
                "complete_specifications": completeness.iter().filter(|s| s.completeness_score >= 0.8).count(),
                "incomplete_specifications": completeness.iter().filter(|s| s.completeness_score < 0.5).count(),
                "average_completeness": if completeness.is_empty() { 0.0 } else { 
                    completeness.iter().map(|s| s.completeness_score).sum::<f64>() / completeness.len() as f64 
                },
                "specifications_with_issues": completeness.iter().filter(|s| !s.quality_issues.is_empty()).count(),
                "specifications_with_missing_sections": completeness.iter().filter(|s| !s.missing_sections.is_empty()).count(),
                "total_recommendations": completeness.iter().map(|s| s.recommendations.len()).sum::<usize>(),
            }
        });

        Ok(CallToolResult::success(vec![Content::text(format!("Specification Completeness Analysis for Project: {}\n\n{}", project_id, serde_json::to_string_pretty(&result).unwrap()))]))
    }

    async fn handle_calculate_development_velocity(&self, arguments: Value) -> Result<CallToolResult, McpError> {
        let project_id = arguments.get("project_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid project_id", None))?;

        let days = arguments.get("days")
            .and_then(|v| v.as_i64())
            .unwrap_or(30);

        if days < 1 || days > 365 {
            return Err(McpError::invalid_params("Days must be between 1 and 365", None));
        }

        let velocity = self.analytics_service.calculate_development_velocity(project_id, days).await?;

        let result = json!({
            "project_id": project_id,
            "velocity_metrics": velocity,
            "insights": {
                "daily_task_completion_rate": velocity.tasks_completed as f64 / velocity.time_period_days as f64,
                "daily_requirement_completion_rate": velocity.requirements_completed as f64 / velocity.time_period_days as f64,
                "velocity_trend_description": match velocity.velocity_trend {
                    crate::services::specification_analytics_service::VelocityTrend::Increasing => "Development velocity is increasing",
                    crate::services::specification_analytics_service::VelocityTrend::Stable => "Development velocity is stable",
                    crate::services::specification_analytics_service::VelocityTrend::Decreasing => "Development velocity is decreasing",
                    crate::services::specification_analytics_service::VelocityTrend::InsufficientData => "Insufficient data to determine trend",
                },
                "productivity_level": if velocity.productivity_score >= 70.0 {
                    "High"
                } else if velocity.productivity_score >= 40.0 {
                    "Medium"
                } else {
                    "Low"
                },
                "has_bottlenecks": !velocity.bottlenecks.is_empty(),
                "bottlenecks_count": velocity.bottlenecks.len(),
            }
        });

        Ok(CallToolResult::success(vec![Content::text(format!("Development Velocity Metrics for Project: {} (Last {} days)\n\n{}", project_id, days, serde_json::to_string_pretty(&result).unwrap()))]))
    }

    async fn handle_generate_specification_health_report(&self, arguments: Value) -> Result<CallToolResult, McpError> {
        let project_id = arguments.get("project_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid project_id", None))?;

        let report = self.analytics_service.generate_health_report(project_id).await?;

        let result = json!({
            "project_id": project_id,
            "health_report": report,
            "executive_summary": {
                "overall_health_level": if report.overall_health_score >= 0.8 {
                    "Excellent"
                } else if report.overall_health_score >= 0.6 {
                    "Good"
                } else if report.overall_health_score >= 0.4 {
                    "Fair"
                } else {
                    "Poor"
                },
                "total_specifications": report.specifications.len(),
                "total_requirements": report.requirements_progress.len(),
                "total_tasks": report.tasks_progress.len(),
                "critical_issues_count": report.critical_issues.len(),
                "recommendations_count": report.recommendations.len(),
                "has_critical_issues": !report.critical_issues.is_empty(),
                "velocity_trend": report.velocity_metrics.velocity_trend,
                "productivity_score": report.velocity_metrics.productivity_score,
            }
        });

        Ok(CallToolResult::success(vec![Content::text(format!("Specification Health Report for Project: {}\n\n{}", project_id, serde_json::to_string_pretty(&result).unwrap()))]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::specification_analytics_service::{
        SpecificationAnalyticsService, RequirementProgress, TaskProgress, 
        SpecificationCompleteness, DevelopmentVelocity, SpecificationHealthReport,
        VelocityTrend
    };
    use crate::models::specification::{RequirementStatus, TaskStatus, Priority, Complexity, SpecType, SpecStatus};
    use async_trait::async_trait;
    use chrono::Utc;

    struct MockSpecificationAnalyticsService;

    #[async_trait]
    impl SpecificationAnalyticsService for MockSpecificationAnalyticsService {
        async fn track_requirements_progress(&self, _project_id: &str) -> Result<Vec<RequirementProgress>, McpError> {
            Ok(vec![RequirementProgress {
                requirement_id: "req-1".to_string(),
                title: "Test Requirement".to_string(),
                status: RequirementStatus::InProgress,
                priority: Priority::High,
                completion_percentage: 0.7,
                linked_tasks_count: 3,
                completed_tasks_count: 2,
                acceptance_criteria_count: 5,
                satisfied_criteria_count: 3,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                days_since_creation: 10,
                days_since_last_update: 2,
            }])
        }

        async fn track_tasks_progress(&self, _project_id: &str) -> Result<Vec<TaskProgress>, McpError> {
            Ok(vec![TaskProgress {
                task_id: "task-1".to_string(),
                title: "Test Task".to_string(),
                status: TaskStatus::InProgress,
                priority: Priority::Medium,
                complexity: Complexity::Medium,
                progress: 0.6,
                estimated_effort: Some("2 days".to_string()),
                actual_effort: Some("1.5 days".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                started_at: Some(Utc::now()),
                completed_at: None,
                days_in_progress: Some(3),
                is_blocked: false,
                dependencies_count: 1,
                subtasks_count: 2,
                completed_subtasks_count: 1,
            }])
        }

        async fn analyze_specification_completeness(&self, _project_id: &str) -> Result<Vec<SpecificationCompleteness>, McpError> {
            Ok(vec![SpecificationCompleteness {
                spec_id: "spec-1".to_string(),
                spec_type: SpecType::Requirements,
                title: "Test Specification".to_string(),
                status: SpecStatus::InProgress,
                completeness_score: 0.75,
                requirements_completeness: 0.8,
                tasks_completeness: 0.7,
                content_completeness: 0.9,
                missing_sections: vec!["User Stories".to_string()],
                quality_issues: vec!["Some requirements lack acceptance criteria".to_string()],
                recommendations: vec!["Add user stories to requirements".to_string()],
            }])
        }

        async fn calculate_development_velocity(&self, _project_id: &str, days: i64) -> Result<DevelopmentVelocity, McpError> {
            Ok(DevelopmentVelocity {
                project_id: "test-project".to_string(),
                time_period_days: days,
                tasks_completed: 15,
                requirements_completed: 8,
                average_task_completion_time_days: 3.5,
                average_requirement_completion_time_days: 7.2,
                velocity_trend: VelocityTrend::Increasing,
                bottlenecks: vec!["Task dependencies causing delays".to_string()],
                productivity_score: 72.5,
            })
        }

        async fn generate_health_report(&self, project_id: &str) -> Result<SpecificationHealthReport, McpError> {
            let requirements_progress = self.track_requirements_progress(project_id).await?;
            let tasks_progress = self.track_tasks_progress(project_id).await?;
            let specifications = self.analyze_specification_completeness(project_id).await?;
            let velocity_metrics = self.calculate_development_velocity(project_id, 30).await?;

            Ok(SpecificationHealthReport {
                project_id: project_id.to_string(),
                overall_health_score: 0.75,
                specifications,
                requirements_progress,
                tasks_progress,
                velocity_metrics,
                critical_issues: vec!["Some tasks are overdue".to_string()],
                recommendations: vec!["Focus on completing blocked tasks".to_string()],
                generated_at: Utc::now(),
            })
        }

        async fn track_task_status_change(&self, _task_id: &str, _old_status: TaskStatus, _new_status: TaskStatus) -> Result<(), McpError> {
            Ok(())
        }

        async fn track_requirement_status_change(&self, _requirement_id: &str, _old_status: RequirementStatus, _new_status: RequirementStatus) -> Result<(), McpError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_track_requirements_progress_tool() {
        let service = Arc::new(MockSpecificationAnalyticsService);
        let tools = SpecificationAnalyticsTools::new(service);

        let arguments = json!({
            "project_id": "test-project"
        });

        let result = tools.handle_track_requirements_progress(arguments).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_track_tasks_progress_tool() {
        let service = Arc::new(MockSpecificationAnalyticsService);
        let tools = SpecificationAnalyticsTools::new(service);

        let arguments = json!({
            "project_id": "test-project"
        });

        let result = tools.handle_track_tasks_progress(arguments).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_specification_completeness_tool() {
        let service = Arc::new(MockSpecificationAnalyticsService);
        let tools = SpecificationAnalyticsTools::new(service);

        let arguments = json!({
            "project_id": "test-project"
        });

        let result = tools.handle_analyze_specification_completeness(arguments).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_calculate_development_velocity_tool() {
        let service = Arc::new(MockSpecificationAnalyticsService);
        let tools = SpecificationAnalyticsTools::new(service);

        let arguments = json!({
            "project_id": "test-project",
            "days": 30
        });

        let result = tools.handle_calculate_development_velocity(arguments).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_generate_specification_health_report_tool() {
        let service = Arc::new(MockSpecificationAnalyticsService);
        let tools = SpecificationAnalyticsTools::new(service);

        let arguments = json!({
            "project_id": "test-project"
        });

        let result = tools.handle_generate_specification_health_report(arguments).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_project_id() {
        let service = Arc::new(MockSpecificationAnalyticsService);
        let tools = SpecificationAnalyticsTools::new(service);

        let arguments = json!({});

        let result = tools.handle_track_requirements_progress(arguments).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_days_parameter() {
        let service = Arc::new(MockSpecificationAnalyticsService);
        let tools = SpecificationAnalyticsTools::new(service);

        let arguments = json!({
            "project_id": "test-project",
            "days": 500  // Invalid: too high
        });

        let result = tools.handle_calculate_development_velocity(arguments).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_get_tools() {
        let tools = SpecificationAnalyticsTools::get_tools();
        assert_eq!(tools.len(), 5);
        
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(tool_names.contains(&"track_requirements_progress"));
        assert!(tool_names.contains(&"track_tasks_progress"));
        assert!(tool_names.contains(&"analyze_specification_completeness"));
        assert!(tool_names.contains(&"calculate_development_velocity"));
        assert!(tool_names.contains(&"generate_specification_health_report"));
    }
}