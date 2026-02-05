use crate::services::{
    AdvancedQueryService, AdvancedQueryServiceImpl, AdvancedQueryConfig, AdvancedSearchQuery, 
    AdvancedSearchResult, QuerySuggestion, AdvancedQueryError, AdvancedSearchFilters, 
    QualityRange, RankingPreferences, UserContext, UserPreferences, SuggestionType
};
use crate::models::embedding::{VectorSearchQuery, SearchFilters, VectorSearchResult, ResultMetadata};
use crate::models::enhanced_context::{EnhancedContextItem, ContextType};
use crate::services::hybrid_search_service::{HybridSearchService, HybridSearchResult, HybridSearchError, SearchStrategy};
use crate::services::semantic_search_service::{SemanticSearchService, EnhancedSearchResult, SearchMetadata, SemanticSearchError, SearchIndexStats};
use crate::services::context_query_service::ContextQueryResult;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio;

// Mock implementations for testing
struct MockHybridSearchService {
    mock_results: Vec<EnhancedSearchResult>,
}

impl MockHybridSearchService {
    fn new() -> Self {
        Self {
            mock_results: create_mock_enhanced_results(),
        }
    }
}

#[async_trait]
impl HybridSearchService for MockHybridSearchService {
    async fn hybrid_search(
        &self,
        _project_id: &str,
        query_text: &str,
        _feature_area: Option<&str>,
        _task_type: Option<&str>,
        _components: &[String],
    ) -> Result<HybridSearchResult, HybridSearchError> {
        // Filter results based on query text for more realistic testing
        let filtered_results = if query_text.contains("auth") {
            self.mock_results.iter()
                .filter(|r| r.vector_result.metadata.content_type.contains("security"))
                .cloned()
                .collect()
        } else if query_text.contains("database") {
            self.mock_results.iter()
                .filter(|r| r.vector_result.metadata.content_type.contains("database"))
                .cloned()
                .collect()
        } else {
            self.mock_results.clone()
        };

        Ok(HybridSearchResult {
            semantic_results: filtered_results,
            traditional_results: ContextQueryResult {
                business_rules: Vec::new(),
                architectural_decisions: Vec::new(),
                performance_requirements: Vec::new(),
                security_policies: Vec::new(),
                project_conventions: Vec::new(),
            },
            combined_score: 0.85,
            search_strategy: SearchStrategy::Hybrid,
            total_results: self.mock_results.len(),
        })
    }

    async fn semantic_search(
        &self,
        _query: &VectorSearchQuery,
    ) -> Result<Vec<EnhancedSearchResult>, HybridSearchError> {
        Ok(self.mock_results.clone())
    }

    async fn traditional_search(
        &self,
        _project_id: &str,
        _feature_area: &str,
        _task_type: &str,
        _components: &[String],
    ) -> Result<ContextQueryResult, HybridSearchError> {
        Ok(ContextQueryResult {
            business_rules: Vec::new(),
            architectural_decisions: Vec::new(),
            performance_requirements: Vec::new(),
            security_policies: Vec::new(),
            project_conventions: Vec::new(),
        })
    }

    async fn get_search_suggestions(
        &self,
        partial_query: &str,
        _project_id: Option<&str>,
    ) -> Result<Vec<String>, HybridSearchError> {
        Ok(vec![
            format!("{} implementation", partial_query),
            format!("{} best practices", partial_query),
            format!("{} examples", partial_query),
        ])
    }
}

struct MockSemanticSearchService {
    suggestions: Vec<String>,
}

impl MockSemanticSearchService {
    fn new() -> Self {
        Self {
            suggestions: vec![
                "authentication patterns".to_string(),
                "security best practices".to_string(),
                "database optimization".to_string(),
                "API design patterns".to_string(),
            ],
        }
    }
}

#[async_trait]
impl SemanticSearchService for MockSemanticSearchService {
    async fn index_context(&self, _context: &EnhancedContextItem) -> Result<(), SemanticSearchError> {
        Ok(())
    }

    async fn index_contexts_batch(&self, _contexts: &[EnhancedContextItem]) -> Result<(), SemanticSearchError> {
        Ok(())
    }

    async fn search(&self, _query: &VectorSearchQuery) -> Result<Vec<EnhancedSearchResult>, SemanticSearchError> {
        Ok(create_mock_enhanced_results())
    }

    async fn find_similar_contexts(&self, _context_id: &str, _max_results: usize) -> Result<Vec<EnhancedSearchResult>, SemanticSearchError> {
        Ok(Vec::new())
    }

