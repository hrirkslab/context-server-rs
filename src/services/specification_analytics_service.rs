use crate::models::specification::{
    ProjectSpecification, Requirement, Task, RequirementStatus, TaskStatus, 
    Priority, Complexity, SpecStatus, SpecType
};
use crate::repositories::SpecificationRepository;
use crate::services::analytics_service::{AnalyticsService, AnalyticsEvent, AnalyticsEventType};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use rmcp::model::ErrorData as McpError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Progress tracking data for requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementProgress {
    pub requirement_id: String,
    pub title: String,
    pub status: RequirementStatus,
    pub priority: Priority,
    pub completion_percentage: f64,
    pub linked_tasks_count: usize,
    pub completed_tasks_count: usize,
    pub acceptance_criteria_count: usize,
    pub satisfied_criteria_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub days_since_creation: i64,
    pub days_since_last_update: i64,
}

/// Progress tracking data for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub task_id: String,
    pub title: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub complexity: Complexity,
    pub progress: f64,
    pub estimated_effort: Option<String>,
    pub actual_effort: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub days_in_progress: Option<i64>,
    pub is_blocked: bool,
    pub dependencies_count: usize,
    pub subtasks_count: usize,
    pub completed_subtasks_count: usize,
}

/// Specification completeness analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationCompleteness {
    pub spec_id: String,
    pub spec_type: SpecType,
    pub title: String,
    pub status: SpecStatus,
    pub completeness_score: f64,
    pub requirements_completeness: f64,
    pub tasks_completeness: f64,
    pub content_completeness: f64,
    pub missing_sections: Vec<String>,
    pub quality_issues: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Development velocity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentVelocity {
    pub project_id: String,
    pub time_period_days: i64,
    pub tasks_completed: usize,
    pub requirements_completed: usize,
    pub average_task_completion_time_days: f64,
    pub average_requirement_completion_time_days: f64,
    pub velocity_trend: VelocityTrend,
    pub bottlenecks: Vec<String>,
    pub productivity_score: f64,
}

/// Velocity trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VelocityTrend {
    Increasing,
    Stable,
    Decreasing,
    InsufficientData,
}

/// Specification health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationHealthReport {
    pub project_id: String,
    pub overall_health_score: f64,
    pub specifications: Vec<SpecificationCompleteness>,
    pub requirements_progress: Vec<RequirementProgress>,
    pub tasks_progress: Vec<TaskProgress>,
    pub velocity_metrics: DevelopmentVelocity,
    pub critical_issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

/// Service for specification analytics and progress tracking
#[async_trait]
pub trait SpecificationAnalyticsService: Send + Sync {
    /// Track progress for all requirements in a project
    async fn track_requirements_progress(&self, project_id: &str) -> Result<Vec<RequirementProgress>, McpError>;
    
    /// Track progress for all tasks in a project
    async fn track_tasks_progress(&self, project_id: &str) -> Result<Vec<TaskProgress>, McpError>;
    
    /// Analyze completeness of specifications
    async fn analyze_specification_completeness(&self, project_id: &str) -> Result<Vec<SpecificationCompleteness>, McpError>;
    
    /// Calculate development velocity metrics
    async fn calculate_development_velocity(&self, project_id: &str, days: i64) -> Result<DevelopmentVelocity, McpError>;
    
    /// Generate comprehensive health report
    async fn generate_health_report(&self, project_id: &str) -> Result<SpecificationHealthReport, McpError>;
    
    /// Track task status change event
    async fn track_task_status_change(&self, task_id: &str, old_status: TaskStatus, new_status: TaskStatus) -> Result<(), McpError>;
    
    /// Track requirement status change event
    async fn track_requirement_status_change(&self, requirement_id: &str, old_status: RequirementStatus, new_status: RequirementStatus) -> Result<(), McpError>;
}

/// Default implementation of specification analytics service
pub struct DefaultSpecificationAnalyticsService {
    specification_repository: Arc<dyn SpecificationRepository>,
    analytics_service: Arc<dyn AnalyticsService>,
}

