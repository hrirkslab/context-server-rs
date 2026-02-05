use crate::models::enhanced_context::{EnhancedContextItem, ContextType, RelationshipType, ContextRelationship};
use crate::models::specification::{ProjectSpecification, Requirement, Task, RequirementId, TaskId, ContextId};
use crate::repositories::{SpecificationRepository, EnhancedContextRepository};
use crate::services::context_query_service::{ContextQueryService, ContextQueryResult};
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Service for linking specifications with context items
#[async_trait]
pub trait SpecificationContextLinkingService: Send + Sync {
    /// Automatically link requirements to relevant context items
    async fn link_requirements_to_context(&self, spec_id: &str) -> Result<Vec<ContextLink>, McpError>;
    
    /// Link a specific requirement to context items based on content analysis
    async fn link_requirement_to_context(&self, requirement_id: &str) -> Result<Vec<ContextLink>, McpError>;
    
    /// Update task progress and sync with related context
    async fn update_task_progress(&self, task_id: &str, progress: f64) -> Result<TaskProgressUpdate, McpError>;
    
    /// Analyze impact of context changes on specifications
    async fn analyze_context_impact(&self, context_id: &str) -> Result<ContextImpactAnalysis, McpError>;
    
    /// Perform bidirectional synchronization between specs and context
    async fn sync_specification_context(&self, spec_id: &str) -> Result<SyncResult, McpError>;
    
    /// Find context items that should be linked to a requirement based on semantic similarity
    async fn suggest_context_links(&self, requirement_id: &str) -> Result<Vec<ContextSuggestion>, McpError>;
    
    /// Track changes in specifications and update related context
    async fn track_specification_changes(&self, spec_id: &str, changes: SpecificationChanges) -> Result<(), McpError>;
}

/// Result of linking context to specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLink {
    pub requirement_id: Option<RequirementId>,
    pub task_id: Option<TaskId>,
    pub context_id: ContextId,
    pub link_type: LinkType,
    pub confidence: f64,
    pub auto_detected: bool,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of links between specifications and context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LinkType {
    Implements,
    Validates,
    References,
    Constrains,
    Enables,
    Conflicts,
    Depends,
    Similar,
}

impl LinkType {
    pub fn as_str(&self) -> &str {
        match self {
            LinkType::Implements => "implements",
            LinkType::Validates => "validates",
            LinkType::References => "references",
            LinkType::Constrains => "constrains",
            LinkType::Enables => "enables",
            LinkType::Conflicts => "conflicts",
            LinkType::Depends => "depends",
            LinkType::Similar => "similar",
        }
    }
    
    pub fn to_relationship_type(&self) -> RelationshipType {
        match self {
            LinkType::Implements => RelationshipType::Implements,
            LinkType::Validates => RelationshipType::Validates,
            LinkType::References => RelationshipType::References,
            LinkType::Constrains => RelationshipType::Constrains,
            LinkType::Enables => RelationshipType::Enables,
            LinkType::Conflicts => RelationshipType::Conflicts,
            LinkType::Depends => RelationshipType::DependsOn,
            LinkType::Similar => RelationshipType::Similar,
        }
    }
}

/// Task progress update with context synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressUpdate {
    pub task_id: TaskId,
    pub old_progress: f64,
    pub new_progress: f64,
    pub updated_at: DateTime<Utc>,
    pub context_updates: Vec<ContextUpdate>,
    pub related_tasks_affected: Vec<TaskId>,
}

/// Context update triggered by task progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUpdate {
    pub context_id: ContextId,
    pub update_type: ContextUpdateType,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of context updates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextUpdateType {
    StatusChange,
    ProgressUpdate,
    ValidationUpdate,
    RelationshipChange,
    MetadataUpdate,
}

/// Analysis of how context changes impact specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextImpactAnalysis {
    pub context_id: ContextId,
    pub affected_specifications: Vec<SpecificationImpact>,
    pub affected_requirements: Vec<RequirementImpact>,
    pub affected_tasks: Vec<TaskImpact>,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
    pub analyzed_at: DateTime<Utc>,
}

/// Impact on a specific specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationImpact {
    pub spec_id: String,
    pub impact_type: ImpactType,
    pub severity: ImpactSeverity,
    pub description: String,
    pub affected_sections: Vec<String>,
}

