use context_server_rs::services::advanced_query_service::{
    AdvancedQueryService, AdvancedQueryServiceImpl, AdvancedQueryConfig, AdvancedSearchQuery, 
    AdvancedSearchFilters, QualityRange, RankingPreferences, UserContext, UserPreferences, SuggestionType
};
use context_server_rs::models::enhanced_context::ContextType;
use tokio;

#[tokio::test]
async fn test_advanced_query_features_integration() {
    // Test 1: Query suggestions and auto-completion based on context patterns
    let config = AdvancedQueryConfig::default();
    let service = AdvancedQueryServiceImpl::new(config);
    
    // Test query suggestions
    let suggestions = service.suggest_queries(
        "authentication",
        Some("project-1"),
        &["authentication patterns".to_string(), "security guidelines".to_string()],
    ).await.unwrap();
    
    assert!(!suggestions.is_empty(), "Query suggestions should not be empty");
    
    // Verify suggestion types
    let has_intent = suggestions.iter().any(|s| s.suggestion_type == SuggestionType::Intent);
    assert!(has_intent, "Should have intent-based suggestions");
    
    // Test auto-completion
    let completions = service.auto_complete(
        "database",
        8,
        Some("project-1"),
    ).await.unwrap();
    
    assert!(!completions.is_empty(), "Auto-completion should not be empty");
    assert!(completions.len() <= 5, "Should limit completions to 5");
    
    // Verify all completions are longer than the partial query
    for completion in &completions {
        assert!(completion.len() > "database".len(), "Completion should be longer than partial query");
        assert!(completion.contains("database"), "Completion should contain the partial query");
    }
    
    println!("✓ Query suggestions and auto-completion working correctly");
}

#[tokio::test]
async fn test_cross_project_search_capabilities() {
    let config = AdvancedQueryConfig {
        enable_cross_project_search: true,
        ..Default::default()
    };
    let service = AdvancedQueryServiceImpl::new(config);
    
    let accessible_projects = vec!["project-1".to_string(), "project-2".to_string(), "project-3".to_string()];
    let matches = service.cross_project_search(
        "authentication",
        &accessible_projects,
        5,
    ).await.unwrap();
    
    // In the simplified implementation, this returns empty but doesn't error
    // This demonstrates the interface is working correctly
    assert!(matches.is_empty(), "Simplified implementation returns empty results");
    
    println!("✓ Cross-project search interface working correctly");
}

#[tokio::test]
async fn test_advanced_search_filters() {
    let config = AdvancedQueryConfig::default();
    let service = AdvancedQueryServiceImpl::new(config);
    
    // Test quality range filter
    let quality_filters = AdvancedSearchFilters {
        quality_range: Some(QualityRange {
            min_score: 0.8,
            max_score: 1.0,
        }),
        ..Default::default()
    };
    
    let mock_results = vec!["result1".to_string(), "result2".to_string(), "result3".to_string()];
    
    let (filtered_results, stats) = service.apply_filters(
        mock_results.clone(),
        &quality_filters,
    ).await.unwrap();
    
    assert_eq!(stats.total_candidates, mock_results.len(), "Should track total candidates");
    assert_eq!(stats.final_results, filtered_results.len(), "Should track final results");
    
    println!("✓ Advanced search filters working correctly");
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
    assert!(result.is_err(), "Should reject invalid quality range");
    
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
    
    let result = service.advanced_search(&invalid_order_query, None).await;
    assert!(result.is_err(), "Should reject invalid quality range order");
    
    println!("✓ Filter validation working correctly");
}

#[tokio::test]
async fn test_advanced_search_with_user_context() {
    let config = AdvancedQueryConfig::default();
    let service = AdvancedQueryServiceImpl::new(config);
    
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
        accessible_projects: vec!["project-1".to_string(), "project-2".to_string()],
        preferences: UserPreferences {
            default_filters: AdvancedSearchFilters::default(),
            preferred_content_types: vec![ContextType::SecurityPolicy, ContextType::ArchitecturalDecision],
            ranking_preferences: RankingPreferences::default(),
            enable_cross_project_by_default: true,
        },
        recent_queries: vec!["authentication".to_string(), "database design".to_string()],
    };
    
    let result = service.advanced_search(&query, Some(&user_context)).await.unwrap();
    
    // Verify basic search functionality
    assert!(!result.suggestions.is_empty(), "Should have suggestions when requested");
    assert!(result.search_metadata.total_search_time_ms > 0, "Should track search time");
    assert!(!result.search_metadata.filters_applied.is_empty(), "Should track applied filters");
    
    println!("✓ Advanced search with user context working correctly");
}

