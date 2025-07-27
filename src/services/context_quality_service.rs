use crate::models::enhanced_context::*;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};
use anyhow::Result;

/// Service for evaluating and improving context quality
#[async_trait]
pub trait ContextQualityService: Send + Sync {
    /// Calculate quality metrics for a context item
    async fn calculate_quality_metrics(&self, context: &EnhancedContextItem, related_contexts: &[EnhancedContextItem]) -> Result<QualityMetrics>;
    
    /// Validate context against quality rules
    async fn validate_context(&self, context: &EnhancedContextItem, validation_rules: &[ValidationRule]) -> Result<ValidationResult>;
    
    /// Generate quality improvement suggestions
    async fn generate_improvement_suggestions(&self, context: &EnhancedContextItem, quality_metrics: &QualityMetrics) -> Result<Vec<QualityImprovement>>;
    
    /// Batch quality assessment for multiple contexts
    async fn assess_context_batch(&self, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextQualityAssessment>>;
    
    /// Get quality rules for a specific context type
    fn get_validation_rules(&self, context_type: &ContextType) -> Vec<ValidationRule>;
}

/// Default implementation of the Context Quality Service
pub struct DefaultContextQualityService {
    quality_analyzers: Vec<Box<dyn QualityAnalyzer>>,
    validation_rules: HashMap<ContextType, Vec<ValidationRule>>,
    improvement_generators: Vec<Box<dyn ImprovementGenerator>>,
}

impl DefaultContextQualityService {
    pub fn new() -> Self {
        let mut service = Self {
            quality_analyzers: vec![
                Box::new(CompletenessAnalyzer::new()),
                Box::new(ConsistencyAnalyzer::new()),
                Box::new(AccuracyAnalyzer::new()),
                Box::new(RelevanceAnalyzer::new()),
                Box::new(FreshnessAnalyzer::new()),
            ],
            validation_rules: HashMap::new(),
            improvement_generators: vec![
                Box::new(CompletenessImprover::new()),
                Box::new(ConsistencyImprover::new()),
                Box::new(ContentImprover::new()),
            ],
        };
        
        service.initialize_validation_rules();
        service
    }

    fn initialize_validation_rules(&mut self) {
        // Business Rule validation rules
        self.validation_rules.insert(
            ContextType::BusinessRule,
            vec![
                ValidationRule::new(
                    "title_required",
                    "Business rule must have a descriptive title",
                    RuleType::Required,
                    |context| !context.content.title.trim().is_empty(),
                ),
                ValidationRule::new(
                    "description_length",
                    "Business rule description should be at least 20 characters",
                    RuleType::Quality,
                    |context| context.content.description.len() >= 20,
                ),
                ValidationRule::new(
                    "domain_area_specified",
                    "Business rule should specify domain area",
                    RuleType::Completeness,
                    |context| {
                        context.content.data.get("domain_area")
                            .and_then(|v| v.as_str())
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false)
                    },
                ),
            ],
        );

