use crate::models::context::{
    ArchitecturalDecision, BusinessRule, PerformanceRequirement,
};
use crate::models::enhanced_context::{EnhancedContextItem, ContextType};
use crate::models::embedding::VectorSearchQuery;
use crate::services::context_query_service::{ContextQueryService, ContextQueryResult};
use crate::services::semantic_search_service::{
    SemanticSearchService, EnhancedSearchResult, SemanticSearchError,
};
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Hybrid search result combining traditional and semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    pub semantic_results: Vec<EnhancedSearchResult>,
    pub traditional_results: ContextQueryResult,
    pub combined_score: f64,
    pub search_strategy: SearchStrategy,
    pub total_results: usize,
}

/// Search strategy used for hybrid search
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchStrategy {
    SemanticOnly,
    TraditionalOnly,
    Hybrid,
    IntentBased,
}

/// Configuration for hybrid search
#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    pub semantic_weight: f32,
    pub traditional_weight: f32,
    pub enable_intent_routing: bool,
    pub max_semantic_results: usize,
    pub similarity_threshold: f32,
    pub enable_result_fusion: bool,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            semantic_weight: 0.7,
            traditional_weight: 0.3,
            enable_intent_routing: true,
            max_semantic_results: 20,
            similarity_threshold: 0.6,
            enable_result_fusion: true,
        }
    }
}

/// Error types for hybrid search operations
#[derive(Debug, thiserror::Error)]
pub enum HybridSearchError {
    #[error("Semantic search error: {source}")]
    SemanticSearchError { source: SemanticSearchError },
    
    #[error("Traditional search error: {source}")]
    TraditionalSearchError { source: McpError },
    
    #[error("Result fusion error: {message}")]
    ResultFusionError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

impl From<SemanticSearchError> for HybridSearchError {
    fn from(error: SemanticSearchError) -> Self {
        HybridSearchError::SemanticSearchError { source: error }
    }
}

impl From<McpError> for HybridSearchError {
    fn from(error: McpError) -> Self {
        HybridSearchError::TraditionalSearchError { source: error }
    }
}

/// Trait for hybrid search operations
#[async_trait]
pub trait HybridSearchService: Send + Sync {
    /// Perform hybrid search combining semantic and traditional approaches
    async fn hybrid_search(
        &self,
        project_id: &str,
        query_text: &str,
        feature_area: Option<&str>,
        task_type: Option<&str>,
        components: &[String],
    ) -> Result<HybridSearchResult, HybridSearchError>;
    
    /// Perform semantic-only search
    async fn semantic_search(
        &self,
        query: &VectorSearchQuery,
    ) -> Result<Vec<EnhancedSearchResult>, HybridSearchError>;
    
    /// Perform traditional context query
    async fn traditional_search(
        &self,
        project_id: &str,
        feature_area: &str,
        task_type: &str,
        components: &[String],
    ) -> Result<ContextQueryResult, HybridSearchError>;
    
    /// Get search suggestions based on query intent
    async fn get_search_suggestions(
        &self,
        partial_query: &str,
        project_id: Option<&str>,
    ) -> Result<Vec<String>, HybridSearchError>;
}

/// Implementation of HybridSearchService
pub struct HybridSearchServiceImpl {
    semantic_search_service: Arc<dyn SemanticSearchService>,
    context_query_service: Arc<dyn ContextQueryService>,
    config: HybridSearchConfig,
}

impl HybridSearchServiceImpl {
    pub fn new(
        semantic_search_service: Arc<dyn SemanticSearchService>,
        context_query_service: Arc<dyn ContextQueryService>,
        config: HybridSearchConfig,
    ) -> Self {
        Self {
            semantic_search_service,
            context_query_service,
            config,
        }
    }
    
    /// Determine search strategy based on query intent and configuration
    fn determine_search_strategy(&self, query_text: &str) -> SearchStrategy {
        if !self.config.enable_intent_routing {
            return SearchStrategy::Hybrid;
        }
        
        // Simple heuristics for strategy selection
        let query_lower = query_text.to_lowercase();
        
        // If query is very specific to semantic concepts, use semantic only
        if query_lower.contains("similar") || query_lower.contains("like") || 
           query_lower.contains("related") || query_lower.contains("example") {
            return SearchStrategy::SemanticOnly;
        }
        
        // If query is asking for specific project context, use traditional
        if query_lower.contains("business rule") || query_lower.contains("architectural decision") ||
           query_lower.contains("performance requirement") {
            return SearchStrategy::TraditionalOnly;
        }
        
        // For natural language queries, use intent-based routing
        if query_lower.contains("how to") || query_lower.contains("what is") ||
           query_lower.contains("best practice") || query_lower.contains("implementation") {
            return SearchStrategy::IntentBased;
        }
        
        // Default to hybrid approach
        SearchStrategy::Hybrid
    }
    