    async fn suggest_queries(&self, partial_query: &str, _project_id: Option<&str>) -> Result<Vec<String>, SemanticSearchError> {
        Ok(self.suggestions.iter()
            .filter(|s| s.contains(partial_query))
            .cloned()
            .collect())
    }

    async fn update_context_index(&self, _context: &EnhancedContextItem) -> Result<(), SemanticSearchError> {
        Ok(())
    }

    async fn remove_from_index(&self, _context_id: &str) -> Result<(), SemanticSearchError> {
        Ok(())
    }

    async fn get_index_stats(&self, _project_id: Option<&str>) -> Result<SearchIndexStats, SemanticSearchError> {
        Ok(SearchIndexStats {
            total_indexed_items: 100,
            items_by_content_type: HashMap::new(),
            items_by_project: HashMap::new(),
            average_embedding_quality: 0.8,
            index_freshness_score: 0.9,
            last_updated: Utc::now(),
        })
    }

    async fn rebuild_index(&self, _project_id: &str, _contexts: &[EnhancedContextItem]) -> Result<(), SemanticSearchError> {
        Ok(())
    }
}

// Helper functions for creating test data
fn create_mock_enhanced_results() -> Vec<EnhancedSearchResult> {
    vec![
        EnhancedSearchResult {
            vector_result: VectorSearchResult {
                context_id: "auth-context-1".to_string(),
                similarity_score: 0.95,
                distance: 0.05,
                rank: 1,
                metadata: ResultMetadata {
                    content_type: "security_policy".to_string(),
                    content_preview: "Authentication security policy".to_string(),
                    match_explanation: "High similarity match for authentication".to_string(),
                    quality_indicators: vec!["High quality".to_string(), "Recently updated".to_string()],
                },
            },
            context_item: None,
            relevance_explanation: "Strong match for authentication query".to_string(),
            search_metadata: SearchMetadata {
                query_processing_time_ms: 10,
                embedding_generation_time_ms: 5,
                similarity_calculation_time_ms: 3,
                total_candidates_evaluated: 100,
                filters_applied: vec!["similarity_threshold".to_string()],
                ranking_method_used: "cosine_similarity".to_string(),
            },
        },
        EnhancedSearchResult {
            vector_result: VectorSearchResult {
                context_id: "db-context-1".to_string(),
                similarity_score: 0.85,
                distance: 0.15,
                rank: 2,
                metadata: ResultMetadata {
                    content_type: "database_schema".to_string(),
                    content_preview: "User database schema".to_string(),
                    match_explanation: "Good match for database query".to_string(),
                    quality_indicators: vec!["Good quality".to_string()],
                },
            },
            context_item: None,
            relevance_explanation: "Good match for database query".to_string(),
            search_metadata: SearchMetadata {
                query_processing_time_ms: 8,
                embedding_generation_time_ms: 4,
                similarity_calculation_time_ms: 2,
                total_candidates_evaluated: 80,
                filters_applied: vec!["similarity_threshold".to_string()],
                ranking_method_used: "cosine_similarity".to_string(),
            },
        },
        EnhancedSearchResult {
            vector_result: VectorSearchResult {
                context_id: "api-context-1".to_string(),
                similarity_score: 0.75,
                distance: 0.25,
                rank: 3,
                metadata: ResultMetadata {
                    content_type: "api_specification".to_string(),
                    content_preview: "REST API specification".to_string(),
                    match_explanation: "Moderate match for API query".to_string(),
                    quality_indicators: vec!["Medium quality".to_string()],
                },
            },
            context_item: None,
            relevance_explanation: "Moderate match for API query".to_string(),
            search_metadata: SearchMetadata {
                query_processing_time_ms: 12,
                embedding_generation_time_ms: 6,
                similarity_calculation_time_ms: 4,
                total_candidates_evaluated: 60,
                filters_applied: vec!["similarity_threshold".to_string()],
                ranking_method_used: "cosine_similarity".to_string(),
            },
        },
    ]
}

fn create_test_user_context() -> UserContext {
    UserContext {
        user_id: "test-user".to_string(),
        accessible_projects: vec!["project-1".to_string(), "project-2".to_string(), "project-3".to_string()],
        preferences: UserPreferences {
            default_filters: AdvancedSearchFilters::default(),
            preferred_content_types: vec![ContextType::SecurityPolicy, ContextType::ArchitecturalDecision],
            ranking_preferences: RankingPreferences::default(),
            enable_cross_project_by_default: true,
        },
        recent_queries: vec!["authentication".to_string(), "database design".to_string()],
    }
}