        // Architectural Decision validation rules
        self.validation_rules.insert(
            ContextType::ArchitecturalDecision,
            vec![
                ValidationRule::new(
                    "decision_documented",
                    "Architectural decision must document the actual decision",
                    RuleType::Required,
                    |context| {
                        context.content.data.get("decision")
                            .and_then(|v| v.as_str())
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false)
                    },
                ),
                ValidationRule::new(
                    "consequences_documented",
                    "Architectural decision should document consequences",
                    RuleType::Quality,
                    |context| {
                        context.content.data.get("consequences")
                            .and_then(|v| v.as_str())
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false)
                    },
                ),
                ValidationRule::new(
                    "alternatives_considered",
                    "Architectural decision should document alternatives considered",
                    RuleType::Quality,
                    |context| {
                        context.content.data.get("alternatives_considered")
                            .and_then(|v| v.as_str())
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false)
                    },
                ),
            ],
        );

        // Security Policy validation rules
        self.validation_rules.insert(
            ContextType::SecurityPolicy,
            vec![
                ValidationRule::new(
                    "policy_area_specified",
                    "Security policy must specify the policy area",
                    RuleType::Required,
                    |context| {
                        context.content.data.get("policy_area")
                            .and_then(|v| v.as_str())
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false)
                    },
                ),
                ValidationRule::new(
                    "implementation_pattern_provided",
                    "Security policy should provide implementation patterns",
                    RuleType::Quality,
                    |context| {
                        context.content.data.get("implementation_pattern")
                            .and_then(|v| v.as_str())
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false)
                    },
                ),
            ],
        );

        // Performance Requirement validation rules
        self.validation_rules.insert(
            ContextType::PerformanceRequirement,
            vec![
                ValidationRule::new(
                    "target_value_specified",
                    "Performance requirement must specify target value",
                    RuleType::Required,
                    |context| {
                        context.content.data.get("target_value")
                            .and_then(|v| v.as_str())
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false)
                    },
                ),
                ValidationRule::new(
                    "component_area_specified",
                    "Performance requirement should specify component area",
                    RuleType::Quality,
                    |context| {
                        context.content.data.get("component_area")
                            .and_then(|v| v.as_str())
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false)
                    },
                ),
            ],
        );

        // Feature Context validation rules
        self.validation_rules.insert(
            ContextType::FeatureContext,
            vec![
                ValidationRule::new(
                    "business_purpose_documented",
                    "Feature context should document business purpose",
                    RuleType::Quality,
                    |context| {
                        context.content.data.get("business_purpose")
                            .and_then(|v| v.as_str())
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false)
                    },
                ),
                ValidationRule::new(
                    "key_workflows_documented",
                    "Feature context should document key workflows",
                    RuleType::Quality,
                    |context| {
                        context.content.data.get("key_workflows")
                            .and_then(|v| v.as_str())
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false)
                    },
                ),
            ],
        );
    }
}

#[async_trait]
impl ContextQualityService for DefaultContextQualityService {
    async fn calculate_quality_metrics(&self, context: &EnhancedContextItem, related_contexts: &[EnhancedContextItem]) -> Result<QualityMetrics> {
        let mut metrics = QualityMetrics::new();
        
        // Run all quality analyzers
        for analyzer in &self.quality_analyzers {
            analyzer.analyze(context, related_contexts, &mut metrics).await?;
        }
        
        // Calculate overall score
        metrics.calculate_overall_score();
        
        Ok(metrics)
    }

    async fn validate_context(&self, context: &EnhancedContextItem, validation_rules: &[ValidationRule]) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        
        for rule in validation_rules {
            let is_valid = (rule.validator)(context);
            
            if !is_valid {
                let issue = QualityIssue {
                    issue_type: match rule.rule_type {
                        RuleType::Required => QualityIssueType::MissingInformation,
                        RuleType::Quality => QualityIssueType::InconsistentData,
                        RuleType::Completeness => QualityIssueType::MissingInformation,
                        RuleType::Consistency => QualityIssueType::InconsistentData,
                    },
                    severity: match rule.rule_type {
                        RuleType::Required => IssueSeverity::Critical,
                        RuleType::Quality => IssueSeverity::Medium,
                        RuleType::Completeness => IssueSeverity::High,
                        RuleType::Consistency => IssueSeverity::High,
                    },
                    description: rule.description.clone(),
                    field: Some(rule.name.clone()),
                    detected_at: Utc::now(),
                };
                
                result.issues.push(issue);
            }
        }
        
        result.is_valid = result.issues.is_empty() || result.issues.iter().all(|i| i.severity != IssueSeverity::Critical);
        result.validation_score = if result.issues.is_empty() {
            1.0
        } else {
            let penalty = result.issues.iter().map(|i| match i.severity {
                IssueSeverity::Critical => 0.5,
                IssueSeverity::High => 0.3,
                IssueSeverity::Medium => 0.2,
                IssueSeverity::Low => 0.1,
                IssueSeverity::Info => 0.05,
            }).sum::<f64>();
            (1.0 - penalty).max(0.0)
        };
        