    /// Fuse results from semantic and traditional search
    fn fuse_results(
        &self,
        semantic_results: Vec<EnhancedSearchResult>,
        traditional_results: ContextQueryResult,
        strategy: SearchStrategy,
    ) -> HybridSearchResult {
        if !self.config.enable_result_fusion {
            return HybridSearchResult {
                semantic_results,
                traditional_results,
                combined_score: 0.0,
                search_strategy: strategy,
                total_results: 0,
            };
        }
        
        // Calculate combined score based on result quality and relevance
        let semantic_score = self.calculate_semantic_results_score(&semantic_results);
        let traditional_score = self.calculate_traditional_results_score(&traditional_results);
        
        let combined_score = match strategy {
            SearchStrategy::SemanticOnly => semantic_score,
            SearchStrategy::TraditionalOnly => traditional_score,
            SearchStrategy::Hybrid | SearchStrategy::IntentBased => {
                semantic_score * self.config.semantic_weight as f64 +
                traditional_score * self.config.traditional_weight as f64
            }
        };
        
        let total_results = semantic_results.len() + 
                          traditional_results.business_rules.len() +
                          traditional_results.architectural_decisions.len() +
                          traditional_results.performance_requirements.len() +
                          traditional_results.security_policies.len() +
                          traditional_results.project_conventions.len();
        
        HybridSearchResult {
            semantic_results,
            traditional_results,
            combined_score,
            search_strategy: strategy,
            total_results,
        }
    }
    