// Integration tests
#[tokio::test]
async fn test_advanced_search_basic_functionality() {
    let hybrid_service = Arc::new(MockHybridSearchService::new());
    let semantic_service = Arc::new(MockSemanticSearchService::new());
    let config = AdvancedQueryConfig::default();
    
    let advanced_service = AdvancedQueryServiceImpl::new(hybrid_service, semantic_service, config);
    
    let query = AdvancedSearchQuery {
        query_text: "authentication security".to_string(),
        filters: AdvancedSearchFilters::default(),
        max_results: 10,
        include_suggestions: true,
        enable_cross_project: false,
        ranking_preferences: RankingPreferences::default(),
    };
    
    let user_context = create_test_user_context();
    let result = advanced_service.advanced_search(&query, Some(&user_context)).await.unwrap();
    
    // Verify basic search functionality
    assert!(!result.hybrid_result.semantic_results.is_empty());
    assert!(!result.suggestions.is_empty());
    assert_eq!(result.hybrid_result.search_strategy, SearchStrategy::Hybrid);
    assert!(result.search_metadata.total_search_time_ms > 0);
}

#[tokio::test]
async fn test_query_suggestions() {
    let hybrid_service = Arc::new(MockHybridSearchService::new());
    let semantic_service = Arc::new(MockSemanticSearchService::new());
    let config = AdvancedQueryConfig::default();
    
    let advanced_service = AdvancedQueryServiceImpl::new(hybrid_service, semantic_service, config);
    
    let suggestions = advanced_service.suggest_queries(
        "auth",
        Some("project-1"),
        &["authentication patterns".to_string(), "security guidelines".to_string()],
    ).await.unwrap();
    
    assert!(!suggestions.is_empty());
    
    // Verify suggestion types and content
    let has_similar = suggestions.iter().any(|s| s.suggestion_type == SuggestionType::Similar);
    let has_intent = suggestions.iter().any(|s| s.suggestion_type == SuggestionType::Intent);
    
    assert!(has_similar || has_intent);
    
    // Verify suggestions contain relevant content
    let auth_suggestions: Vec<_> = suggestions.iter()
        .filter(|s| s.suggestion.contains("auth"))
        .collect();
    assert!(!auth_suggestions.is_empty());
}

#[tokio::test]
async fn test_auto_completion() {
    let hybrid_service = Arc::new(MockHybridSearchService::new());
    let semantic_service = Arc::new(MockSemanticSearchService::new());
    let config = AdvancedQueryConfig::default();
    
    let advanced_service = AdvancedQueryServiceImpl::new(hybrid_service, semantic_service, config);
    
    let completions = advanced_service.auto_complete(
        "database",
        8, // cursor position
        Some("project-1"),
    ).await.unwrap();
    
    assert!(!completions.is_empty());
    assert!(completions.len() <= 5); // Should be limited to 5 completions
    
    // Verify all completions are longer than the partial query
    for completion in &completions {
        assert!(completion.len() > "database".len());
        assert!(completion.contains("database"));
    }
}

#[tokio::test]
async fn test_cross_project_search() {
    let hybrid_service = Arc::new(MockHybridSearchService::new());
    let semantic_service = Arc::new(MockSemanticSearchService::new());
    let config = AdvancedQueryConfig::default();
    
    let advanced_service = AdvancedQueryServiceImpl::new(hybrid_service, semantic_service, config);
    
    let accessible_projects = vec!["project-1".to_string(), "project-2".to_string(), "project-3".to_string()];
    let matches = advanced_service.cross_project_search(
        "authentication",
        &accessible_projects,
        5, // max_results_per_project
    ).await.unwrap();
    
    // Should have matches for each project (mock returns results for all projects)
    assert_eq!(matches.len(), accessible_projects.len());
    
    // Verify match structure
    for project_match in &matches {
        assert!(accessible_projects.contains(&project_match.project_id));
        assert!(project_match.match_count > 0);
        assert!(!project_match.top_matches.is_empty());
        assert!(project_match.similarity_score > 0.0);
    }
    
    // Verify results are sorted by similarity score (descending)
    for i in 1..matches.len() {
        assert!(matches[i-1].similarity_score >= matches[i].similarity_score);
    }
}