impl DefaultSpecificationAnalyticsService {
    pub fn new(
        specification_repository: Arc<dyn SpecificationRepository>,
        analytics_service: Arc<dyn AnalyticsService>,
    ) -> Self {
        Self {
            specification_repository,
            analytics_service,
        }
    }

    /// Calculate completion percentage for a requirement based on acceptance criteria
    fn calculate_requirement_completion(&self, requirement: &Requirement) -> f64 {
        if requirement.acceptance_criteria.is_empty() {
            // If no acceptance criteria, base on status
            match requirement.status {
                RequirementStatus::Accepted => 1.0,
                RequirementStatus::Tested => 0.9,
                RequirementStatus::Implemented => 0.8,
                RequirementStatus::InProgress => 0.5,
                RequirementStatus::Defined => 0.2,
                RequirementStatus::Draft => 0.1,
                RequirementStatus::Rejected | RequirementStatus::Deferred => 0.0,
            }
        } else {
            let satisfied_count = requirement.acceptance_criteria.iter()
                .filter(|c| c.status == crate::models::specification::CriterionStatus::Satisfied)
                .count();
            satisfied_count as f64 / requirement.acceptance_criteria.len() as f64
        }
    }

    /// Calculate completeness score for a specification
    fn calculate_specification_completeness(&self, spec: &ProjectSpecification, requirements: &[Requirement], tasks: &[Task]) -> SpecificationCompleteness {
        let completeness_score;
        let mut missing_sections = Vec::new();
        let mut quality_issues = Vec::new();
        let mut recommendations = Vec::new();

        // Content completeness (40% of total score)
        let content_completeness = if spec.content.raw_content.len() < 100 {
            quality_issues.push("Specification content is too brief".to_string());
            0.2
        } else if spec.content.raw_content.len() < 500 {
            0.6
        } else {
            1.0
        };

        // Check for essential sections based on spec type
        match spec.spec_type {
            SpecType::Requirements => {
                if !spec.content.raw_content.to_lowercase().contains("acceptance criteria") {
                    missing_sections.push("Acceptance Criteria".to_string());
                }
                if !spec.content.raw_content.to_lowercase().contains("user story") {
                    missing_sections.push("User Stories".to_string());
                }
            }
            SpecType::Design => {
                if !spec.content.raw_content.to_lowercase().contains("architecture") {
                    missing_sections.push("Architecture".to_string());
                }
                if !spec.content.raw_content.to_lowercase().contains("component") {
                    missing_sections.push("Components".to_string());
                }
            }
            SpecType::Tasks => {
                if !spec.content.raw_content.contains("[ ]") && !spec.content.raw_content.contains("[x]") {
                    missing_sections.push("Task Checkboxes".to_string());
                }
            }
            _ => {}
        }

        // Requirements completeness (30% of total score)
        let requirements_completeness = if requirements.is_empty() {
            if spec.spec_type == SpecType::Requirements {
                quality_issues.push("No requirements defined".to_string());
                0.0
            } else {
                0.5 // Not all specs need requirements
            }
        } else {
            let completed_requirements = requirements.iter()
                .filter(|r| matches!(r.status, RequirementStatus::Accepted | RequirementStatus::Tested))
                .count();
            completed_requirements as f64 / requirements.len() as f64
        };

        // Tasks completeness (30% of total score)
        let tasks_completeness = if tasks.is_empty() {
            if spec.spec_type == SpecType::Tasks {
                quality_issues.push("No tasks defined".to_string());
                0.0
            } else {
                0.5 // Not all specs need tasks
            }
        } else {
            let completed_tasks = tasks.iter()
                .filter(|t| t.status == TaskStatus::Completed)
                .count();
            completed_tasks as f64 / tasks.len() as f64
        };

        completeness_score = (content_completeness * 0.4) + (requirements_completeness * 0.3) + (tasks_completeness * 0.3);

        // Generate recommendations
        if completeness_score < 0.5 {
            recommendations.push("Specification needs significant improvement".to_string());
        }
        if !missing_sections.is_empty() {
            recommendations.push(format!("Add missing sections: {}", missing_sections.join(", ")));
        }
        if requirements_completeness < 0.3 {
            recommendations.push("Focus on completing requirements".to_string());
        }
        if tasks_completeness < 0.3 {
            recommendations.push("Focus on completing tasks".to_string());
        }

        SpecificationCompleteness {
            spec_id: spec.id.clone(),
            spec_type: spec.spec_type.clone(),
            title: spec.title.clone(),
            status: spec.status.clone(),
            completeness_score,
            requirements_completeness,
            tasks_completeness,
            content_completeness,
            missing_sections,
            quality_issues,
            recommendations,
        }
    }