    /// Calculate quality score for semantic results
    fn calculate_semantic_results_score(&self, results: &[EnhancedSearchResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        
        let total_score: f32 = results.iter()
            .map(|r| r.vector_result.similarity_score)
            .sum();
        
        (total_score / results.len() as f32) as f64
    }
    
    /// Calculate quality score for traditional results
    fn calculate_traditional_results_score(&self, results: &ContextQueryResult) -> f64 {
        let total_items = results.business_rules.len() +
                         results.architectural_decisions.len() +
                         results.performance_requirements.len() +
                         results.security_policies.len() +
                         results.project_conventions.len();
        
        if total_items == 0 {
            0.0
        } else {
            // Simple scoring based on number of results found
            // In a real implementation, this would consider result quality and relevance
            (total_items as f64 / 10.0).min(1.0)
        }
    }
    
    /// Convert traditional context results to enhanced context items for unified processing
    fn convert_traditional_to_enhanced(&self, traditional_results: &ContextQueryResult, project_id: &str) -> Vec<EnhancedContextItem> {
        let mut enhanced_items = Vec::new();
        
        // Convert business rules
        for rule in &traditional_results.business_rules {
            let content = crate::models::enhanced_context::ContextContent {
                content_type: ContextType::BusinessRule,
                title: rule.rule_name.clone(),
                description: rule.description.clone().unwrap_or_default(),
                data: serde_json::to_value(rule).unwrap_or_default(),
                source_file: None,
                source_line: None,
            };
            
            let mut item = EnhancedContextItem::new(project_id.to_string(), content);
            item.id = rule.id.clone();
            enhanced_items.push(item);
        }
        
        // Convert architectural decisions
        for decision in &traditional_results.architectural_decisions {
            let content = crate::models::enhanced_context::ContextContent {
                content_type: ContextType::ArchitecturalDecision,
                title: decision.decision_title.clone(),
                description: decision.context.clone().unwrap_or_default(),
                data: serde_json::to_value(decision).unwrap_or_default(),
                source_file: None,
                source_line: None,
            };
            
            let mut item = EnhancedContextItem::new(project_id.to_string(), content);
            item.id = decision.id.clone();
            enhanced_items.push(item);
        }
        
        // Convert performance requirements
        for requirement in &traditional_results.performance_requirements {
            let content = crate::models::enhanced_context::ContextContent {
                content_type: ContextType::PerformanceRequirement,
                title: requirement.requirement_type.clone().unwrap_or_default(),
                description: requirement.target_value.clone().unwrap_or_default(),
                data: serde_json::to_value(requirement).unwrap_or_default(),
                source_file: None,
                source_line: None,
            };
            
            let mut item = EnhancedContextItem::new(project_id.to_string(), content);
            item.id = requirement.id.clone();
            enhanced_items.push(item);
        }
        
        enhanced_items
    }
}

#[async_trait]
impl HybridSearchService for HybridSearchServiceImpl {
    async fn hybrid_search(
        &self,
        project_id: &str,
        query_text: &str,
        feature_area: Option<&str>,
        task_type: Option<&str>,
        components: &[String],
    ) -> Result<HybridSearchResult, HybridSearchError> {
        info!("Performing hybrid search for query: {}", query_text);
        
        let strategy = self.determine_search_strategy(query_text);
        debug!("Selected search strategy: {:?}", strategy);
        
        let mut semantic_results = Vec::new();
        let mut traditional_results = ContextQueryResult {
            business_rules: Vec::new(),
            architectural_decisions: Vec::new(),
            performance_requirements: Vec::new(),
            security_policies: Vec::new(),
            project_conventions: Vec::new(),
        };
        
        match strategy {
            SearchStrategy::SemanticOnly | SearchStrategy::Hybrid | SearchStrategy::IntentBased => {
                // Perform semantic search
                let query = VectorSearchQuery {
                    query_text: query_text.to_string(),
                    similarity_threshold: self.config.similarity_threshold,
                    max_results: self.config.max_semantic_results,
                    filters: crate::models::embedding::SearchFilters {
                        project_ids: Some(vec![project_id.to_string()]),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                
                semantic_results = self.semantic_search_service.search(&query).await?;
                debug!("Semantic search returned {} results", semantic_results.len());
            }
            _ => {}
        }
        
        match strategy {
            SearchStrategy::TraditionalOnly | SearchStrategy::Hybrid => {
                // Perform traditional search if we have the required parameters
                if let (Some(feature_area), Some(task_type)) = (feature_area, task_type) {
                    traditional_results = self.context_query_service
                        .query_context(project_id, feature_area, task_type, components)
                        .await?;
                    
                    debug!("Traditional search returned {} business rules, {} architectural decisions, {} performance requirements",
                           traditional_results.business_rules.len(),
                           traditional_results.architectural_decisions.len(),
                           traditional_results.performance_requirements.len());
                }
            }
            _ => {}
        }
        
        // Fuse results
        let hybrid_result = self.fuse_results(semantic_results, traditional_results, strategy);
        
        info!("Hybrid search completed: {} total results with combined score {:.3}",
              hybrid_result.total_results, hybrid_result.combined_score);
        
        Ok(hybrid_result)
    }
    
    async fn semantic_search(
        &self,
        query: &VectorSearchQuery,
    ) -> Result<Vec<EnhancedSearchResult>, HybridSearchError> {
        debug!("Performing semantic-only search");
        let results = self.semantic_search_service.search(query).await?;
        Ok(results)
    }
    
    async fn traditional_search(
        &self,
        project_id: &str,
        feature_area: &str,
        task_type: &str,
        components: &[String],
    ) -> Result<ContextQueryResult, HybridSearchError> {
        debug!("Performing traditional-only search");
        let results = self.context_query_service
            .query_context(project_id, feature_area, task_type, components)
            .await?;
        Ok(results)
    }
    
    async fn get_search_suggestions(
        &self,
        partial_query: &str,
        project_id: Option<&str>,
    ) -> Result<Vec<String>, HybridSearchError> {
        debug!("Getting search suggestions for: {}", partial_query);
        
        // Get suggestions from semantic search service
        let semantic_suggestions = self.semantic_search_service
            .suggest_queries(partial_query, project_id.map(|s| s))
            .await?;
        
        // Add hybrid-specific suggestions
        let mut all_suggestions = semantic_suggestions;
        
        // Add intent-based suggestions
        if partial_query.len() >= 3 {
            let intent_suggestions = vec![
                format!("how to implement {}", partial_query),
                format!("best practices for {}", partial_query),
                format!("examples of {}", partial_query),
                format!("architecture for {}", partial_query),
                format!("security considerations for {}", partial_query),
            ];
            
            all_suggestions.extend(intent_suggestions);
        }
        
        // Remove duplicates and limit results
        all_suggestions.sort();
        all_suggestions.dedup();
        all_suggestions.truncate(10);
        
        debug!("Generated {} search suggestions", all_suggestions.len());
        Ok(all_suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::embedding::VectorSearchQuery;
    use crate::services::semantic_search_service::{EnhancedSearchResult, SearchMetadata};
    use crate::models::embedding::{VectorSearchResult, ResultMetadata};
    use std::sync::Arc;

    
    // Mock implementations for testing
    struct MockSemanticSearchService;
    
    #[async_trait]
    impl SemanticSearchService for MockSemanticSearchService {
        async fn index_context(&self, _context: &EnhancedContextItem) -> Result<(), SemanticSearchError> {
            Ok(())
        }
        
        async fn index_contexts_batch(&self, _contexts: &[EnhancedContextItem]) -> Result<(), SemanticSearchError> {
            Ok(())
        }
        
        async fn search(&self, query: &VectorSearchQuery) -> Result<Vec<EnhancedSearchResult>, SemanticSearchError> {
            // Return mock results
            let mock_result = EnhancedSearchResult {
                vector_result: VectorSearchResult {
                    context_id: "test-context-1".to_string(),
                    similarity_score: 0.85,
                    distance: 0.15,
                    rank: 1,
                    metadata: ResultMetadata {
                        content_type: "business_rule".to_string(),
                        content_preview: "Test business rule".to_string(),
                        match_explanation: "High similarity match".to_string(),
                        quality_indicators: vec!["High quality".to_string()],
                    },
                },
                context_item: None,
                relevance_explanation: "Test relevance".to_string(),
                search_metadata: SearchMetadata {
                    query_processing_time_ms: 10,
                    embedding_generation_time_ms: 5,
                    similarity_calculation_time_ms: 3,
                    total_candidates_evaluated: 100,
                    filters_applied: vec!["similarity_threshold".to_string()],
                    ranking_method_used: "cosine_similarity".to_string(),
                },
            };
            
            Ok(vec![mock_result])
        }
        
        async fn find_similar_contexts(&self, _context_id: &str, _max_results: usize) -> Result<Vec<EnhancedSearchResult>, SemanticSearchError> {
            Ok(Vec::new())
        }
        
        async fn suggest_queries(&self, _partial_query: &str, _project_id: Option<&str>) -> Result<Vec<String>, SemanticSearchError> {
            Ok(vec!["test suggestion".to_string()])
        }
        
        async fn update_context_index(&self, _context: &EnhancedContextItem) -> Result<(), SemanticSearchError> {
            Ok(())
        }
        
        async fn remove_from_index(&self, _context_id: &str) -> Result<(), SemanticSearchError> {
            Ok(())
        }
        
        async fn get_index_stats(&self, _project_id: Option<&str>) -> Result<crate::services::semantic_search_service::SearchIndexStats, SemanticSearchError> {
            Ok(crate::services::semantic_search_service::SearchIndexStats {
                total_indexed_items: 0,
                items_by_content_type: HashMap::new(),
                items_by_project: HashMap::new(),
                average_embedding_quality: 0.8,
                index_freshness_score: 0.9,
                last_updated: chrono::Utc::now(),
            })
        }
        
        async fn rebuild_index(&self, _project_id: &str, _contexts: &[EnhancedContextItem]) -> Result<(), SemanticSearchError> {
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
            Ok(ContextQueryResult {
                business_rules: Vec::new(),
                architectural_decisions: Vec::new(),
                performance_requirements: Vec::new(),
                security_policies: Vec::new(),
                project_conventions: Vec::new(),
            })
        }
    }
    
    #[tokio::test]
    async fn test_hybrid_search() {
        let semantic_service = Arc::new(MockSemanticSearchService);
        let context_service = Arc::new(MockContextQueryService);
        let config = HybridSearchConfig::default();
        
        let hybrid_service = HybridSearchServiceImpl::new(
            semantic_service,
            context_service,
            config,
        );
        
        let result = hybrid_service.hybrid_search(
            "test-project",
            "authentication security",
            Some("auth"),
            Some("implementation"),
            &[],
        ).await.unwrap();
        
        assert_eq!(result.search_strategy, SearchStrategy::Hybrid);
        assert!(!result.semantic_results.is_empty());
    }
    
    #[tokio::test]
    async fn test_search_suggestions() {
        let semantic_service = Arc::new(MockSemanticSearchService);
        let context_service = Arc::new(MockContextQueryService);
        let config = HybridSearchConfig::default();
        
        let hybrid_service = HybridSearchServiceImpl::new(
            semantic_service,
            context_service,
            config,
        );
        
        let suggestions = hybrid_service.get_search_suggestions("auth", Some("test-project")).await.unwrap();
        assert!(!suggestions.is_empty());
    }
}