#[tokio::test]
async fn test_advanced_filters() {
    let hybrid_service = Arc::new(MockHybridSearchService::new());
    let semantic_service = Arc::new(MockSemanticSearchService::new());
    let config = AdvancedQueryConfig::default();
    
    let advanced_service = AdvancedQueryServiceImpl::new(hybrid_service, semantic_service, config);
    
    let mock_results = create_mock_enhanced_results();
    
    // Test quality filter
    let quality_filters = AdvancedSearchFilters {
        quality_range: Some(QualityRange {
            min_score: 0.8,
            max_score: 1.0,
        }),
        ..Default::default()
    };
    
    let (filtered_results, stats) = advanced_service.apply_filters(
        mock_results.clone(),
        &quality_filters,
    ).await.unwrap();
    
    // Should filter out results with similarity < 0.8
    assert!(filtered_results.len() <= mock_results.len());
    assert_eq!(stats.total_candidates, mock_results.len());
    assert!(stats.final_results <= stats.total_candidates);
    
    // Verify all remaining results meet quality threshold
    for result in &filtered_results {
        assert!(result.vector_result.similarity_score >= 0.8);
    }
}

#[tokio::test]
async fn test_content_type_filtering() {
    let hybrid_service = Arc::new(MockHybridSearchService::new());
    let semantic_service = Arc::new(MockSemanticSearchService::new());
    let config = AdvancedQueryConfig::default();
    
    let advanced_service = AdvancedQueryServiceImpl::new(hybrid_service, semantic_service, config);
    
    let mock_results = create_mock_enhanced_results();
    
    // Test content type filter
    let content_type_filters = AdvancedSearchFilters {
        base_filters: SearchFilters {
            content_types: Some(vec!["security_policy".to_string()]),
            ..Default::default()
        },
        ..Default::default()
    };
    
    let (filtered_results, stats) = advanced_service.apply_filters(
        mock_results.clone(),
        &content_type_filters,
    ).await.unwrap();
    
    // Should only include security_policy results
    assert!(filtered_results.len() <= mock_results.len());
    
    // Verify all remaining results are security_policy type
    for result in &filtered_results {
        assert_eq!(result.vector_result.metadata.content_type, "security_policy");
    }
}

#[tokio::test]
async fn test_filter_suggestions() {
    let hybrid_service = Arc::new(MockHybridSearchService::new());
    let semantic_service = Arc::new(MockSemanticSearchService::new());
    let config = AdvancedQueryConfig::default();
    
    let advanced_service = AdvancedQueryServiceImpl::new(hybrid_service, semantic_service, config);
    
    let mock_results = create_mock_enhanced_results();
    
    let filter_suggestions = advanced_service.suggest_filters(
        "authentication",
        &mock_results,
        Some("project-1"),
    ).await.unwrap();
    
    assert!(!filter_suggestions.is_empty());
    
    // Should suggest content type filters for types that appear multiple times
    let content_type_suggestions: Vec<_> = filter_suggestions.iter()
        .filter(|s| s.filter_type == "content_type")
        .collect();
    
    // Should suggest quality and date filters
    let quality_suggestions: Vec<_> = filter_suggestions.iter()
        .filter(|s| s.filter_type == "quality")
        .collect();
    
    let date_suggestions: Vec<_> = filter_suggestions.iter()
        .filter(|s| s.filter_type == "date")
        .collect();
    
    assert!(!quality_suggestions.is_empty());
    assert!(!date_suggestions.is_empty());
    
    // Verify suggestion structure
    for suggestion in &filter_suggestions {
        assert!(!suggestion.filter_type.is_empty());
        assert!(!suggestion.filter_value.is_empty());
        assert!(!suggestion.description.is_empty());
        assert!(suggestion.confidence > 0.0 && suggestion.confidence <= 1.0);
        assert!(suggestion.estimated_reduction >= 0.0 && suggestion.estimated_reduction <= 1.0);
    }
}