        Ok(result)
    }

    async fn generate_improvement_suggestions(&self, context: &EnhancedContextItem, quality_metrics: &QualityMetrics) -> Result<Vec<QualityImprovement>> {
        let mut suggestions = Vec::new();
        
        for generator in &self.improvement_generators {
            let generated = generator.generate_suggestions(context, quality_metrics).await?;
            suggestions.extend(generated);
        }
        
        // Sort by priority and estimated impact
        suggestions.sort_by(|a, b| {
            b.priority.score().partial_cmp(&a.priority.score())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.estimated_impact.partial_cmp(&a.estimated_impact).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        Ok(suggestions)
    }

    async fn assess_context_batch(&self, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextQualityAssessment>> {
        let mut assessments = Vec::new();
        
        for context in contexts {
            let validation_rules = self.get_validation_rules(&context.content.content_type);
            let quality_metrics = self.calculate_quality_metrics(context, contexts).await?;
            let validation_result = self.validate_context(context, &validation_rules).await?;
            let improvement_suggestions = self.generate_improvement_suggestions(context, &quality_metrics).await?;
            
            assessments.push(ContextQualityAssessment {
                context_id: context.id.clone(),
                quality_metrics,
                validation_result,
                improvement_suggestions,
                assessed_at: Utc::now(),
            });
        }
        
        Ok(assessments)
    }

    fn get_validation_rules(&self, context_type: &ContextType) -> Vec<ValidationRule> {
        self.validation_rules.get(context_type).cloned().unwrap_or_default()
    }
}

/// Validation rule for context quality
#[derive(Clone)]
pub struct ValidationRule {
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub validator: fn(&EnhancedContextItem) -> bool,
}

impl ValidationRule {
    pub fn new(name: &str, description: &str, rule_type: RuleType, validator: fn(&EnhancedContextItem) -> bool) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            rule_type,
            validator,
        }
    }
}

/// Types of validation rules
#[derive(Debug, Clone, PartialEq)]
pub enum RuleType {
    Required,
    Quality,
    Completeness,
    Consistency,
}

/// Result of context validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub validation_score: f64,
    pub issues: Vec<QualityIssue>,
    pub validated_at: DateTime<Utc>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            validation_score: 1.0,
            issues: Vec::new(),
            validated_at: Utc::now(),
        }
    }
}

/// Complete quality assessment for a context item
#[derive(Debug, Clone)]
pub struct ContextQualityAssessment {
    pub context_id: ContextId,
    pub quality_metrics: QualityMetrics,
    pub validation_result: ValidationResult,
    pub improvement_suggestions: Vec<QualityImprovement>,
    pub assessed_at: DateTime<Utc>,
}

/// Trait for analyzing specific quality aspects
#[async_trait]
trait QualityAnalyzer: Send + Sync {
    async fn analyze(&self, context: &EnhancedContextItem, related_contexts: &[EnhancedContextItem], metrics: &mut QualityMetrics) -> Result<()>;
}

/// Analyzer for content completeness
struct CompletenessAnalyzer;