    /// Calculate velocity trend based on completion data
    fn calculate_velocity_trend(&self, recent_completions: usize, historical_completions: usize) -> VelocityTrend {
        if historical_completions == 0 {
            return VelocityTrend::InsufficientData;
        }

        let ratio = recent_completions as f64 / historical_completions as f64;
        
        if ratio > 1.2 {
            VelocityTrend::Increasing
        } else if ratio < 0.8 {
            VelocityTrend::Decreasing
        } else {
            VelocityTrend::Stable
        }
    }
}

#[async_trait]
impl SpecificationAnalyticsService for DefaultSpecificationAnalyticsService {
    async fn track_requirements_progress(&self, project_id: &str) -> Result<Vec<RequirementProgress>, McpError> {
        let specifications = self.specification_repository.find_specifications_by_project(project_id).await?;
        let mut all_progress = Vec::new();
        let now = Utc::now();

        for spec in specifications {
            let requirements = self.specification_repository.find_requirements_by_spec(&spec.id).await?;
            
            for requirement in requirements {
                let tasks = self.specification_repository.find_tasks_by_spec(&spec.id).await?;
                let linked_tasks: Vec<_> = tasks.iter()
                    .filter(|t| t.linked_requirements.contains(&requirement.id))
                    .collect();
                
                let completed_tasks_count = linked_tasks.iter()
                    .filter(|t| t.status == TaskStatus::Completed)
                    .count();

                let satisfied_criteria_count = requirement.acceptance_criteria.iter()
                    .filter(|c| c.status == crate::models::specification::CriterionStatus::Satisfied)
                    .count();

                let completion_percentage = self.calculate_requirement_completion(&requirement);

                let days_since_creation = (now - requirement.created_at).num_days();
                let days_since_last_update = (now - requirement.updated_at).num_days();

                all_progress.push(RequirementProgress {
                    requirement_id: requirement.id,
                    title: requirement.title,
                    status: requirement.status,
                    priority: requirement.priority,
                    completion_percentage,
                    linked_tasks_count: linked_tasks.len(),
                    completed_tasks_count,
                    acceptance_criteria_count: requirement.acceptance_criteria.len(),
                    satisfied_criteria_count,
                    created_at: requirement.created_at,
                    updated_at: requirement.updated_at,
                    days_since_creation,
                    days_since_last_update,
                });
            }
        }

        Ok(all_progress)
    }

    async fn track_tasks_progress(&self, project_id: &str) -> Result<Vec<TaskProgress>, McpError> {
        let specifications = self.specification_repository.find_specifications_by_project(project_id).await?;
        let mut all_progress = Vec::new();
        let now = Utc::now();

        for spec in specifications {
            let tasks = self.specification_repository.find_tasks_by_spec(&spec.id).await?;
            
            for task in tasks {
                let days_in_progress = if let Some(started_at) = task.started_at {
                    Some((now - started_at).num_days())
                } else {
                    None
                };

                let is_blocked = task.status == TaskStatus::Blocked;

                // Count subtasks
                let all_tasks = self.specification_repository.find_tasks_by_spec(&spec.id).await?;
                let subtasks: Vec<_> = all_tasks.iter()
                    .filter(|t| t.parent_task.as_ref() == Some(&task.id))
                    .collect();
                
                let completed_subtasks_count = subtasks.iter()
                    .filter(|t| t.status == TaskStatus::Completed)
                    .count();

                all_progress.push(TaskProgress {
                    task_id: task.id,
                    title: task.title,
                    status: task.status,
                    priority: task.metadata.priority,
                    complexity: task.metadata.complexity,
                    progress: task.progress,
                    estimated_effort: task.estimated_effort,
                    actual_effort: task.actual_effort,
                    created_at: task.created_at,
                    updated_at: task.updated_at,
                    started_at: task.started_at,
                    completed_at: task.completed_at,
                    days_in_progress,
                    is_blocked,
                    dependencies_count: task.dependencies.len(),
                    subtasks_count: subtasks.len(),
                    completed_subtasks_count,
                });
            }
        }

        Ok(all_progress)
    }