/// Impact on a specific requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementImpact {
    pub requirement_id: RequirementId,
    pub impact_type: ImpactType,
    pub severity: ImpactSeverity,
    pub description: String,
    pub validation_affected: bool,
}

/// Impact on a specific task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskImpact {
    pub task_id: TaskId,
    pub impact_type: ImpactType,
    pub severity: ImpactSeverity,
    pub description: String,
    pub progress_affected: bool,
    pub dependencies_affected: Vec<TaskId>,
}

/// Types of impact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactType {
    Validation,
    Implementation,
    Testing,
    Documentation,
    Architecture,
    Performance,
    Security,
    Compatibility,
}

/// Severity of impact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

/// Risk level for context changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
    Minimal,
}

/// Result of bidirectional synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub spec_id: String,
    pub sync_type: SyncType,
    pub changes_applied: Vec<SyncChange>,
    pub conflicts_detected: Vec<SyncConflict>,
    pub sync_status: SyncStatus,
    pub synced_at: DateTime<Utc>,
}

/// Types of synchronization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncType {
    SpecToContext,
    ContextToSpec,
    Bidirectional,
}

/// Individual synchronization change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncChange {
    pub change_type: ChangeType,
    pub target_id: String,
    pub description: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
}

/// Types of synchronization changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    LinkAdded,
    LinkRemoved,
    LinkUpdated,
    StatusUpdated,
    ProgressUpdated,
    MetadataUpdated,
    RelationshipAdded,
    RelationshipRemoved,
}

/// Synchronization conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub conflict_type: ConflictType,
    pub description: String,
    pub spec_value: serde_json::Value,
    pub context_value: serde_json::Value,
    pub resolution_strategy: ResolutionStrategy,
}

/// Types of synchronization conflicts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    StatusMismatch,
    ProgressMismatch,
    MetadataMismatch,
    RelationshipMismatch,
    ValidationMismatch,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResolutionStrategy {
    PreferSpec,
    PreferContext,
    Manual,
    Merge,
    Skip,
}

/// Status of synchronization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Success,
    PartialSuccess,
    Failed,
    ConflictsDetected,
}

/// Suggestion for linking context to specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSuggestion {
    pub context_id: ContextId,
    pub context_type: ContextType,
    pub title: String,
    pub description: String,
    pub relevance_score: f64,
    pub suggested_link_type: LinkType,
    pub reasoning: String,
    pub keywords_matched: Vec<String>,
}

/// Changes in specifications that need to be tracked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationChanges {
    pub spec_id: String,
    pub change_type: SpecChangeType,
    pub changes: Vec<SpecChange>,
    pub changed_at: DateTime<Utc>,
    pub changed_by: Option<String>,
}

/// Types of specification changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpecChangeType {
    RequirementAdded,
    RequirementModified,
    RequirementRemoved,
    TaskAdded,
    TaskModified,
    TaskRemoved,
    StatusChanged,
    ContentUpdated,
    MetadataChanged,
}

/// Individual specification change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecChange {
    pub field: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub description: String,
}

/// Default implementation of SpecificationContextLinkingService
pub struct DefaultSpecificationContextLinkingService {
    specification_repository: Arc<dyn SpecificationRepository>,
    enhanced_context_repository: Arc<dyn EnhancedContextRepository>,
    context_query_service: Arc<dyn ContextQueryService>,
}

impl DefaultSpecificationContextLinkingService {
    pub fn new(
        specification_repository: Arc<dyn SpecificationRepository>,
        enhanced_context_repository: Arc<dyn EnhancedContextRepository>,
        context_query_service: Arc<dyn ContextQueryService>,
    ) -> Self {
        Self {
            specification_repository,
            enhanced_context_repository,
            context_query_service,
        }
    }
    
    /// Analyze requirement text to extract keywords for context matching
    fn extract_keywords_from_requirement(&self, requirement: &Requirement) -> Vec<String> {
        let mut keywords = Vec::new();
        
        // Extract from title
        keywords.extend(self.extract_keywords_from_text(&requirement.title));
        
        // Extract from description
        keywords.extend(self.extract_keywords_from_text(&requirement.description));
        
        // Extract from user story if available
        if let Some(user_story) = &requirement.user_story {
            keywords.extend(self.extract_keywords_from_text(user_story));
        }
        
        // Extract from acceptance criteria
        for criterion in &requirement.acceptance_criteria {
            keywords.extend(self.extract_keywords_from_text(&criterion.description));
        }
        
        // Remove duplicates and return
        keywords.sort();
        keywords.dedup();
        keywords
    }
    