impl CompletenessAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl QualityAnalyzer for CompletenessAnalyzer {
    async fn analyze(&self, context: &EnhancedContextItem, _related_contexts: &[EnhancedContextItem], metrics: &mut QualityMetrics) -> Result<()> {
        let mut score = 0.0;
        let mut total_checks = 0;
        
        // Check title completeness
        total_checks += 1;
        if !context.content.title.trim().is_empty() && context.content.title.len() > 5 {
            score += 1.0;
        }
        
        // Check description completeness
        total_checks += 1;
        if !context.content.description.trim().is_empty() && context.content.description.len() > 20 {
            score += 1.0;
        }
        
        // Check metadata completeness
        total_checks += 1;
        if !context.metadata.tags.is_empty() {
            score += 1.0;
        }
        
        // Check semantic tags
        total_checks += 1;
        if !context.semantic_tags.is_empty() {
            score += 1.0;
        }
        
        // Check type-specific completeness
        total_checks += 1;
        let type_specific_score = match context.content.content_type {
            ContextType::BusinessRule => {
                let has_domain = context.content.data.get("domain_area").is_some();
                let has_pattern = context.content.data.get("implementation_pattern").is_some();
                if has_domain && has_pattern { 1.0 } else if has_domain || has_pattern { 0.5 } else { 0.0 }
            },
            ContextType::ArchitecturalDecision => {
                let has_decision = context.content.data.get("decision").is_some();
                let has_consequences = context.content.data.get("consequences").is_some();
                if has_decision && has_consequences { 1.0 } else if has_decision { 0.7 } else { 0.0 }
            },
            ContextType::SecurityPolicy => {
                let has_area = context.content.data.get("policy_area").is_some();
                let has_pattern = context.content.data.get("implementation_pattern").is_some();
                if has_area && has_pattern { 1.0 } else if has_area { 0.6 } else { 0.0 }
            },
            _ => 0.5, // Default score for other types
        };
        score += type_specific_score;
        
        metrics.completeness_score = score / total_checks as f64;
        
        // Add issues for missing information
        if metrics.completeness_score < 0.7 {
            metrics.issues.push(QualityIssue {
                issue_type: QualityIssueType::MissingInformation,
                severity: IssueSeverity::Medium,
                description: "Context is missing important information".to_string(),
                field: None,
                detected_at: Utc::now(),
            });
        }
        
        Ok(())
    }
}

/// Analyzer for content consistency
struct ConsistencyAnalyzer;

impl ConsistencyAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl QualityAnalyzer for ConsistencyAnalyzer {
    async fn analyze(&self, context: &EnhancedContextItem, related_contexts: &[EnhancedContextItem], metrics: &mut QualityMetrics) -> Result<()> {
        let mut consistency_score = 1.0;
        
        // Check for conflicting relationships
        let conflicting_relationships = context.relationships.iter()
            .filter(|r| r.relationship_type == RelationshipType::Conflicts)
            .count();
        
        if conflicting_relationships > 0 {
            consistency_score -= 0.2 * conflicting_relationships as f64;
            
            metrics.issues.push(QualityIssue {
                issue_type: QualityIssueType::ConflictingRules,
                severity: IssueSeverity::High,
                description: format!("Context has {} conflicting relationships", conflicting_relationships),
                field: None,
                detected_at: Utc::now(),
            });
        }
        
        // Check for duplicate content
        let similar_contexts = related_contexts.iter()
            .filter(|other| other.id != context.id && other.project_id == context.project_id)
            .filter(|other| {
                let title_similarity = calculate_similarity(&context.content.title, &other.content.title);
                let desc_similarity = calculate_similarity(&context.content.description, &other.content.description);
                title_similarity > 0.8 || desc_similarity > 0.8
            })
            .count();
        
        if similar_contexts > 0 {
            consistency_score -= 0.3;
            
            metrics.issues.push(QualityIssue {
                issue_type: QualityIssueType::DuplicateContent,
                severity: IssueSeverity::Medium,
                description: "Context appears to be similar to existing contexts".to_string(),
                field: None,
                detected_at: Utc::now(),
            });
        }
        
        metrics.consistency_score = consistency_score.max(0.0);
        
        Ok(())
    }
}

/// Analyzer for content accuracy
struct AccuracyAnalyzer;