#[tokio::test]
async fn test_filter_validation() {
    let hybrid_service = Arc::new(MockHybridSearchService::new());
    let semantic_service = Arc::new(MockSemanticSearchService::new());
    let config = AdvancedQueryConfig::default();
    
    let advanced_service = AdvancedQueryServiceImpl::new(hybrid_service, semantic_service, config);
    
    // Test invalid quality range
    let invalid_quality_query = AdvancedSearchQuery {
        query_text: "test".to_string(),
        filters: AdvancedSearchFilters {
            quality_range: Some(QualityRange {
                min_score: 1.5, // Invalid: > 1.0
                max_score: 2.0, // Invalid: > 1.0
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    
    let result = advanced_service.advanced_search(&invalid_quality_query, None).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AdvancedQueryError::FilterValidationError { message } => {
            assert!(message.contains("Quality score must be between 0.0 and 1.0"));
        }
        _ => panic!("Expected FilterValidationError"),
    }
    
    // Test invalid quality range order
    let invalid_order_query = AdvancedSearchQuery {
        query_text: "test".to_string(),
        filters: AdvancedSearchFilters {
            quality_range: Some(QualityRange {
                min_score: 0.9,
                max_score: 0.5, // Invalid: min > max
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    
    let result = advanced_service.advanced_search(&invalid_order_query, None).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AdvancedQueryError::FilterValidationError { message } => {
            assert!(message.contains("Minimum quality score cannot be greater than maximum"));
        }
        _ => panic!("Expected FilterValidationError"),
    }
}

#[tokio::test]
async fn test_advanced_search_with_cross_project() {
    let hybrid_service = Arc::new(MockHybridSearchService::new());
    let semantic_service = Arc::new(MockSemanticSearchService::new());
    let config = AdvancedQueryConfig::default();
    
    let advanced_service = AdvancedQueryServiceImpl::new(hybrid_service, semantic_service, config);
    
    let query = AdvancedSearchQuery {
        query_text: "authentication security".to_string(),
        filters: AdvancedSearchFilters::default(),
        max_results: 10,
        include_suggestions: true,
        enable_cross_project: true, // Enable cross-project search
        ranking_preferences: RankingPreferences::default(),
    };
    
    let user_context = create_test_user_context();
    let result = advanced_service.advanced_search(&query, Some(&user_context)).await.unwrap();
    
    // Should have cross-project matches
    assert!(!result.cross_project_matches.is_empty());
    assert_eq!(result.cross_project_matches.len(), user_context.accessible_projects.len());
    
    // Verify search metadata includes cross-project timing
    assert!(result.search_metadata.total_search_time_ms > 0);
}

#[tokio::test]
async fn test_disabled_features() {
    let hybrid_service = Arc::new(MockHybridSearchService::new());
    let semantic_service = Arc::new(MockSemanticSearchService::new());
    
    // Create config with features disabled
    let config = AdvancedQueryConfig {
        enable_query_suggestions: false,
        enable_cross_project_search: false,
        enable_auto_completion: false,
        ..Default::default()
    };
    
    let advanced_service = AdvancedQueryServiceImpl::new(hybrid_service, semantic_service, config);
    
    // Test that suggestions are empty when disabled
    let suggestions = advanced_service.suggest_queries(
        "auth",
        Some("project-1"),
        &[],
    ).await.unwrap();
    assert!(suggestions.is_empty());
    
    // Test that auto-completion is empty when disabled
    let completions = advanced_service.auto_complete(
        "database",
        8,
        Some("project-1"),
    ).await.unwrap();
    assert!(completions.is_empty());
    
    // Test that cross-project search is empty when disabled
    let matches = advanced_service.cross_project_search(
        "authentication",
        &["project-1".to_string(), "project-2".to_string()],
        5,
    ).await.unwrap();
    assert!(matches.is_empty());
}

#[tokio::test]
async fn test_search_performance_metadata() {
    let hybrid_service = Arc::new(MockHybridSearchService::new());
    let semantic_service = Arc::new(MockSemanticSearchService::new());
    let config = AdvancedQueryConfig::default();
    
    let advanced_service = AdvancedQueryServiceImpl::new(hybrid_service, semantic_service, config);
    
    let query = AdvancedSearchQuery {
        query_text: "performance test query".to_string(),
        filters: AdvancedSearchFilters::default(),
        max_results: 10,
        include_suggestions: true,
        enable_cross_project: true,
        ranking_preferences: RankingPreferences::default(),
    };
    
    let user_context = create_test_user_context();
    let result = advanced_service.advanced_search(&query, Some(&user_context)).await.unwrap();
    
    // Verify performance metadata is populated
    let metadata = &result.search_metadata;
    assert!(metadata.total_search_time_ms > 0);
    assert!(metadata.query_processing_time_ms > 0);
    assert!(!metadata.filters_applied.is_empty());
    assert!(!metadata.search_strategy.is_empty());
    
    // Verify filter statistics
    let stats = &result.filter_stats;
    assert!(stats.total_candidates > 0);
    assert_eq!(stats.final_results, result.hybrid_result.semantic_results.len());
}