    /// Simple keyword extraction from text
    fn extract_keywords_from_text(&self, text: &str) -> Vec<String> {
        // Simple implementation - in production, you might use NLP libraries
        let stop_words = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "up", "about", "into", "through", "during", "before", "after", "above",
            "below", "between", "among", "is", "are", "was", "were", "be", "been", "being", "have",
            "has", "had", "do", "does", "did", "will", "would", "could", "should", "may", "might",
            "must", "can", "shall", "when", "then", "if", "that", "this", "these", "those",
        ];
        
        text.to_lowercase()
            .split_whitespace()
            .filter(|word| {
                word.len() > 2 && !stop_words.contains(word) && word.chars().all(|c| c.is_alphabetic())
            })
            .map(|word| word.to_string())
            .collect()
    }
    
    /// Calculate relevance score between requirement and context
    fn calculate_relevance_score(&self, requirement_keywords: &[String], context_item: &EnhancedContextItem) -> f64 {
        let context_text = format!("{} {}", context_item.content.title, context_item.content.description);
        let context_keywords = self.extract_keywords_from_text(&context_text);
        
        if context_keywords.is_empty() || requirement_keywords.is_empty() {
            return 0.0;
        }
        
        let matches = requirement_keywords.iter()
            .filter(|keyword| context_keywords.contains(keyword))
            .count();
        
        let total_unique_keywords = requirement_keywords.len() + context_keywords.len() - matches;
        
        if total_unique_keywords == 0 {
            0.0
        } else {
            (matches as f64 * 2.0) / total_unique_keywords as f64
        }
    }
    
    /// Determine the most appropriate link type based on context type and content
    fn determine_link_type(&self, context_type: &ContextType, requirement: &Requirement) -> LinkType {
        match context_type {
            ContextType::BusinessRule => {
                if requirement.title.to_lowercase().contains("implement") {
                    LinkType::Implements
                } else if requirement.title.to_lowercase().contains("validate") {
                    LinkType::Validates
                } else {
                    LinkType::Constrains
                }
            },
            ContextType::ArchitecturalDecision => {
                if requirement.description.to_lowercase().contains("architecture") {
                    LinkType::Implements
                } else {
                    LinkType::Constrains
                }
            },
            ContextType::PerformanceRequirement => {
                if requirement.title.to_lowercase().contains("performance") {
                    LinkType::Validates
                } else {
                    LinkType::Constrains
                }
            },
            ContextType::SecurityPolicy => {
                if requirement.title.to_lowercase().contains("security") {
                    LinkType::Validates
                } else {
                    LinkType::Constrains
                }
            },
            ContextType::FeatureContext => LinkType::References,
            ContextType::CodePattern => LinkType::Implements,
            ContextType::ApiSpecification => LinkType::Implements,
            ContextType::TestCase => LinkType::Validates,
            ContextType::Documentation => LinkType::References,
            _ => LinkType::Similar,
        }
    }
}

#[async_trait]
impl SpecificationContextLinkingService for DefaultSpecificationContextLinkingService {
    async fn link_requirements_to_context(&self, spec_id: &str) -> Result<Vec<ContextLink>, McpError> {
        let requirements = self.specification_repository.find_requirements_by_spec(spec_id).await?;
        let mut all_links = Vec::new();
        
        for requirement in requirements {
            let links = self.link_requirement_to_context(&requirement.id).await?;
            all_links.extend(links);
        }
        
        Ok(all_links)
    }
    