impl AccuracyAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl QualityAnalyzer for AccuracyAnalyzer {
    async fn analyze(&self, context: &EnhancedContextItem, _related_contexts: &[EnhancedContextItem], metrics: &mut QualityMetrics) -> Result<()> {
        // Base accuracy on confidence and validation status
        let mut accuracy_score = context.metadata.confidence;
        
        // Adjust based on validation status
        match context.metadata.validation_status {
            ValidationStatus::Valid => accuracy_score *= 1.0,
            ValidationStatus::Pending => accuracy_score *= 0.8,
            ValidationStatus::NeedsReview => accuracy_score *= 0.6,
            ValidationStatus::Invalid => accuracy_score *= 0.3,
            ValidationStatus::Outdated => accuracy_score *= 0.4,
        }
        
        // Adjust based on source reliability
        match context.metadata.source {
            ContextSource::Manual => accuracy_score *= 0.9,
            ContextSource::AutoDetected => accuracy_score *= 0.7,
            ContextSource::CodeAnalysis => accuracy_score *= 0.8,
            ContextSource::Documentation => accuracy_score *= 0.85,
            ContextSource::Git => accuracy_score *= 0.75,
            ContextSource::Plugin(_) => accuracy_score *= 0.7,
        }
        
        metrics.accuracy_score = accuracy_score;
        
        Ok(())
    }
}

/// Analyzer for content relevance
struct RelevanceAnalyzer;

impl RelevanceAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl QualityAnalyzer for RelevanceAnalyzer {
    async fn analyze(&self, context: &EnhancedContextItem, _related_contexts: &[EnhancedContextItem], metrics: &mut QualityMetrics) -> Result<()> {
        // Base relevance on usage statistics
        let usage_score = if context.usage_stats.total_queries == 0 {
            0.5 // Neutral score for unused contexts
        } else {
            let success_rate = context.usage_stats.success_rate();
            let frequency_score = match context.usage_stats.usage_frequency {
                UsageFrequency::Never => 0.1,
                UsageFrequency::Rare => 0.3,
                UsageFrequency::Occasional => 0.6,
                UsageFrequency::Regular => 0.8,
                UsageFrequency::Frequent => 1.0,
            };
            
            (success_rate + frequency_score) / 2.0
        };
        
        // Adjust based on priority
        let priority_multiplier = context.metadata.priority.score();
        
        metrics.relevance_score = usage_score * priority_multiplier;
        
        // Flag low usage contexts
        if context.usage_stats.total_queries == 0 && 
           (Utc::now() - context.created_at) > Duration::days(30) {
            metrics.issues.push(QualityIssue {
                issue_type: QualityIssueType::LowUsage,
                severity: IssueSeverity::Low,
                description: "Context has not been used in over 30 days".to_string(),
                field: None,
                detected_at: Utc::now(),
            });
        }
        
        Ok(())
    }
}

/// Analyzer for content freshness
struct FreshnessAnalyzer;

impl FreshnessAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl QualityAnalyzer for FreshnessAnalyzer {
    async fn analyze(&self, context: &EnhancedContextItem, _related_contexts: &[EnhancedContextItem], metrics: &mut QualityMetrics) -> Result<()> {
        let now = Utc::now();
        let age = now - context.updated_at;
        
        // Calculate freshness based on age
        let freshness_score = if age <= Duration::days(7) {
            1.0
        } else if age <= Duration::days(30) {
            0.8
        } else if age <= Duration::days(90) {
            0.6
        } else if age <= Duration::days(180) {
            0.4
        } else {
            0.2
        };
        
        metrics.freshness_score = freshness_score;
        
        // Flag outdated content
        if age > Duration::days(180) {
            metrics.issues.push(QualityIssue {
                issue_type: QualityIssueType::OutdatedContent,
                severity: IssueSeverity::Medium,
                description: "Context has not been updated in over 6 months".to_string(),
                field: None,
                detected_at: Utc::now(),
            });
        }
        
        Ok(())
    }
}

/// Trait for generating improvement suggestions
#[async_trait]
trait ImprovementGenerator: Send + Sync {
    async fn generate_suggestions(&self, context: &EnhancedContextItem, metrics: &QualityMetrics) -> Result<Vec<QualityImprovement>>;
}

/// Generator for completeness improvements
struct CompletenessImprover;

