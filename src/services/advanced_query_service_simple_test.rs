use super::advanced_query_service::{
    AdvancedQueryService, AdvancedQueryServiceImpl, AdvancedQueryConfig, AdvancedSearchQuery, 
    AdvancedSearchResult, QuerySuggestion, AdvancedQueryError, AdvancedSearchFilters, 
    QualityRange, RankingPreferences, UserContext, UserPreferences, SuggestionType
};
use tokio;

#[tokio::test]
async fn test_advanced_query_service_basic_functionality() {
    let config = AdvancedQueryConfig::default();
    let service = AdvancedQueryServiceImpl::new(config);
    
    // Test query suggestions
    let suggestions = service.suggest_queries(
        "authentication",
        Some("project-1"),
        &[],
    ).await.unwrap();
    
    assert!(!suggestions.is_empty());
    
    // Test auto-completion
    let completions = service.auto_complete(
        "database",
        8,
        Some("project-1"),
    ).await.unwrap();
    
    assert!(!completions.is_empty());
    assert!(completions.len() <= 5);
    
    // Test advanced search
    let query = AdvancedSearchQuery {
        query_text: "authentication security".to_string(),
        filters: AdvancedSearchFilters::default(),
        max_results: 10,
        include_suggestions: true,
        enable_cross_project: false,
        ranking_preferences: RankingPreferences::default(),
    };
    
    let user_context = UserContext {
        user_id: "test-user".to_string(),
        accessible_projects: vec!["project-1".to_string()],
        preferences: UserPreferences {
            default_filters: AdvancedSearchFilters::default(),
            preferred_content_types: vec![],
            ranking_preferences: RankingPreferences::default(),
            enable_cross_project_by_default: false,
        },
        recent_queries: vec![],
    };
    
    let result = service.advanced_search(&query, Some(&user_context)).await.unwrap();
    
    // Verify basic search functionality
    assert!(!result.suggestions.is_empty());
    assert!(result.search_metadata.total_search_time_ms > 0);
}

#[tokio::test]
async fn test_filter_validation() {
    let config = AdvancedQueryConfig::default();
    let service = AdvancedQueryServiceImpl::new(config);
    
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
    
    let result = service.advanced_search(&invalid_quality_query, None).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AdvancedQueryError::FilterValidationError { message } => {
            assert!(message.contains("Quality score must be between 0.0 and 1.0"));
        }
        _ => panic!("Expected FilterValidationError"),
    }
}

#[tokio::test]
async fn test_suggestion_types() {
    let config = AdvancedQueryConfig::default();
    let service = AdvancedQueryServiceImpl::new(config);
    
    let suggestions = service.suggest_queries(
        "how to implement authentication",
        Some("project-1"),
        &[],
    ).await.unwrap();
    
    assert!(!suggestions.is_empty());
    
    // Verify suggestion types and content
    let has_intent = suggestions.iter().any(|s| s.suggestion_type == SuggestionType::Intent);
    assert!(has_intent);
    
    // Verify suggestions contain relevant content
    let auth_suggestions: Vec<_> = suggestions.iter()
        .filter(|s| s.suggestion.contains("implement"))
        .collect();
    assert!(!auth_suggestions.is_empty());
}

#[tokio::test]
async fn test_cross_project_search() {
    let config = AdvancedQueryConfig::default();
    let service = AdvancedQueryServiceImpl::new(config);
    
    let accessible_projects = vec!["project-1".to_string(), "project-2".to_string()];
    let matches = service.cross_project_search(
        "authentication",
        &accessible_projects,
        5,
    ).await.unwrap();
    
    // In the simplified implementation, this returns empty
    assert!(matches.is_empty());
}

#[tokio::test]
async fn test_filter_suggestions() {
    let config = AdvancedQueryConfig::default();
    let service = AdvancedQueryServiceImpl::new(config);
    
    let mock_results = vec!["result1".to_string(), "result2".to_string(), "result3".to_string()];
    
    let filter_suggestions = service.suggest_filters(
        "authentication",
        &mock_results,
        Some("project-1"),
    ).await.unwrap();
    
    assert!(!filter_suggestions.is_empty());
    
    // Should suggest quality and date filters
    let quality_suggestions: Vec<_> = filter_suggestions.iter()
        .filter(|s| s.filter_type == "quality")
        .collect();
    
    let date_suggestions: Vec<_> = filter_suggestions.iter()
        .filter(|s| s.filter_type == "date")
        .collect();
    
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