    async fn analyze_specification_completeness(&self, project_id: &str) -> Result<Vec<SpecificationCompleteness>, McpError> {
        let specifications = self.specification_repository.find_specifications_by_project(project_id).await?;
        let mut completeness_analysis = Vec::new();

        for spec in specifications {
            let requirements = self.specification_repository.find_requirements_by_spec(&spec.id).await?;
            let tasks = self.specification_repository.find_tasks_by_spec(&spec.id).await?;
            
            let completeness = self.calculate_specification_completeness(&spec, &requirements, &tasks);
            completeness_analysis.push(completeness);
        }

        Ok(completeness_analysis)
    }

    async fn calculate_development_velocity(&self, project_id: &str, days: i64) -> Result<DevelopmentVelocity, McpError> {
        let cutoff_date = Utc::now() - Duration::days(days);
        let specifications = self.specification_repository.find_specifications_by_project(project_id).await?;
        
        let mut tasks_completed = 0;
        let mut requirements_completed = 0;
        let mut task_completion_times = Vec::new();
        let mut requirement_completion_times = Vec::new();
        let mut bottlenecks = Vec::new();

        for spec in &specifications {
            let tasks = self.specification_repository.find_tasks_by_spec(&spec.id).await?;
            let requirements = self.specification_repository.find_requirements_by_spec(&spec.id).await?;

            // Count completed tasks in the time period
            for task in &tasks {
                if let Some(completed_at) = task.completed_at {
                    if completed_at >= cutoff_date {
                        tasks_completed += 1;
                        
                        if let Some(started_at) = task.started_at {
                            let completion_time = (completed_at - started_at).num_days();
                            task_completion_times.push(completion_time as f64);
                        }
                    }
                }
                
                // Identify bottlenecks
                if task.status == TaskStatus::Blocked {
                    bottlenecks.push(format!("Task '{}' is blocked", task.title));
                }
                if let Some(days_in_progress) = (Utc::now() - task.created_at).num_days().into() {
                    if days_in_progress > 30 && task.status == TaskStatus::InProgress {
                        bottlenecks.push(format!("Task '{}' has been in progress for {} days", task.title, days_in_progress));
                    }
                }
            }

            // Count completed requirements in the time period
            for requirement in &requirements {
                if matches!(requirement.status, RequirementStatus::Accepted | RequirementStatus::Tested) {
                    if requirement.updated_at >= cutoff_date {
                        requirements_completed += 1;
                        
                        let completion_time = (requirement.updated_at - requirement.created_at).num_days();
                        requirement_completion_times.push(completion_time as f64);
                    }
                }
            }
        }

        let average_task_completion_time_days = if task_completion_times.is_empty() {
            0.0
        } else {
            task_completion_times.iter().sum::<f64>() / task_completion_times.len() as f64
        };

        let average_requirement_completion_time_days = if requirement_completion_times.is_empty() {
            0.0
        } else {
            requirement_completion_times.iter().sum::<f64>() / requirement_completion_times.len() as f64
        };

        // Calculate velocity trend (simplified - compare with previous period)
        let previous_cutoff = cutoff_date - Duration::days(days);
        let mut previous_tasks_completed = 0;
        
        for spec in &specifications {
            let tasks = self.specification_repository.find_tasks_by_spec(&spec.id).await?;
            for task in &tasks {
                if let Some(completed_at) = task.completed_at {
                    if completed_at >= previous_cutoff && completed_at < cutoff_date {
                        previous_tasks_completed += 1;
                    }
                }
            }
        }

        let velocity_trend = self.calculate_velocity_trend(tasks_completed, previous_tasks_completed);

        // Calculate productivity score (0-100)
        let productivity_score = if days > 0 {
            let daily_task_rate = tasks_completed as f64 / days as f64;
            let daily_requirement_rate = requirements_completed as f64 / days as f64;
            ((daily_task_rate * 50.0) + (daily_requirement_rate * 30.0)).min(100.0)
        } else {
            0.0
        };

        Ok(DevelopmentVelocity {
            project_id: project_id.to_string(),
            time_period_days: days,
            tasks_completed,
            requirements_completed,
            average_task_completion_time_days,
            average_requirement_completion_time_days,
            velocity_trend,
            bottlenecks,
            productivity_score,
        })
    }