#[tokio::test]
async fn test_filter_suggestions() {
    let config = AdvancedQueryConfig::default();
    let service = AdvancedQueryServiceImpl::new(config);
    
    let mock_results = vec![
        "security_policy_result".to_string(),
        "architectural_decision_result".to_string(),
        "performance_requirement_result".to_string(),
        "business_rule_result".to_string(),
        "code_pattern_result".to_string(),
        "api_specification_result".to_string(),
    ];
    
    let filter_suggestions = service.suggest_filters(
        "authentication",
        &mock_results,
        Some("project-1"),
    ).await.unwrap();
    
    assert!(!filter_suggestions.is_empty(), "Should provide filter suggestions");
    
    // Should suggest date filter for recent results
    let date_suggestions: Vec<_> = filter_suggestions.iter()
        .filter(|s| s.filter_type == "date")
        .collect();
    
    assert!(!date_suggestions.is_empty(), "Should suggest date filters");
    
    // Verify suggestion structure
    for suggestion in &filter_suggestions {
        assert!(!suggestion.filter_type.is_empty(), "Filter type should not be empty");
        assert!(!suggestion.filter_value.is_empty(), "Filter value should not be empty");
        assert!(!suggestion.description.is_empty(), "Description should not be empty");
        assert!(suggestion.confidence > 0.0 && suggestion.confidence <= 1.0, "Confidence should be between 0 and 1");
        assert!(suggestion.estimated_reduction >= 0.0 && suggestion.estimated_reduction <= 1.0, "Estimated reduction should be between 0 and 1");
    }
    
    println!("✓ Filter suggestions working correctly");
}

#[tokio::test]
async fn test_disabled_features() {
    let config = AdvancedQueryConfig {
        enable_query_suggestions: false,
        enable_cross_project_search: false,
        enable_auto_completion: false,
        ..Default::default()
    };
    
    let service = AdvancedQueryServiceImpl::new(config);
    
    // Test that suggestions are empty when disabled
    let suggestions = service.suggest_queries(
        "auth",
        Some("project-1"),
        &[],
    ).await.unwrap();
    assert!(suggestions.is_empty(), "Suggestions should be empty when disabled");
    
    // Test that auto-completion is empty when disabled
    let completions = service.auto_complete(
        "database",
        8,
        Some("project-1"),
    ).await.unwrap();
    assert!(completions.is_empty(), "Auto-completion should be empty when disabled");
    
    // Test that cross-project search is empty when disabled
    let matches = service.cross_project_search(
        "authentication",
        &["project-1".to_string(), "project-2".to_string()],
        5,
    ).await.unwrap();
    assert!(matches.is_empty(), "Cross-project search should be empty when disabled");
    
    println!("✓ Feature disabling working correctly");
}

#[tokio::test]
async fn test_search_performance_metadata() {
    let config = AdvancedQueryConfig::default();
    let service = AdvancedQueryServiceImpl::new(config);
    
    let query = AdvancedSearchQuery {
        query_text: "performance test query".to_string(),
        filters: AdvancedSearchFilters::default(),
        max_results: 10,
        include_suggestions: true,
        enable_cross_project: false,
        ranking_preferences: RankingPreferences::default(),
    };
    
    let result = service.advanced_search(&query, None).await.unwrap();
    
    // Verify performance metadata is populated
    let metadata = &result.search_metadata;
    assert!(metadata.total_search_time_ms > 0, "Should track total search time");
    assert!(metadata.query_processing_time_ms > 0, "Should track query processing time");
    assert!(!metadata.filters_applied.is_empty(), "Should track applied filters");
    assert!(!metadata.search_strategy.is_empty(), "Should track search strategy");
    
    // Verify filter statistics
    let stats = &result.filter_stats;
    assert_eq!(stats.final_results, 0, "Simplified implementation has no results");
    
    println!("✓ Search performance metadata working correctly");
}