    async fn link_requirement_to_context(&self, requirement_id: &str) -> Result<Vec<ContextLink>, McpError> {
        let requirement = self.specification_repository.find_requirement_by_id(requirement_id).await?
            .ok_or_else(|| McpError::resource_not_found(format!("Requirement not found: {}", requirement_id), None))?;
        
        // Get the specification to find the project
        let spec = self.specification_repository.find_specification_by_id(&requirement.spec_id).await?
            .ok_or_else(|| McpError::resource_not_found(format!("Specification not found: {}", requirement.spec_id), None))?;
        
        // Extract keywords from requirement
        let keywords = self.extract_keywords_from_requirement(&requirement);
        
        // Query context for the project
        let context_result = self.context_query_service.query_context(
            &spec.project_id,
            &requirement.title, // Use title as feature area
            "implementation", // Default task type
            &keywords,
        ).await?;
        
        let mut links = Vec::new();
        let now = Utc::now();
        
        // Create links for business rules
        for business_rule in context_result.business_rules {
            let context_item = EnhancedContextItem {
                id: business_rule.id.clone(),
                project_id: business_rule.project_id.clone(),
                content: crate::models::enhanced_context::ContextContent {
                    content_type: ContextType::BusinessRule,
                    title: business_rule.rule_name.clone(),
                    description: business_rule.description.clone().unwrap_or_default(),
                    data: serde_json::to_value(&business_rule).unwrap_or(serde_json::Value::Null),
                    source_file: None,
                    source_line: None,
                },
                ..Default::default()
            };
            
            let relevance = self.calculate_relevance_score(&keywords, &context_item);
            if relevance > 0.3 { // Threshold for relevance
                let link_type = self.determine_link_type(&ContextType::BusinessRule, &requirement);
                
                links.push(ContextLink {
                    requirement_id: Some(requirement.id.clone()),
                    task_id: None,
                    context_id: business_rule.id,
                    link_type,
                    confidence: relevance,
                    auto_detected: true,
                    created_at: now,
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Create links for architectural decisions
        for arch_decision in context_result.architectural_decisions {
            let context_item = EnhancedContextItem {
                id: arch_decision.id.clone(),
                project_id: arch_decision.project_id.clone(),
                content: crate::models::enhanced_context::ContextContent {
                    content_type: ContextType::ArchitecturalDecision,
                    title: arch_decision.decision_title.clone(),
                    description: arch_decision.context.clone().unwrap_or_default(),
                    data: serde_json::to_value(&arch_decision).unwrap_or(serde_json::Value::Null),
                    source_file: None,
                    source_line: None,
                },
                ..Default::default()
            };
            
            let relevance = self.calculate_relevance_score(&keywords, &context_item);
            if relevance > 0.3 {
                let link_type = self.determine_link_type(&ContextType::ArchitecturalDecision, &requirement);
                
                links.push(ContextLink {
                    requirement_id: Some(requirement.id.clone()),
                    task_id: None,
                    context_id: arch_decision.id,
                    link_type,
                    confidence: relevance,
                    auto_detected: true,
                    created_at: now,
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Create links for performance requirements
        for perf_req in context_result.performance_requirements {
            let context_item = EnhancedContextItem {
                id: perf_req.id.clone(),
                project_id: perf_req.project_id.clone(),
                content: crate::models::enhanced_context::ContextContent {
                    content_type: ContextType::PerformanceRequirement,
                    title: perf_req.component_area.clone().unwrap_or("Performance Requirement".to_string()),
                    description: perf_req.requirement_type.clone().unwrap_or_default(),
                    data: serde_json::to_value(&perf_req).unwrap_or(serde_json::Value::Null),
                    source_file: None,
                    source_line: None,
                },
                ..Default::default()
            };
            
            let relevance = self.calculate_relevance_score(&keywords, &context_item);
            if relevance > 0.3 {
                let link_type = self.determine_link_type(&ContextType::PerformanceRequirement, &requirement);
                
                links.push(ContextLink {
                    requirement_id: Some(requirement.id.clone()),
                    task_id: None,
                    context_id: perf_req.id,
                    link_type,
                    confidence: relevance,
                    auto_detected: true,
                    created_at: now,
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Store the links in the repository
        for link in &links {
            if let Some(req_id) = &link.requirement_id {
                self.specification_repository.link_requirement_to_context(req_id, &link.context_id).await?;
            }
        }
        
        Ok(links)
    }
    
    async fn update_task_progress(&self, task_id: &str, progress: f64) -> Result<TaskProgressUpdate, McpError> {
        let mut task = self.specification_repository.find_task_by_id(task_id).await?
            .ok_or_else(|| McpError::resource_not_found(format!("Task not found: {}", task_id), None))?;
        
        let old_progress = task.progress;
        task.update_progress(progress);
        
        // Update the task in the repository
        self.specification_repository.update_task(&task).await?;
        
        let now = Utc::now();
        let mut context_updates = Vec::new();
        
        // Update related context items
        for context_id in &task.linked_context {
            let mut metadata = HashMap::new();
            metadata.insert("task_id".to_string(), serde_json::Value::String(task_id.to_string()));
            metadata.insert("old_progress".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(old_progress).unwrap()));
            metadata.insert("new_progress".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(progress).unwrap()));
            
            context_updates.push(ContextUpdate {
                context_id: context_id.clone(),
                update_type: ContextUpdateType::ProgressUpdate,
                description: format!("Task progress updated from {:.1}% to {:.1}%", old_progress * 100.0, progress * 100.0),
                metadata,
            });
        }
        
        // Find related tasks that might be affected
        let related_tasks = self.specification_repository.find_tasks_by_spec(&task.spec_id).await?;
        let related_tasks_affected: Vec<TaskId> = related_tasks.iter()
            .filter(|t| t.dependencies.contains(&task.id) || task.dependencies.contains(&t.id))
            .map(|t| t.id.clone())
            .collect();
        
        Ok(TaskProgressUpdate {
            task_id: task_id.to_string(),
            old_progress,
            new_progress: progress,
            updated_at: now,
            context_updates,
            related_tasks_affected,
        })
    }
    
    async fn analyze_context_impact(&self, context_id: &str) -> Result<ContextImpactAnalysis, McpError> {
        // This is a simplified implementation - in production, you'd want more sophisticated analysis
        let now = Utc::now();
        
        // Find all specifications that reference this context
        // For now, we'll return a basic analysis structure
        Ok(ContextImpactAnalysis {
            context_id: context_id.to_string(),
            affected_specifications: Vec::new(),
            affected_requirements: Vec::new(),
            affected_tasks: Vec::new(),
            risk_level: RiskLevel::Low,
            recommendations: vec![
                "Review linked requirements for consistency".to_string(),
                "Update related tasks if necessary".to_string(),
            ],
            analyzed_at: now,
        })
    }
    
    async fn sync_specification_context(&self, spec_id: &str) -> Result<SyncResult, McpError> {
        let now = Utc::now();
        
        // This is a simplified implementation
        Ok(SyncResult {
            spec_id: spec_id.to_string(),
            sync_type: SyncType::Bidirectional,
            changes_applied: Vec::new(),
            conflicts_detected: Vec::new(),
            sync_status: SyncStatus::Success,
            synced_at: now,
        })
    }
    
    async fn suggest_context_links(&self, requirement_id: &str) -> Result<Vec<ContextSuggestion>, McpError> {
        let requirement = self.specification_repository.find_requirement_by_id(requirement_id).await?
            .ok_or_else(|| McpError::resource_not_found(format!("Requirement not found: {}", requirement_id), None))?;
        
        // Get the specification to find the project
        let spec = self.specification_repository.find_specification_by_id(&requirement.spec_id).await?
            .ok_or_else(|| McpError::resource_not_found(format!("Specification not found: {}", requirement.spec_id), None))?;
        
        let keywords = self.extract_keywords_from_requirement(&requirement);
        
        // Query enhanced context repository for relevant context items
        let context_items = self.enhanced_context_repository.find_contexts_by_keywords(&spec.project_id, &keywords).await?;
        
        let mut suggestions = Vec::new();
        
        for context_item in context_items {
            let relevance_score = self.calculate_relevance_score(&keywords, &context_item);
            
            if relevance_score > 0.3 { // Threshold for relevance
                let suggested_link_type = self.determine_link_type(&context_item.content.content_type, &requirement);
                
                // Find matched keywords
                let context_text = format!("{} {}", context_item.content.title, context_item.content.description);
                let context_keywords = self.extract_keywords_from_text(&context_text);
                let keywords_matched: Vec<String> = keywords.iter()
                    .filter(|keyword| context_keywords.contains(keyword))
                    .cloned()
                    .collect();
                
                suggestions.push(ContextSuggestion {
                    context_id: context_item.id,
                    context_type: context_item.content.content_type,
                    title: context_item.content.title,
                    description: context_item.content.description,
                    relevance_score,
                    suggested_link_type,
                    reasoning: format!("Matched {} keywords and context type is relevant", keywords_matched.len()),
                    keywords_matched,
                });
            }
        }
        
        // Sort by relevance score (highest first)
        suggestions.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(suggestions)
    }
    
    async fn track_specification_changes(&self, spec_id: &str, changes: SpecificationChanges) -> Result<(), McpError> {
        // This would typically log changes and trigger appropriate updates
        tracing::info!("Tracking specification changes for spec {}: {:?}", spec_id, changes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::specification::{SpecContent, SpecFormat, SpecType, AcceptanceCriterion, CriterionType};
    use crate::repositories::{SpecificationRepository, EnhancedContextRepository};
    use crate::services::context_query_service::{ContextQueryService, ContextQueryResult};
    use async_trait::async_trait;
    
    // Mock implementations for testing
    struct MockSpecificationRepository;
    struct MockEnhancedContextRepository;
    
    #[async_trait]
    impl SpecificationRepository for MockSpecificationRepository {
        async fn create_specification(&self, spec: &ProjectSpecification) -> Result<ProjectSpecification, McpError> {
            Ok(spec.clone())
        }
        
        async fn find_specification_by_id(&self, _id: &str) -> Result<Option<ProjectSpecification>, McpError> {
            Ok(Some(ProjectSpecification::new(
                "test-project".to_string(),
                SpecType::Requirements,
                "Test Spec".to_string(),
                SpecContent::new(SpecFormat::Markdown, "# Test".to_string()),
            )))
        }
        
        async fn find_specifications_by_project(&self, _project_id: &str) -> Result<Vec<ProjectSpecification>, McpError> {
            Ok(Vec::new())
        }
        
        async fn find_specifications_by_type(&self, _project_id: &str, _spec_type: &str) -> Result<Vec<ProjectSpecification>, McpError> {
            Ok(Vec::new())
        }
        
        async fn update_specification(&self, spec: &ProjectSpecification) -> Result<ProjectSpecification, McpError> {
            Ok(spec.clone())
        }
        
        async fn delete_specification(&self, _id: &str) -> Result<bool, McpError> {
            Ok(true)
        }
        
        async fn create_requirement(&self, requirement: &Requirement) -> Result<Requirement, McpError> {
            Ok(requirement.clone())
        }
        
        async fn find_requirement_by_id(&self, _id: &str) -> Result<Option<Requirement>, McpError> {
            let mut requirement = Requirement::new(
                "test-spec".to_string(),
                "Test Requirement".to_string(),
                "A test requirement for authentication".to_string(),
            );
            requirement.add_acceptance_criterion(AcceptanceCriterion::new(
                "WHEN user enters valid credentials THEN system SHALL authenticate user".to_string(),
                CriterionType::Functional,
            ));
            Ok(Some(requirement))
        }
        
        async fn find_requirements_by_spec(&self, _spec_id: &str) -> Result<Vec<Requirement>, McpError> {
            Ok(Vec::new())
        }
        
        async fn update_requirement(&self, requirement: &Requirement) -> Result<Requirement, McpError> {
            Ok(requirement.clone())
        }
        
        async fn delete_requirement(&self, _id: &str) -> Result<bool, McpError> {
            Ok(true)
        }
        
        async fn create_task(&self, task: &Task) -> Result<Task, McpError> {
            Ok(task.clone())
        }
        
        async fn find_task_by_id(&self, _id: &str) -> Result<Option<Task>, McpError> {
            Ok(Some(Task::new(
                "test-spec".to_string(),
                "Test Task".to_string(),
                "A test task".to_string(),
            )))
        }
        
        async fn find_tasks_by_spec(&self, _spec_id: &str) -> Result<Vec<Task>, McpError> {
            Ok(Vec::new())
        }
        
        async fn find_tasks_by_status(&self, _spec_id: &str, _status: &str) -> Result<Vec<Task>, McpError> {
            Ok(Vec::new())
        }
        
        async fn update_task(&self, task: &Task) -> Result<Task, McpError> {
            Ok(task.clone())
        }
        
        async fn delete_task(&self, _id: &str) -> Result<bool, McpError> {
            Ok(true)
        }
        
        async fn link_requirement_to_context(&self, _requirement_id: &str, _context_id: &str) -> Result<(), McpError> {
            Ok(())
        }
        
        async fn link_task_to_context(&self, _task_id: &str, _context_id: &str) -> Result<(), McpError> {
            Ok(())
        }
        
        async fn link_task_to_requirement(&self, _task_id: &str, _requirement_id: &str) -> Result<(), McpError> {
            Ok(())
        }
        
        async fn unlink_requirement_from_context(&self, _requirement_id: &str, _context_id: &str) -> Result<(), McpError> {
            Ok(())
        }
        
        async fn unlink_task_from_context(&self, _task_id: &str, _context_id: &str) -> Result<(), McpError> {
            Ok(())
        }
        
        async fn unlink_task_from_requirement(&self, _task_id: &str, _requirement_id: &str) -> Result<(), McpError> {
            Ok(())
        }
    }
    
    #[async_trait]
    impl EnhancedContextRepository for MockEnhancedContextRepository {
        async fn create_context(&self, context: &EnhancedContextItem) -> Result<EnhancedContextItem, McpError> {
            Ok(context.clone())
        }
        
        async fn find_context_by_id(&self, _id: &str) -> Result<Option<EnhancedContextItem>, McpError> {
            Ok(None)
        }
        
        async fn find_contexts_by_project(&self, _project_id: &str) -> Result<Vec<EnhancedContextItem>, McpError> {
            Ok(Vec::new())
        }
        
        async fn find_contexts_by_type(&self, _project_id: &str, _context_type: ContextType) -> Result<Vec<EnhancedContextItem>, McpError> {
            Ok(Vec::new())
        }
        
        async fn find_contexts_by_keywords(&self, _project_id: &str, keywords: &[String]) -> Result<Vec<EnhancedContextItem>, McpError> {
            use crate::models::enhanced_context::*;
            
            // Return mock context items that match authentication keywords
            if keywords.iter().any(|k| k.contains("authentication") || k.contains("user") || k.contains("login")) {
                Ok(vec![
                    EnhancedContextItem {
                        id: "context-1".to_string(),
                        project_id: "test-project".to_string(),
                        content: ContextContent {
                            content_type: ContextType::BusinessRule,
                            title: "User Authentication Rule".to_string(),
                            description: "Users must authenticate with valid credentials".to_string(),
                            data: serde_json::Value::Null,
                            source_file: None,
                            source_line: None,
                        },
                        ..Default::default()
                    }
                ])
            } else {
                Ok(Vec::new())
            }
        }
        
        async fn update_context(&self, context: &EnhancedContextItem) -> Result<EnhancedContextItem, McpError> {
            Ok(context.clone())
        }
        
        async fn delete_context(&self, _id: &str) -> Result<bool, McpError> {
            Ok(true)
        }
        
        async fn find_contexts_linked_to_requirement(&self, _requirement_id: &str) -> Result<Vec<EnhancedContextItem>, McpError> {
            Ok(Vec::new())
        }
        
        async fn find_contexts_linked_to_task(&self, _task_id: &str) -> Result<Vec<EnhancedContextItem>, McpError> {
            Ok(Vec::new())
        }
        
        async fn find_related_contexts(&self, _context_id: &str) -> Result<Vec<EnhancedContextItem>, McpError> {
            Ok(Vec::new())
        }
        
        async fn update_quality_score(&self, _context_id: &str, _score: f64) -> Result<(), McpError> {
            Ok(())
        }
        
        async fn record_context_usage(&self, _context_id: &str) -> Result<(), McpError> {
            Ok(())
        }
    }
    
    struct MockContextQueryService;
    
    #[async_trait]
    impl ContextQueryService for MockContextQueryService {
        async fn query_context(
            &self,
            _project_id: &str,
            _feature_area: &str,
            _task_type: &str,
            _components: &[String],
        ) -> Result<ContextQueryResult, McpError> {
            use crate::models::context::{BusinessRule, ArchitecturalDecision, PerformanceRequirement};
            
            Ok(ContextQueryResult {
                business_rules: vec![
                    BusinessRule {
                        id: "br-1".to_string(),
                        project_id: "test-project".to_string(),
                        rule_name: "Authentication Rule".to_string(),
                        description: Some("Users must authenticate with valid credentials".to_string()),
                        domain_area: Some("authentication".to_string()),
                        implementation_pattern: None,
                        constraints: None,
                        examples: None,
                        created_at: None,
                    }
                ],
                architectural_decisions: vec![
                    ArchitecturalDecision {
                        id: "ad-1".to_string(),
                        project_id: "test-project".to_string(),
                        decision_title: "Authentication Architecture".to_string(),
                        context: Some("Need to implement user authentication".to_string()),
                        decision: Some("Use JWT tokens".to_string()),
                        consequences: None,
                        alternatives_considered: None,
                        status: None,
                        created_at: None,
                    }
                ],
                performance_requirements: vec![
                    PerformanceRequirement {
                        id: "pr-1".to_string(),
                        project_id: "test-project".to_string(),
                        component_area: Some("authentication".to_string()),
                        requirement_type: Some("response_time".to_string()),
                        target_value: Some("< 200ms".to_string()),
                        optimization_patterns: None,
                        avoid_patterns: None,
                        created_at: None,
                    }
                ],
                security_policies: Vec::new(),
                project_conventions: Vec::new(),
            })
        }
    }
    
    #[tokio::test]
    async fn test_link_requirement_to_context() {
        let spec_repo = Arc::new(MockSpecificationRepository);
        let enhanced_context_repo = Arc::new(MockEnhancedContextRepository);
        let context_service = Arc::new(MockContextQueryService);
        let linking_service = DefaultSpecificationContextLinkingService::new(spec_repo, enhanced_context_repo, context_service);
        
        let result = linking_service.link_requirement_to_context("test-req").await;
        assert!(result.is_ok());
        
        let links = result.unwrap();
        assert!(!links.is_empty());
        
        // Should have found links to business rule and architectural decision
        // Performance requirement might be filtered out due to low relevance score
        assert!(links.iter().any(|link| link.context_id == "br-1"));
        assert!(links.iter().any(|link| link.context_id == "ad-1"));
        
        // Check that we have at least some links
        assert!(links.len() >= 2);
        
        // Verify link types are appropriate
        let br_link = links.iter().find(|link| link.context_id == "br-1").unwrap();
        assert!(matches!(br_link.link_type, LinkType::Implements | LinkType::Validates | LinkType::Constrains));
    }
    
    #[tokio::test]
    async fn test_update_task_progress() {
        let spec_repo = Arc::new(MockSpecificationRepository);
        let enhanced_context_repo = Arc::new(MockEnhancedContextRepository);
        let context_service = Arc::new(MockContextQueryService);
        let linking_service = DefaultSpecificationContextLinkingService::new(spec_repo, enhanced_context_repo, context_service);
        
        let result = linking_service.update_task_progress("test-task", 0.75).await;
        assert!(result.is_ok());
        
        let update = result.unwrap();
        assert_eq!(update.task_id, "test-task");
        assert_eq!(update.new_progress, 0.75);
    }
    
    #[tokio::test]
    async fn test_extract_keywords_from_requirement() {
        let spec_repo = Arc::new(MockSpecificationRepository);
        let enhanced_context_repo = Arc::new(MockEnhancedContextRepository);
        let context_service = Arc::new(MockContextQueryService);
        let linking_service = DefaultSpecificationContextLinkingService::new(spec_repo, enhanced_context_repo, context_service);
        
        let requirement = Requirement::new(
            "test-spec".to_string(),
            "User Authentication System".to_string(),
            "Implement secure user authentication with JWT tokens".to_string(),
        );
        
        let keywords = linking_service.extract_keywords_from_requirement(&requirement);
        
        assert!(keywords.contains(&"user".to_string()));
        assert!(keywords.contains(&"authentication".to_string()));
        assert!(keywords.contains(&"system".to_string()));
        assert!(keywords.contains(&"implement".to_string()));
        assert!(keywords.contains(&"secure".to_string()));
        assert!(keywords.contains(&"jwt".to_string()));
        assert!(keywords.contains(&"tokens".to_string()));
    }
    
    #[tokio::test]
    async fn test_suggest_context_links() {
        let spec_repo = Arc::new(MockSpecificationRepository);
        let enhanced_context_repo = Arc::new(MockEnhancedContextRepository);
        let context_service = Arc::new(MockContextQueryService);
        let linking_service = DefaultSpecificationContextLinkingService::new(spec_repo, enhanced_context_repo, context_service);
        
        let result = linking_service.suggest_context_links("test-req").await;
        assert!(result.is_ok());
        
        let suggestions = result.unwrap();
        assert!(!suggestions.is_empty());
        
        // Should find the mock authentication context
        let auth_suggestion = suggestions.iter().find(|s| s.context_id == "context-1");
        assert!(auth_suggestion.is_some());
        
        let suggestion = auth_suggestion.unwrap();
        assert_eq!(suggestion.context_type, ContextType::BusinessRule);
        assert!(suggestion.relevance_score > 0.0);
        assert!(!suggestion.keywords_matched.is_empty());
    }
}