    async fn generate_health_report(&self, project_id: &str) -> Result<SpecificationHealthReport, McpError> {
        let specifications = self.analyze_specification_completeness(project_id).await?;
        let requirements_progress = self.track_requirements_progress(project_id).await?;
        let tasks_progress = self.track_tasks_progress(project_id).await?;
        let velocity_metrics = self.calculate_development_velocity(project_id, 30).await?;

        // Calculate overall health score
        let spec_health_avg = if specifications.is_empty() {
            0.0
        } else {
            specifications.iter().map(|s| s.completeness_score).sum::<f64>() / specifications.len() as f64
        };

        let req_completion_avg = if requirements_progress.is_empty() {
            0.0
        } else {
            requirements_progress.iter().map(|r| r.completion_percentage).sum::<f64>() / requirements_progress.len() as f64
        };

        let task_completion_avg = if tasks_progress.is_empty() {
            0.0
        } else {
            tasks_progress.iter().map(|t| t.progress).sum::<f64>() / tasks_progress.len() as f64
        };

        let overall_health_score = (spec_health_avg * 0.4) + (req_completion_avg * 0.3) + (task_completion_avg * 0.3);

        // Identify critical issues
        let mut critical_issues = Vec::new();
        
        if overall_health_score < 0.3 {
            critical_issues.push("Overall project health is critically low".to_string());
        }

        let blocked_tasks = tasks_progress.iter().filter(|t| t.is_blocked).count();
        if blocked_tasks > 0 {
            critical_issues.push(format!("{} tasks are currently blocked", blocked_tasks));
        }

        let stale_requirements = requirements_progress.iter()
            .filter(|r| r.days_since_last_update > 30)
            .count();
        if stale_requirements > 0 {
            critical_issues.push(format!("{} requirements haven't been updated in over 30 days", stale_requirements));
        }

        // Generate recommendations
        let mut recommendations = Vec::new();
        
        if overall_health_score < 0.5 {
            recommendations.push("Focus on improving specification completeness".to_string());
        }
        
        if velocity_metrics.productivity_score < 30.0 {
            recommendations.push("Consider reviewing development processes to improve velocity".to_string());
        }

        if !velocity_metrics.bottlenecks.is_empty() {
            recommendations.push("Address identified bottlenecks to improve flow".to_string());
        }

        let incomplete_specs = specifications.iter()
            .filter(|s| s.completeness_score < 0.7)
            .count();
        if incomplete_specs > 0 {
            recommendations.push(format!("Complete {} incomplete specifications", incomplete_specs));
        }

        Ok(SpecificationHealthReport {
            project_id: project_id.to_string(),
            overall_health_score,
            specifications,
            requirements_progress,
            tasks_progress,
            velocity_metrics,
            critical_issues,
            recommendations,
            generated_at: Utc::now(),
        })
    }

    async fn track_task_status_change(&self, task_id: &str, old_status: TaskStatus, new_status: TaskStatus) -> Result<(), McpError> {
        let event = AnalyticsEvent {
            id: Uuid::new_v4().to_string(),
            event_type: AnalyticsEventType::EntityUpdate,
            project_id: None, // Will be filled by the caller if available
            entity_type: Some("task".to_string()),
            entity_id: Some(task_id.to_string()),
            user_agent: None,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("old_status".to_string(), serde_json::Value::String(old_status.as_str().to_string()));
                metadata.insert("new_status".to_string(), serde_json::Value::String(new_status.as_str().to_string()));
                metadata.insert("change_type".to_string(), serde_json::Value::String("status_change".to_string()));
                metadata
            },
            timestamp: Utc::now(),
            duration_ms: None,
            success: true,
            error_message: None,
        };