impl CompletenessImprover {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ImprovementGenerator for CompletenessImprover {
    async fn generate_suggestions(&self, context: &EnhancedContextItem, metrics: &QualityMetrics) -> Result<Vec<QualityImprovement>> {
        let mut suggestions = Vec::new();
        
        if metrics.completeness_score < 0.7 {
            if context.content.description.len() < 20 {
                suggestions.push(QualityImprovement {
                    suggestion_type: ImprovementType::ImproveDescription,
                    description: "Add a more detailed description explaining the context and its purpose".to_string(),
                    priority: Priority::High,
                    estimated_impact: 0.3,
                    action_required: "Expand the description to at least 20 characters with meaningful content".to_string(),
                });
            }
            
            if context.metadata.tags.is_empty() {
                suggestions.push(QualityImprovement {
                    suggestion_type: ImprovementType::UpdateTags,
                    description: "Add relevant tags to improve discoverability".to_string(),
                    priority: Priority::Medium,
                    estimated_impact: 0.2,
                    action_required: "Add 2-5 relevant tags that describe the context domain and purpose".to_string(),
                });
            }
            
            if context.semantic_tags.is_empty() {
                suggestions.push(QualityImprovement {
                    suggestion_type: ImprovementType::AddMissingInfo,
                    description: "Add semantic tags for better categorization".to_string(),
                    priority: Priority::Medium,
                    estimated_impact: 0.15,
                    action_required: "Review content and add appropriate semantic tags".to_string(),
                });
            }
        }
        
        Ok(suggestions)
    }
}

/// Generator for consistency improvements
struct ConsistencyImprover;

impl ConsistencyImprover {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ImprovementGenerator for ConsistencyImprover {
    async fn generate_suggestions(&self, _context: &EnhancedContextItem, metrics: &QualityMetrics) -> Result<Vec<QualityImprovement>> {
        let mut suggestions = Vec::new();
        
        if metrics.consistency_score < 0.7 {
            let has_conflicts = metrics.issues.iter().any(|i| i.issue_type == QualityIssueType::ConflictingRules);
            let has_duplicates = metrics.issues.iter().any(|i| i.issue_type == QualityIssueType::DuplicateContent);
            
            if has_conflicts {
                suggestions.push(QualityImprovement {
                    suggestion_type: ImprovementType::ResolveConflict,
                    description: "Resolve conflicting relationships with other contexts".to_string(),
                    priority: Priority::High,
                    estimated_impact: 0.4,
                    action_required: "Review conflicting contexts and resolve inconsistencies".to_string(),
                });
            }
            
            if has_duplicates {
                suggestions.push(QualityImprovement {
                    suggestion_type: ImprovementType::ArchiveUnused,
                    description: "Consider merging or removing duplicate content".to_string(),
                    priority: Priority::Medium,
                    estimated_impact: 0.3,
                    action_required: "Review similar contexts and consolidate or differentiate them".to_string(),
                });
            }
        }
        
        Ok(suggestions)
    }
}

/// Generator for general content improvements
struct ContentImprover;

impl ContentImprover {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ImprovementGenerator for ContentImprover {
    async fn generate_suggestions(&self, context: &EnhancedContextItem, metrics: &QualityMetrics) -> Result<Vec<QualityImprovement>> {
        let mut suggestions = Vec::new();
        
        if metrics.freshness_score < 0.5 {
            suggestions.push(QualityImprovement {
                suggestion_type: ImprovementType::UpdateContent,
                description: "Content appears to be outdated and should be reviewed".to_string(),
                priority: Priority::Medium,
                estimated_impact: 0.3,
                action_required: "Review and update content to reflect current state".to_string(),
            });
        }
        
        if metrics.relevance_score < 0.4 {
            suggestions.push(QualityImprovement {
                suggestion_type: ImprovementType::ArchiveUnused,
                description: "Context has low usage and may be candidates for archival".to_string(),
                priority: Priority::Low,
                estimated_impact: 0.1,
                action_required: "Review usage patterns and consider archiving if no longer relevant".to_string(),
            });
        }
        
        if context.relationships.is_empty() {
            suggestions.push(QualityImprovement {
                suggestion_type: ImprovementType::AddRelationships,
                description: "Add relationships to other relevant contexts".to_string(),
                priority: Priority::Medium,
                estimated_impact: 0.25,
                action_required: "Identify and link related contexts to improve discoverability".to_string(),
            });
        }
        
        Ok(suggestions)
    }
}

/// Simple text similarity calculation
fn calculate_similarity(text1: &str, text2: &str) -> f64 {
    let text1_lower = text1.to_lowercase();
    let text2_lower = text2.to_lowercase();
    let words1: HashSet<_> = text1_lower.split_whitespace().collect();
    let words2: HashSet<_> = text2_lower.split_whitespace().collect();
    
    if words1.is_empty() || words2.is_empty() {
        return 0.0;
    }
    
    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();
    
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_context(id: &str, project_id: &str, content_type: ContextType, title: &str, description: &str) -> EnhancedContextItem {
        let content = ContextContent {
            content_type,
            title: title.to_string(),
            description: description.to_string(),
            data: json!({}),
            source_file: None,
            source_line: None,
        };
        
        let mut context = EnhancedContextItem::new(project_id.to_string(), content);
        context.id = id.to_string();
        context
    }

    #[tokio::test]
    async fn test_quality_metrics_calculation() {
        let service = DefaultContextQualityService::new();
        
        let context = create_test_context(
            "1",
            "project1",
            ContextType::BusinessRule,
            "User Authentication Rule",
            "Users must authenticate using OAuth2 with proper validation and error handling"
        );
        
        let metrics = service.calculate_quality_metrics(&context, &[]).await.unwrap();
        
        assert!(metrics.overall_score > 0.0);
        assert!(metrics.completeness_score > 0.0);
        assert!(metrics.consistency_score > 0.0);
    }

    #[tokio::test]
    async fn test_context_validation() {
        let service = DefaultContextQualityService::new();
        
        let mut context = create_test_context(
            "1",
            "project1",
            ContextType::BusinessRule,
            "Test Rule",
            "Short desc"
        );
        
        // Add domain area to make it more complete
        context.content.data = json!({
            "domain_area": "authentication",
            "implementation_pattern": "OAuth2"
        });
        
        let rules = service.get_validation_rules(&ContextType::BusinessRule);
        let result = service.validate_context(&context, &rules).await.unwrap();
        
        assert!(result.validation_score > 0.0);
    }

    #[tokio::test]
    async fn test_improvement_suggestions() {
        let service = DefaultContextQualityService::new();
        
        let context = create_test_context(
            "1",
            "project1",
            ContextType::BusinessRule,
            "Rule",
            "Short"
        );
        
        let metrics = service.calculate_quality_metrics(&context, &[]).await.unwrap();
        let suggestions = service.generate_improvement_suggestions(&context, &metrics).await.unwrap();
        
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.suggestion_type == ImprovementType::ImproveDescription));
    }

    #[tokio::test]
    async fn test_batch_assessment() {
        let service = DefaultContextQualityService::new();
        
        let contexts = vec![
            create_test_context("1", "project1", ContextType::BusinessRule, "Rule 1", "Description 1"),
            create_test_context("2", "project1", ContextType::SecurityPolicy, "Policy 1", "Description 2"),
        ];
        
        let assessments = service.assess_context_batch(&contexts).await.unwrap();
        
        assert_eq!(assessments.len(), 2);
        assert!(assessments.iter().all(|a| a.quality_metrics.overall_score >= 0.0));
    }

    #[test]
    fn test_validation_rules() {
        let service = DefaultContextQualityService::new();
        
        let business_rules = service.get_validation_rules(&ContextType::BusinessRule);
        assert!(!business_rules.is_empty());
        
        let security_rules = service.get_validation_rules(&ContextType::SecurityPolicy);
        assert!(!security_rules.is_empty());
    }

    #[test]
    fn test_text_similarity() {
        let similarity = calculate_similarity("hello world", "world hello");
        assert!(similarity > 0.8);
        
        let no_similarity = calculate_similarity("hello", "goodbye");
        assert!(no_similarity < 0.5);
    }
}