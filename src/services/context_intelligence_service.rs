use crate::models::enhanced_context::*;
use crate::services::{
    ContextRelationshipEngine, DefaultContextRelationshipEngine,
    ContextQualityService, DefaultContextQualityService,
};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use anyhow::Result;

/// Service that orchestrates context intelligence features
#[async_trait]
pub trait ContextIntelligenceService: Send + Sync {
    /// Analyze a context item and provide comprehensive intelligence
    async fn analyze_context(&self, context: &EnhancedContextItem, all_contexts: &[EnhancedContextItem]) -> Result<ContextIntelligence>;
    
    /// Suggest related contexts based on current query
    async fn suggest_related_contexts(&self, query: &ContextQuery, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextSuggestion>>;
    
    /// Provide context recommendations for improving AI agent results
    async fn recommend_context_improvements(&self, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextRecommendation>>;
    
    /// Analyze context gaps and suggest missing information
    async fn analyze_context_gaps(&self, project_contexts: &[EnhancedContextItem]) -> Result<Vec<ContextGap>>;
    
    /// Generate context insights for a project
    async fn generate_project_insights(&self, project_id: &str, contexts: &[EnhancedContextItem]) -> Result<ProjectContextInsights>;
}

/// Default implementation of the Context Intelligence Service
pub struct DefaultContextIntelligenceService {
    relationship_engine: Box<dyn ContextRelationshipEngine>,
    quality_service: Box<dyn ContextQualityService>,
    suggestion_generators: Vec<Box<dyn SuggestionGenerator>>,
    gap_analyzers: Vec<Box<dyn GapAnalyzer>>,
}

impl DefaultContextIntelligenceService {
    pub fn new() -> Self {
        Self {
            relationship_engine: Box::new(DefaultContextRelationshipEngine::new()),
            quality_service: Box::new(DefaultContextQualityService::new()),
            suggestion_generators: vec![
                Box::new(SimilarityBasedSuggestionGenerator::new()),
                Box::new(TypeBasedSuggestionGenerator::new()),
                Box::new(UsagePatternSuggestionGenerator::new()),
            ],
            gap_analyzers: vec![
                Box::new(TypeCoverageAnalyzer::new()),
                Box::new(RelationshipGapAnalyzer::new()),
                Box::new(QualityGapAnalyzer::new()),
            ],
        }
    }

    pub fn with_relationship_engine(mut self, engine: Box<dyn ContextRelationshipEngine>) -> Self {
        self.relationship_engine = engine;
        self
    }

    pub fn with_quality_service(mut self, service: Box<dyn ContextQualityService>) -> Self {
        self.quality_service = service;
        self
    }
}

#[async_trait]
impl ContextIntelligenceService for DefaultContextIntelligenceService {
    async fn analyze_context(&self, context: &EnhancedContextItem, all_contexts: &[EnhancedContextItem]) -> Result<ContextIntelligence> {
        // Get quality metrics
        let quality_metrics = self.quality_service.calculate_quality_metrics(context, all_contexts).await?;
        
        // Get validation results
        let validation_rules = self.quality_service.get_validation_rules(&context.content.content_type);
        let validation_result = self.quality_service.validate_context(context, &validation_rules).await?;
        
        // Get improvement suggestions
        let improvement_suggestions = self.quality_service.generate_improvement_suggestions(context, &quality_metrics).await?;
        
        // Detect relationships
        let relationships = self.relationship_engine.detect_relationships(context, all_contexts).await?;
        
        // Find related contexts
        let related_contexts = self.relationship_engine.find_related_contexts(&context.id, all_contexts, 3).await?;
        
        // Calculate intelligence score
        let intelligence_score = calculate_intelligence_score(&quality_metrics, &validation_result, &relationships);
        
        Ok(ContextIntelligence {
            context_id: context.id.clone(),
            quality_metrics,
            validation_result,
            improvement_suggestions,
            detected_relationships: relationships,
            related_contexts,
            intelligence_score,
            analyzed_at: Utc::now(),
        })
    }

    async fn suggest_related_contexts(&self, query: &ContextQuery, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextSuggestion>> {
        let mut suggestions = Vec::new();
        
        // Run all suggestion generators
        for generator in &self.suggestion_generators {
            let generated = generator.generate_suggestions(query, contexts).await?;
            suggestions.extend(generated);
        }
        
        // Remove duplicates and sort by relevance
        let mut suggestion_map: HashMap<String, ContextSuggestion> = HashMap::new();
        
        for suggestion in suggestions {
            let key = suggestion.context_id.clone();
            
            if let Some(existing) = suggestion_map.get(&key) {
                if suggestion.relevance_score > existing.relevance_score {
                    suggestion_map.insert(key, suggestion);
                }
            } else {
                suggestion_map.insert(key, suggestion);
            }
        }
        
        let mut final_suggestions: Vec<_> = suggestion_map.into_values().collect();
        final_suggestions.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit to top 10 suggestions
        final_suggestions.truncate(10);
        
        Ok(final_suggestions)
    }

    async fn recommend_context_improvements(&self, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextRecommendation>> {
        let mut recommendations = Vec::new();
        
        // Analyze each context for improvement opportunities
        for context in contexts {
            let intelligence = self.analyze_context(context, contexts).await?;
            
            // Generate recommendations based on intelligence analysis
            if intelligence.intelligence_score < 0.7 {
                let recommendation = ContextRecommendation {
                    context_id: context.id.clone(),
                    recommendation_type: RecommendationType::QualityImprovement,
                    title: "Improve Context Quality".to_string(),
                    description: format!("Context has low intelligence score ({:.2})", intelligence.intelligence_score),
                    priority: if intelligence.intelligence_score < 0.5 { Priority::High } else { Priority::Medium },
                    estimated_impact: 0.3,
                    actions: intelligence.improvement_suggestions.iter()
                        .map(|s| s.action_required.clone())
                        .collect(),
                    created_at: Utc::now(),
                };
                
                recommendations.push(recommendation);
            }
            
            // Check for missing relationships
            if intelligence.detected_relationships.is_empty() && contexts.len() > 1 {
                let recommendation = ContextRecommendation {
                    context_id: context.id.clone(),
                    recommendation_type: RecommendationType::AddRelationships,
                    title: "Add Context Relationships".to_string(),
                    description: "Context has no relationships to other contexts".to_string(),
                    priority: Priority::Medium,
                    estimated_impact: 0.2,
                    actions: vec!["Review related contexts and establish appropriate relationships".to_string()],
                    created_at: Utc::now(),
                };
                
                recommendations.push(recommendation);
            }
        }
        
        // Sort by priority and impact
        recommendations.sort_by(|a, b| {
            b.priority.score().partial_cmp(&a.priority.score())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.estimated_impact.partial_cmp(&a.estimated_impact).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        Ok(recommendations)
    }

    async fn analyze_context_gaps(&self, project_contexts: &[EnhancedContextItem]) -> Result<Vec<ContextGap>> {
        let mut gaps = Vec::new();
        
        // Run all gap analyzers
        for analyzer in &self.gap_analyzers {
            let detected_gaps = analyzer.analyze_gaps(project_contexts).await?;
            gaps.extend(detected_gaps);
        }
        
        // Sort by severity and impact
        gaps.sort_by(|a, b| {
            b.severity.partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.impact_score.partial_cmp(&a.impact_score).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        Ok(gaps)
    }

    async fn generate_project_insights(&self, project_id: &str, contexts: &[EnhancedContextItem]) -> Result<ProjectContextInsights> {
        let project_contexts: Vec<_> = contexts.iter()
            .filter(|c| c.project_id == project_id)
            .cloned()
            .collect();
        
        if project_contexts.is_empty() {
            return Ok(ProjectContextInsights {
                project_id: project_id.to_string(),
                total_contexts: 0,
                context_type_distribution: HashMap::new(),
                average_quality_score: 0.0,
                relationship_density: 0.0,
                context_gaps: Vec::new(),
                recommendations: Vec::new(),
                top_quality_contexts: Vec::new(),
                low_quality_contexts: Vec::new(),
                generated_at: Utc::now(),
            });
        }
        
        // Calculate context type distribution
        let mut type_distribution = HashMap::new();
        for context in &project_contexts {
            *type_distribution.entry(context.content.content_type.clone()).or_insert(0) += 1;
        }
        
        // Calculate average quality score
        let mut total_quality = 0.0;
        let mut quality_contexts = Vec::new();
        
        for context in &project_contexts {
            let quality_metrics = self.quality_service.calculate_quality_metrics(context, &project_contexts).await?;
            total_quality += quality_metrics.overall_score;
            quality_contexts.push((context.clone(), quality_metrics));
        }
        
        let average_quality = total_quality / project_contexts.len() as f64;
        
        // Sort contexts by quality
        quality_contexts.sort_by(|a, b| b.1.overall_score.partial_cmp(&a.1.overall_score).unwrap_or(std::cmp::Ordering::Equal));
        
        let top_quality_contexts = quality_contexts.iter()
            .take(5)
            .map(|(context, metrics)| QualityContextInfo {
                context_id: context.id.clone(),
                title: context.content.title.clone(),
                quality_score: metrics.overall_score,
                context_type: context.content.content_type.clone(),
            })
            .collect();
        
        let low_quality_contexts = quality_contexts.iter()
            .rev()
            .take(5)
            .filter(|(_, metrics)| metrics.overall_score < 0.6)
            .map(|(context, metrics)| QualityContextInfo {
                context_id: context.id.clone(),
                title: context.content.title.clone(),
                quality_score: metrics.overall_score,
                context_type: context.content.content_type.clone(),
            })
            .collect();
        
        // Calculate relationship density
        let total_possible_relationships = if project_contexts.len() > 1 {
            project_contexts.len() * (project_contexts.len() - 1)
        } else {
            1
        };
        
        let actual_relationships: usize = project_contexts.iter()
            .map(|c| c.relationships.len())
            .sum();
        
        let relationship_density = actual_relationships as f64 / total_possible_relationships as f64;
        
        // Analyze gaps and get recommendations
        let context_gaps = self.analyze_context_gaps(&project_contexts).await?;
        let recommendations = self.recommend_context_improvements(&project_contexts).await?;
        
        Ok(ProjectContextInsights {
            project_id: project_id.to_string(),
            total_contexts: project_contexts.len(),
            context_type_distribution: type_distribution,
            average_quality_score: average_quality,
            relationship_density,
            context_gaps,
            recommendations,
            top_quality_contexts,
            low_quality_contexts,
            generated_at: Utc::now(),
        })
    }
}

/// Comprehensive intelligence analysis for a context item
#[derive(Debug, Clone)]
pub struct ContextIntelligence {
    pub context_id: ContextId,
    pub quality_metrics: QualityMetrics,
    pub validation_result: ValidationResult,
    pub improvement_suggestions: Vec<QualityImprovement>,
    pub detected_relationships: Vec<ContextRelationship>,
    pub related_contexts: Vec<RelatedContext>,
    pub intelligence_score: f64,
    pub analyzed_at: DateTime<Utc>,
}

/// Query for context suggestions
#[derive(Debug, Clone)]
pub struct ContextQuery {
    pub project_id: String,
    pub query_text: String,
    pub context_types: Vec<ContextType>,
    pub tags: Vec<String>,
    pub max_results: usize,
}

impl ContextQuery {
    pub fn new(project_id: String, query_text: String) -> Self {
        Self {
            project_id,
            query_text,
            context_types: Vec::new(),
            tags: Vec::new(),
            max_results: 10,
        }
    }

    pub fn with_types(mut self, types: Vec<ContextType>) -> Self {
        self.context_types = types;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_max_results(mut self, max_results: usize) -> Self {
        self.max_results = max_results;
        self
    }
}

/// Context suggestion with relevance information
#[derive(Debug, Clone)]
pub struct ContextSuggestion {
    pub context_id: ContextId,
    pub title: String,
    pub description: String,
    pub context_type: ContextType,
    pub relevance_score: f64,
    pub relevance_reasons: Vec<String>,
    pub suggested_at: DateTime<Utc>,
}

/// Recommendation for context improvements
#[derive(Debug, Clone)]
pub struct ContextRecommendation {
    pub context_id: ContextId,
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub estimated_impact: f64,
    pub actions: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Types of recommendations
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationType {
    QualityImprovement,
    AddRelationships,
    ResolveConflicts,
    UpdateContent,
    AddMissingInfo,
    ArchiveUnused,
}

/// Context gap analysis
#[derive(Debug, Clone)]
pub struct ContextGap {
    pub gap_type: GapType,
    pub title: String,
    pub description: String,
    pub severity: f64,
    pub impact_score: f64,
    pub suggested_actions: Vec<String>,
    pub detected_at: DateTime<Utc>,
}

/// Types of context gaps
#[derive(Debug, Clone, PartialEq)]
pub enum GapType {
    MissingContextType,
    InsufficientCoverage,
    QualityIssues,
    RelationshipGaps,
    OutdatedContent,
}

/// Project-level context insights
#[derive(Debug, Clone)]
pub struct ProjectContextInsights {
    pub project_id: String,
    pub total_contexts: usize,
    pub context_type_distribution: HashMap<ContextType, usize>,
    pub average_quality_score: f64,
    pub relationship_density: f64,
    pub context_gaps: Vec<ContextGap>,
    pub recommendations: Vec<ContextRecommendation>,
    pub top_quality_contexts: Vec<QualityContextInfo>,
    pub low_quality_contexts: Vec<QualityContextInfo>,
    pub generated_at: DateTime<Utc>,
}

/// Context information with quality score
#[derive(Debug, Clone)]
pub struct QualityContextInfo {
    pub context_id: ContextId,
    pub title: String,
    pub quality_score: f64,
    pub context_type: ContextType,
}

/// Calculate intelligence score based on various factors
fn calculate_intelligence_score(
    quality_metrics: &QualityMetrics,
    validation_result: &ValidationResult,
    relationships: &[ContextRelationship],
) -> f64 {
    let quality_weight = 0.4;
    let validation_weight = 0.3;
    let relationship_weight = 0.3;
    
    let quality_score = quality_metrics.overall_score;
    let validation_score = validation_result.validation_score;
    
    // Relationship score based on number and strength of relationships
    let relationship_score = if relationships.is_empty() {
        0.5 // Neutral score for no relationships
    } else {
        let avg_strength: f64 = relationships.iter().map(|r| r.strength).sum::<f64>() / relationships.len() as f64;
        let count_factor = (relationships.len() as f64 / 10.0).min(1.0); // Max benefit at 10 relationships
        (avg_strength + count_factor) / 2.0
    };
    
    (quality_score * quality_weight + validation_score * validation_weight + relationship_score * relationship_weight)
        .min(1.0)
        .max(0.0)
}

/// Trait for generating context suggestions
#[async_trait]
trait SuggestionGenerator: Send + Sync {
    async fn generate_suggestions(&self, query: &ContextQuery, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextSuggestion>>;
}

/// Similarity-based suggestion generator
struct SimilarityBasedSuggestionGenerator;

impl SimilarityBasedSuggestionGenerator {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SuggestionGenerator for SimilarityBasedSuggestionGenerator {
    async fn generate_suggestions(&self, query: &ContextQuery, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextSuggestion>> {
        let mut suggestions = Vec::new();
        
        for context in contexts {
            if context.project_id != query.project_id {
                continue;
            }
            
            let title_similarity = calculate_text_similarity(&query.query_text, &context.content.title);
            let desc_similarity = calculate_text_similarity(&query.query_text, &context.content.description);
            let relevance_score = (title_similarity * 0.6 + desc_similarity * 0.4).max(0.0).min(1.0);
            
            if relevance_score > 0.2 {
                let mut reasons = Vec::new();
                if title_similarity > 0.3 {
                    reasons.push("Similar title content".to_string());
                }
                if desc_similarity > 0.3 {
                    reasons.push("Similar description content".to_string());
                }
                
                suggestions.push(ContextSuggestion {
                    context_id: context.id.clone(),
                    title: context.content.title.clone(),
                    description: context.content.description.clone(),
                    context_type: context.content.content_type.clone(),
                    relevance_score,
                    relevance_reasons: reasons,
                    suggested_at: Utc::now(),
                });
            }
        }
        
        Ok(suggestions)
    }
}

/// Type-based suggestion generator
struct TypeBasedSuggestionGenerator;

impl TypeBasedSuggestionGenerator {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SuggestionGenerator for TypeBasedSuggestionGenerator {
    async fn generate_suggestions(&self, query: &ContextQuery, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextSuggestion>> {
        let mut suggestions = Vec::new();
        
        if query.context_types.is_empty() {
            return Ok(suggestions);
        }
        
        for context in contexts {
            if context.project_id != query.project_id {
                continue;
            }
            
            if query.context_types.contains(&context.content.content_type) {
                suggestions.push(ContextSuggestion {
                    context_id: context.id.clone(),
                    title: context.content.title.clone(),
                    description: context.content.description.clone(),
                    context_type: context.content.content_type.clone(),
                    relevance_score: 0.8,
                    relevance_reasons: vec!["Matches requested context type".to_string()],
                    suggested_at: Utc::now(),
                });
            }
        }
        
        Ok(suggestions)
    }
}

/// Usage pattern-based suggestion generator
struct UsagePatternSuggestionGenerator;

impl UsagePatternSuggestionGenerator {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SuggestionGenerator for UsagePatternSuggestionGenerator {
    async fn generate_suggestions(&self, query: &ContextQuery, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextSuggestion>> {
        let mut suggestions = Vec::new();
        
        for context in contexts {
            if context.project_id != query.project_id {
                continue;
            }
            
            // Suggest frequently used contexts
            let usage_score = match context.usage_stats.usage_frequency {
                UsageFrequency::Frequent => 0.9,
                UsageFrequency::Regular => 0.7,
                UsageFrequency::Occasional => 0.5,
                UsageFrequency::Rare => 0.3,
                UsageFrequency::Never => 0.1,
            };
            
            let success_rate = context.usage_stats.success_rate();
            let relevance_score = (usage_score + success_rate) / 2.0;
            
            if relevance_score > 0.4 {
                let mut reasons = Vec::new();
                if usage_score > 0.6 {
                    reasons.push("Frequently used context".to_string());
                }
                if success_rate > 0.7 {
                    reasons.push("High success rate in previous queries".to_string());
                }
                
                suggestions.push(ContextSuggestion {
                    context_id: context.id.clone(),
                    title: context.content.title.clone(),
                    description: context.content.description.clone(),
                    context_type: context.content.content_type.clone(),
                    relevance_score,
                    relevance_reasons: reasons,
                    suggested_at: Utc::now(),
                });
            }
        }
        
        Ok(suggestions)
    }
}

/// Trait for analyzing context gaps
#[async_trait]
trait GapAnalyzer: Send + Sync {
    async fn analyze_gaps(&self, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextGap>>;
}

/// Analyzer for context type coverage gaps
struct TypeCoverageAnalyzer;

impl TypeCoverageAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GapAnalyzer for TypeCoverageAnalyzer {
    async fn analyze_gaps(&self, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextGap>> {
        let mut gaps = Vec::new();
        
        // Expected context types for a complete project
        let expected_types = vec![
            ContextType::BusinessRule,
            ContextType::ArchitecturalDecision,
            ContextType::SecurityPolicy,
            ContextType::PerformanceRequirement,
        ];
        
        let existing_types: HashSet<_> = contexts.iter()
            .map(|c| &c.content.content_type)
            .collect();
        
        for expected_type in expected_types {
            if !existing_types.contains(&expected_type) {
                gaps.push(ContextGap {
                    gap_type: GapType::MissingContextType,
                    title: format!("Missing {} contexts", expected_type.as_str()),
                    description: format!("Project lacks {} context items", expected_type.as_str()),
                    severity: 0.7,
                    impact_score: 0.8,
                    suggested_actions: vec![
                        format!("Add {} context items to improve project coverage", expected_type.as_str())
                    ],
                    detected_at: Utc::now(),
                });
            }
        }
        
        Ok(gaps)
    }
}

/// Analyzer for relationship gaps
struct RelationshipGapAnalyzer;

impl RelationshipGapAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GapAnalyzer for RelationshipGapAnalyzer {
    async fn analyze_gaps(&self, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextGap>> {
        let mut gaps = Vec::new();
        
        let isolated_contexts: Vec<_> = contexts.iter()
            .filter(|c| c.relationships.is_empty())
            .collect();
        
        if !isolated_contexts.is_empty() && contexts.len() > 1 {
            gaps.push(ContextGap {
                gap_type: GapType::RelationshipGaps,
                title: "Isolated contexts detected".to_string(),
                description: format!("{} contexts have no relationships to other contexts", isolated_contexts.len()),
                severity: 0.6,
                impact_score: 0.5,
                suggested_actions: vec![
                    "Review isolated contexts and establish appropriate relationships".to_string()
                ],
                detected_at: Utc::now(),
            });
        }
        
        Ok(gaps)
    }
}

/// Analyzer for quality gaps
struct QualityGapAnalyzer;

impl QualityGapAnalyzer {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GapAnalyzer for QualityGapAnalyzer {
    async fn analyze_gaps(&self, contexts: &[EnhancedContextItem]) -> Result<Vec<ContextGap>> {
        let mut gaps = Vec::new();
        
        let low_quality_contexts: Vec<_> = contexts.iter()
            .filter(|c| c.quality_score < 0.5)
            .collect();
        
        if !low_quality_contexts.is_empty() {
            gaps.push(ContextGap {
                gap_type: GapType::QualityIssues,
                title: "Low quality contexts detected".to_string(),
                description: format!("{} contexts have quality scores below 0.5", low_quality_contexts.len()),
                severity: 0.8,
                impact_score: 0.7,
                suggested_actions: vec![
                    "Review and improve low quality contexts".to_string(),
                    "Add missing information and improve descriptions".to_string(),
                ],
                detected_at: Utc::now(),
            });
        }
        
        Ok(gaps)
    }
}

/// Simple text similarity calculation
fn calculate_text_similarity(text1: &str, text2: &str) -> f64 {
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

// Import the missing types from context_quality_service
use crate::services::context_quality_service::ValidationResult;
use crate::services::context_relationship_engine::RelatedContext;

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
    async fn test_analyze_context() {
        let service = DefaultContextIntelligenceService::new();
        
        let context = create_test_context(
            "1",
            "project1",
            ContextType::BusinessRule,
            "User Authentication",
            "Users must authenticate using OAuth2 with proper validation"
        );
        
        let all_contexts = vec![
            context.clone(),
            create_test_context("2", "project1", ContextType::SecurityPolicy, "Auth Policy", "OAuth2 security requirements"),
        ];
        
        let intelligence = service.analyze_context(&context, &all_contexts).await.unwrap();
        
        assert_eq!(intelligence.context_id, "1");
        assert!(intelligence.intelligence_score >= 0.0);
        assert!(intelligence.intelligence_score <= 1.0);
    }

    #[tokio::test]
    async fn test_suggest_related_contexts() {
        let service = DefaultContextIntelligenceService::new();
        
        let query = ContextQuery::new("project1".to_string(), "authentication security".to_string());
        
        let contexts = vec![
            create_test_context("1", "project1", ContextType::BusinessRule, "User Authentication", "OAuth2 authentication rules"),
            create_test_context("2", "project1", ContextType::SecurityPolicy, "Auth Security", "Security policies for authentication"),
            create_test_context("3", "project1", ContextType::FeatureContext, "Login Feature", "User login functionality"),
        ];
        
        let suggestions = service.suggest_related_contexts(&query, &contexts).await.unwrap();
        
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().all(|s| s.relevance_score > 0.0));
    }

    #[tokio::test]
    async fn test_recommend_context_improvements() {
        let service = DefaultContextIntelligenceService::new();
        
        let contexts = vec![
            create_test_context("1", "project1", ContextType::BusinessRule, "Rule", "Short description"),
            create_test_context("2", "project1", ContextType::SecurityPolicy, "Policy", "Another short description"),
        ];
        
        let recommendations = service.recommend_context_improvements(&contexts).await.unwrap();
        
        // Should have recommendations for improvement due to short descriptions
        assert!(!recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_context_gaps() {
        let service = DefaultContextIntelligenceService::new();
        
        // Only business rules, missing other types
        let contexts = vec![
            create_test_context("1", "project1", ContextType::BusinessRule, "Rule 1", "Business rule description"),
            create_test_context("2", "project1", ContextType::BusinessRule, "Rule 2", "Another business rule"),
        ];
        
        let gaps = service.analyze_context_gaps(&contexts).await.unwrap();
        
        // Should detect missing context types
        assert!(!gaps.is_empty());
        assert!(gaps.iter().any(|g| g.gap_type == GapType::MissingContextType));
    }

    #[tokio::test]
    async fn test_generate_project_insights() {
        let service = DefaultContextIntelligenceService::new();
        
        let contexts = vec![
            create_test_context("1", "project1", ContextType::BusinessRule, "Rule 1", "Business rule description"),
            create_test_context("2", "project1", ContextType::SecurityPolicy, "Policy 1", "Security policy description"),
            create_test_context("3", "project2", ContextType::BusinessRule, "Rule 2", "Different project rule"),
        ];
        
        let insights = service.generate_project_insights("project1", &contexts).await.unwrap();
        
        assert_eq!(insights.project_id, "project1");
        assert_eq!(insights.total_contexts, 2);
        assert!(insights.context_type_distribution.contains_key(&ContextType::BusinessRule));
        assert!(insights.context_type_distribution.contains_key(&ContextType::SecurityPolicy));
    }

    #[test]
    fn test_calculate_intelligence_score() {
        let mut quality_metrics = QualityMetrics::new();
        quality_metrics.overall_score = 0.8;
        
        let mut validation_result = ValidationResult::new();
        validation_result.validation_score = 0.9;
        
        let relationships = vec![
            ContextRelationship::new("target1".to_string(), RelationshipType::DependsOn, 0.7, true),
            ContextRelationship::new("target2".to_string(), RelationshipType::Similar, 0.6, true),
        ];
        
        let score = calculate_intelligence_score(&quality_metrics, &validation_result, &relationships);
        
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_context_query_builder() {
        let query = ContextQuery::new("project1".to_string(), "test query".to_string())
            .with_types(vec![ContextType::BusinessRule])
            .with_tags(vec!["auth".to_string()])
            .with_max_results(5);
        
        assert_eq!(query.project_id, "project1");
        assert_eq!(query.query_text, "test query");
        assert_eq!(query.context_types, vec![ContextType::BusinessRule]);
        assert_eq!(query.tags, vec!["auth".to_string()]);
        assert_eq!(query.max_results, 5);
    }
}