        self.analytics_service.track_event(event).await
            .map_err(|e| McpError::internal_error(format!("Failed to track task status change: {}", e), None))?;

        Ok(())
    }

    async fn track_requirement_status_change(&self, requirement_id: &str, old_status: RequirementStatus, new_status: RequirementStatus) -> Result<(), McpError> {
        let event = AnalyticsEvent {
            id: Uuid::new_v4().to_string(),
            event_type: AnalyticsEventType::EntityUpdate,
            project_id: None, // Will be filled by the caller if available
            entity_type: Some("requirement".to_string()),
            entity_id: Some(requirement_id.to_string()),
            user_agent: None,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("old_status".to_string(), serde_json::Value::String(old_status.as_str().to_string()));
                metadata.insert("new_status".to_string(), serde_json::Value::String(new_status.as_str().to_string()));
                metadata.insert("change_type".to_string(), serde_json::Value::String("status_change".to_string()));
                metadata
            },
            timestamp: Utc::now(),
            duration_ms: None,
            success: true,
            error_message: None,
        };

        self.analytics_service.track_event(event).await
            .map_err(|e| McpError::internal_error(format!("Failed to track requirement status change: {}", e), None))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::specification::{SpecContent, SpecFormat, AcceptanceCriterion, CriterionType, CriterionStatus};
    use crate::repositories::SpecificationRepository;
    use crate::services::analytics_service::{AnalyticsService, UsageStatistics, ProjectInsights};
    use async_trait::async_trait;

    // Mock repositories for testing
    struct MockSpecificationRepository {
        specifications: Vec<ProjectSpecification>,
        requirements: Vec<Requirement>,
        tasks: Vec<Task>,
    }

    impl MockSpecificationRepository {
        fn new() -> Self {
            Self {
                specifications: Vec::new(),
                requirements: Vec::new(),
                tasks: Vec::new(),
            }
        }

        fn with_test_data() -> Self {
            let mut repo = Self::new();
            
            // Add test specification
            let spec = ProjectSpecification::new(
                "test-project".to_string(),
                SpecType::Requirements,
                "Test Specification".to_string(),
                SpecContent::new(SpecFormat::Markdown, "# Test Spec\n\nThis is a test specification with requirements and acceptance criteria.".to_string()),
            );
            repo.specifications.push(spec.clone());

            // Add test requirement
            let mut requirement = Requirement::new(
                spec.id.clone(),
                "Test Requirement".to_string(),
                "This is a test requirement".to_string(),
            );
            requirement.add_acceptance_criterion(AcceptanceCriterion::new(
                "WHEN user performs action THEN system SHALL respond".to_string(),
                CriterionType::Functional,
            ));
            repo.requirements.push(requirement);

            // Add test task
            let mut task = Task::new(
                spec.id.clone(),
                "Test Task".to_string(),
                "This is a test task".to_string(),
            );
            task.update_progress(0.5);
            repo.tasks.push(task);

            repo
        }
    }

    #[async_trait]
    impl SpecificationRepository for MockSpecificationRepository {
        async fn create_specification(&self, _spec: &ProjectSpecification) -> Result<ProjectSpecification, McpError> {
            unimplemented!()
        }

        async fn find_specification_by_id(&self, id: &str) -> Result<Option<ProjectSpecification>, McpError> {
            Ok(self.specifications.iter().find(|s| s.id == id).cloned())
        }

        async fn find_specifications_by_project(&self, project_id: &str) -> Result<Vec<ProjectSpecification>, McpError> {
            Ok(self.specifications.iter()
                .filter(|s| s.project_id == project_id)
                .cloned()
                .collect())
        }

        async fn find_specifications_by_type(&self, project_id: &str, spec_type: &str) -> Result<Vec<ProjectSpecification>, McpError> {
            Ok(self.specifications.iter()
                .filter(|s| s.project_id == project_id && s.spec_type.as_str() == spec_type)
                .cloned()
                .collect())
        }

        async fn update_specification(&self, _spec: &ProjectSpecification) -> Result<ProjectSpecification, McpError> {
            unimplemented!()
        }

        async fn delete_specification(&self, _id: &str) -> Result<bool, McpError> {
            unimplemented!()
        }

        async fn create_requirement(&self, _requirement: &Requirement) -> Result<Requirement, McpError> {
            unimplemented!()
        }

        async fn find_requirement_by_id(&self, id: &str) -> Result<Option<Requirement>, McpError> {
            Ok(self.requirements.iter().find(|r| r.id == id).cloned())
        }

        async fn find_requirements_by_spec(&self, spec_id: &str) -> Result<Vec<Requirement>, McpError> {
            Ok(self.requirements.iter()
                .filter(|r| r.spec_id == spec_id)
                .cloned()
                .collect())
        }

        async fn update_requirement(&self, _requirement: &Requirement) -> Result<Requirement, McpError> {
            unimplemented!()
        }

        async fn delete_requirement(&self, _id: &str) -> Result<bool, McpError> {
            unimplemented!()
        }

        async fn create_task(&self, _task: &Task) -> Result<Task, McpError> {
            unimplemented!()
        }

        async fn find_task_by_id(&self, id: &str) -> Result<Option<Task>, McpError> {
            Ok(self.tasks.iter().find(|t| t.id == id).cloned())
        }

        async fn find_tasks_by_spec(&self, spec_id: &str) -> Result<Vec<Task>, McpError> {
            Ok(self.tasks.iter()
                .filter(|t| t.spec_id == spec_id)
                .cloned()
                .collect())
        }

        async fn find_tasks_by_status(&self, spec_id: &str, status: &str) -> Result<Vec<Task>, McpError> {
            Ok(self.tasks.iter()
                .filter(|t| t.spec_id == spec_id && t.status.as_str() == status)
                .cloned()
                .collect())
        }

        async fn update_task(&self, _task: &Task) -> Result<Task, McpError> {
            unimplemented!()
        }

        async fn delete_task(&self, _id: &str) -> Result<bool, McpError> {
            unimplemented!()
        }

        async fn link_requirement_to_context(&self, _requirement_id: &str, _context_id: &str) -> Result<(), McpError> {
            unimplemented!()
        }

        async fn link_task_to_context(&self, _task_id: &str, _context_id: &str) -> Result<(), McpError> {
            unimplemented!()
        }

        async fn link_task_to_requirement(&self, _task_id: &str, _requirement_id: &str) -> Result<(), McpError> {
            unimplemented!()
        }

        async fn unlink_requirement_from_context(&self, _requirement_id: &str, _context_id: &str) -> Result<(), McpError> {
            unimplemented!()
        }

        async fn unlink_task_from_context(&self, _task_id: &str, _context_id: &str) -> Result<(), McpError> {
            unimplemented!()
        }

        async fn unlink_task_from_requirement(&self, _task_id: &str, _requirement_id: &str) -> Result<(), McpError> {
            unimplemented!()
        }
    }

    struct MockAnalyticsService;

    #[async_trait]
    impl AnalyticsService for MockAnalyticsService {
        async fn track_event(&self, _event: AnalyticsEvent) -> Result<()> {
            Ok(())
        }

        async fn get_entity_usage(&self, _entity_type: &str, _entity_id: &str) -> Result<UsageStatistics> {
            Ok(UsageStatistics {
                total_queries: 10,
                successful_queries: 9,
                failed_queries: 1,
                last_query: Some(Utc::now()),
                average_response_time_ms: 150.0,
                most_common_operations: vec!["query".to_string()],
            })
        }

        async fn get_project_insights(&self, _project_id: &str) -> Result<ProjectInsights> {
            Ok(ProjectInsights {
                project_id: "test-project".to_string(),
                total_events: 100,
                most_active_entity_types: vec![("task".to_string(), 50)],
                success_rate: 0.9,
                average_response_time_ms: 150.0,
                peak_usage_hours: vec![9, 10, 11],
                context_health_score: 85.0,
                recommendations: vec!["Keep up the good work".to_string()],
            })
        }

        async fn get_global_statistics(&self) -> Result<HashMap<String, serde_json::Value>> {
            Ok(HashMap::new())
        }

        async fn generate_usage_report(&self, _start_date: DateTime<Utc>, _end_date: DateTime<Utc>) -> Result<serde_json::Value> {
            Ok(serde_json::json!({}))
        }
    }

    #[tokio::test]
    async fn test_track_requirements_progress() {
        let spec_repo = Arc::new(MockSpecificationRepository::with_test_data());
        let analytics_service = Arc::new(MockAnalyticsService);
        let service = DefaultSpecificationAnalyticsService::new(spec_repo, analytics_service);

        let progress = service.track_requirements_progress("test-project").await.unwrap();
        
        assert_eq!(progress.len(), 1);
        assert_eq!(progress[0].title, "Test Requirement");
        assert_eq!(progress[0].acceptance_criteria_count, 1);
        assert!(progress[0].completion_percentage > 0.0);
    }

    #[tokio::test]
    async fn test_track_tasks_progress() {
        let spec_repo = Arc::new(MockSpecificationRepository::with_test_data());
        let analytics_service = Arc::new(MockAnalyticsService);
        let service = DefaultSpecificationAnalyticsService::new(spec_repo, analytics_service);

        let progress = service.track_tasks_progress("test-project").await.unwrap();
        
        assert_eq!(progress.len(), 1);
        assert_eq!(progress[0].title, "Test Task");
        assert_eq!(progress[0].progress, 0.5);
        assert!(!progress[0].is_blocked);
    }

    #[tokio::test]
    async fn test_analyze_specification_completeness() {
        let spec_repo = Arc::new(MockSpecificationRepository::with_test_data());
        let analytics_service = Arc::new(MockAnalyticsService);
        let service = DefaultSpecificationAnalyticsService::new(spec_repo, analytics_service);

        let completeness = service.analyze_specification_completeness("test-project").await.unwrap();
        
        assert_eq!(completeness.len(), 1);
        assert_eq!(completeness[0].title, "Test Specification");
        assert!(completeness[0].completeness_score > 0.0);
        assert!(completeness[0].content_completeness > 0.0);
    }

    #[tokio::test]
    async fn test_calculate_development_velocity() {
        let spec_repo = Arc::new(MockSpecificationRepository::with_test_data());
        let analytics_service = Arc::new(MockAnalyticsService);
        let service = DefaultSpecificationAnalyticsService::new(spec_repo, analytics_service);

        let velocity = service.calculate_development_velocity("test-project", 30).await.unwrap();
        
        assert_eq!(velocity.project_id, "test-project");
        assert_eq!(velocity.time_period_days, 30);
        assert!(velocity.productivity_score >= 0.0);
    }

    #[tokio::test]
    async fn test_generate_health_report() {
        let spec_repo = Arc::new(MockSpecificationRepository::with_test_data());
        let analytics_service = Arc::new(MockAnalyticsService);
        let service = DefaultSpecificationAnalyticsService::new(spec_repo, analytics_service);

        let report = service.generate_health_report("test-project").await.unwrap();
        
        assert_eq!(report.project_id, "test-project");
        assert!(report.overall_health_score >= 0.0);
        assert_eq!(report.specifications.len(), 1);
        assert_eq!(report.requirements_progress.len(), 1);
        assert_eq!(report.tasks_progress.len(), 1);
        assert!(!report.recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_track_task_status_change() {
        let spec_repo = Arc::new(MockSpecificationRepository::with_test_data());
        let analytics_service = Arc::new(MockAnalyticsService);
        let service = DefaultSpecificationAnalyticsService::new(spec_repo, analytics_service);

        let result = service.track_task_status_change(
            "task-1",
            TaskStatus::NotStarted,
            TaskStatus::InProgress,
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_track_requirement_status_change() {
        let spec_repo = Arc::new(MockSpecificationRepository::with_test_data());
        let analytics_service = Arc::new(MockAnalyticsService);
        let service = DefaultSpecificationAnalyticsService::new(spec_repo, analytics_service);

        let result = service.track_requirement_status_change(
            "req-1",
            RequirementStatus::Draft,
            RequirementStatus::Defined,
        ).await;
        
        assert!(result.is_ok());
    }
}