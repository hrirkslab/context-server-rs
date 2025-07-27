use crate::models::embedding::{
    ContextEmbedding, EmbeddingConfig, VectorSearchQuery, VectorSearchResult,
};
use crate::models::enhanced_context::{EnhancedContextItem, ContextType};
use crate::repositories::embedding_repository::{EmbeddingRepository, EmbeddingRepositoryError};
use crate::services::embedding_service::{EmbeddingService, EmbeddingError};
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Error types for semantic search operations
#[derive(Debug, thiserror::Error)]
pub enum SemanticSearchError {
    #[error("Embedding service error: {source}")]
    EmbeddingServiceError { source: EmbeddingError },
    
    #[error("Repository error: {source}")]
    RepositoryError { source: EmbeddingRepositoryError },
    
    #[error("Search configuration error: {message}")]
    ConfigurationError { message: String },
    
    #[error("Index not ready: {message}")]
    IndexNotReady { message: String },
    
    #[error("Query processing error: {message}")]
    QueryProcessingError { message: String },
}

impl From<EmbeddingError> for SemanticSearchError {
    fn from(error: EmbeddingError) -> Self {
        SemanticSearchError::EmbeddingServiceError { source: error }
    }
}

impl From<EmbeddingRepositoryError> for SemanticSearchError {
    fn from(error: EmbeddingRepositoryError) -> Self {
        SemanticSearchError::RepositoryError { source: error }
    }
}

/// Configuration for semantic search
#[derive(Debug, Clone)]
pub struct SemanticSearchConfig {
    pub embedding_config: EmbeddingConfig,
    pub default_similarity_threshold: f32,
    pub max_results_per_query: usize,
    pub enable_query_expansion: bool,
    pub enable_result_reranking: bool,
    pub cache_query_embeddings: bool,
    pub enable_intent_detection: bool,
    pub recency_weight: f32,
    pub usage_weight: f32,
    pub quality_weight: f32,
}

impl Default for SemanticSearchConfig {
    fn default() -> Self {
        Self {
            embedding_config: EmbeddingConfig::default(),
            default_similarity_threshold: 0.7,
            max_results_per_query: 20,
            enable_query_expansion: false,
            enable_result_reranking: true,
            cache_query_embeddings: true,
            enable_intent_detection: true,
            recency_weight: 0.2,
            usage_weight: 0.15,
            quality_weight: 0.15,
        }
    }
}

/// Enhanced search result with context information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnhancedSearchResult {
    pub vector_result: VectorSearchResult,
    pub context_item: Option<EnhancedContextItem>,
    pub relevance_explanation: String,
    pub search_metadata: SearchMetadata,
}

/// Metadata about the search process
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchMetadata {
    pub query_processing_time_ms: u64,
    pub embedding_generation_time_ms: u64,
    pub similarity_calculation_time_ms: u64,
    pub total_candidates_evaluated: usize,
    pub filters_applied: Vec<String>,
    pub ranking_method_used: String,
}

/// Statistics about search index
#[derive(Debug, Clone)]
pub struct SearchIndexStats {
    pub total_indexed_items: usize,
    pub items_by_content_type: HashMap<String, usize>,
    pub items_by_project: HashMap<String, usize>,
    pub average_embedding_quality: f32,
    pub index_freshness_score: f32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Query intent detected from natural language processing
#[derive(Debug, Clone, PartialEq)]
pub enum QueryIntent {
    FindImplementation,
    FindBestPractices,
    FindExamples,
    FindArchitecture,
    FindSecurity,
    FindPerformance,
    FindDocumentation,
    FindSimilar,
    General,
}

impl QueryIntent {
    pub fn as_str(&self) -> &str {
        match self {
            QueryIntent::FindImplementation => "find_implementation",
            QueryIntent::FindBestPractices => "find_best_practices",
            QueryIntent::FindExamples => "find_examples",
            QueryIntent::FindArchitecture => "find_architecture",
            QueryIntent::FindSecurity => "find_security",
            QueryIntent::FindPerformance => "find_performance",
            QueryIntent::FindDocumentation => "find_documentation",
            QueryIntent::FindSimilar => "find_similar",
            QueryIntent::General => "general",
        }
    }
}

/// Processed query with intent and expanded terms
#[derive(Debug, Clone)]
pub struct ProcessedQuery {
    pub original_query: String,
    pub processed_query: String,
    pub intent: QueryIntent,
    pub key_terms: Vec<String>,
    pub expanded_terms: Vec<String>,
    pub content_type_hints: Vec<ContextType>,
    pub confidence: f32,
}

/// Trait for semantic search operations
#[async_trait]
pub trait SemanticSearchService: Send + Sync {
    /// Index a single context item for search
    async fn index_context(&self, context: &EnhancedContextItem) -> Result<(), SemanticSearchError>;
    
    /// Index multiple context items in batch
    async fn index_contexts_batch(&self, contexts: &[EnhancedContextItem]) -> Result<(), SemanticSearchError>;
    
    /// Perform semantic search
    async fn search(&self, query: &VectorSearchQuery) -> Result<Vec<EnhancedSearchResult>, SemanticSearchError>;
    
    /// Find similar contexts to a given context item
    async fn find_similar_contexts(&self, context_id: &str, max_results: usize) -> Result<Vec<EnhancedSearchResult>, SemanticSearchError>;
    
    /// Generate query suggestions based on partial input
    async fn suggest_queries(&self, partial_query: &str, project_id: Option<&str>) -> Result<Vec<String>, SemanticSearchError>;
    
    /// Update index for a context item
    async fn update_context_index(&self, context: &EnhancedContextItem) -> Result<(), SemanticSearchError>;
    
    /// Remove context from index
    async fn remove_from_index(&self, context_id: &str) -> Result<(), SemanticSearchError>;
    
    /// Get search index statistics
    async fn get_index_stats(&self, project_id: Option<&str>) -> Result<SearchIndexStats, SemanticSearchError>;
    
    /// Rebuild search index for a project
    async fn rebuild_index(&self, project_id: &str, contexts: &[EnhancedContextItem]) -> Result<(), SemanticSearchError>;
}

/// Implementation of SemanticSearchService
pub struct SemanticSearchServiceImpl {
    embedding_service: Arc<dyn EmbeddingService>,
    embedding_repository: Arc<dyn EmbeddingRepository>,
    config: SemanticSearchConfig,
    query_cache: Arc<tokio::sync::Mutex<HashMap<String, Vec<f32>>>>,
}

impl SemanticSearchServiceImpl {
    pub fn new(
        embedding_service: Arc<dyn EmbeddingService>,
        embedding_repository: Arc<dyn EmbeddingRepository>,
        config: SemanticSearchConfig,
    ) -> Self {
        Self {
            embedding_service,
            embedding_repository,
            config,
            query_cache: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
    
    /// Process natural language query with intent detection
    fn process_query(&self, query_text: &str) -> ProcessedQuery {
        let original_query = query_text.to_string();
        let mut processed_query = query_text.to_lowercase().trim().to_string();
        
        // Detect intent from query patterns
        let intent = self.detect_query_intent(&processed_query);
        
        // Extract key terms
        let key_terms = self.extract_key_terms(&processed_query);
        
        // Expand query terms based on intent
        let expanded_terms = if self.config.enable_query_expansion {
            self.expand_query_terms(&key_terms, &intent)
        } else {
            Vec::new()
        };
        
        // Suggest content types based on intent
        let content_type_hints = self.suggest_content_types(&intent);
        
        // Calculate confidence based on intent detection clarity
        let confidence = self.calculate_intent_confidence(&processed_query, &intent);
        
        // Enhance processed query with expanded terms
        if !expanded_terms.is_empty() {
            processed_query = format!("{} {}", processed_query, expanded_terms.join(" "));
        }
        
        ProcessedQuery {
            original_query,
            processed_query,
            intent,
            key_terms,
            expanded_terms,
            content_type_hints,
            confidence,
        }
    }
    
    /// Detect query intent from natural language patterns
    fn detect_query_intent(&self, query: &str) -> QueryIntent {
        // Define intent patterns
        let implementation_patterns = vec![
            Regex::new(r"\b(how to implement|implementation|code|develop|build)\b").unwrap(),
            Regex::new(r"\b(create|make|write|implement)\b.*\b(function|class|method|service)\b").unwrap(),
        ];
        
        let best_practices_patterns = vec![
            Regex::new(r"\b(best practice|guideline|standard|convention|pattern)\b").unwrap(),
            Regex::new(r"\b(should|recommended|proper way|correct way)\b").unwrap(),
        ];
        
        let examples_patterns = vec![
            Regex::new(r"\b(example|sample|demo|tutorial|how to)\b").unwrap(),
            Regex::new(r"\b(show me|give me an example)\b").unwrap(),
        ];
        
        let architecture_patterns = vec![
            Regex::new(r"\b(architecture|design|structure|component|module)\b").unwrap(),
            Regex::new(r"\b(system design|architectural decision)\b").unwrap(),
        ];
        
        let security_patterns = vec![
            Regex::new(r"\b(security|authentication|authorization|encryption|secure)\b").unwrap(),
            Regex::new(r"\b(vulnerability|threat|attack|protection)\b").unwrap(),
        ];
        
        let performance_patterns = vec![
            Regex::new(r"\b(performance|optimization|speed|fast|slow|latency)\b").unwrap(),
            Regex::new(r"\b(benchmark|profiling|memory|cpu)\b").unwrap(),
        ];
        
        let documentation_patterns = vec![
            Regex::new(r"\b(documentation|doc|readme|guide|manual)\b").unwrap(),
            Regex::new(r"\b(explain|describe|what is|definition)\b").unwrap(),
        ];
        
        let similar_patterns = vec![
            Regex::new(r"\b(similar|like|related|comparable)\b").unwrap(),
            Regex::new(r"\b(find similar|show related)\b").unwrap(),
        ];
        
        // Check patterns in order of specificity
        if implementation_patterns.iter().any(|p| p.is_match(query)) {
            QueryIntent::FindImplementation
        } else if best_practices_patterns.iter().any(|p| p.is_match(query)) {
            QueryIntent::FindBestPractices
        } else if examples_patterns.iter().any(|p| p.is_match(query)) {
            QueryIntent::FindExamples
        } else if architecture_patterns.iter().any(|p| p.is_match(query)) {
            QueryIntent::FindArchitecture
        } else if security_patterns.iter().any(|p| p.is_match(query)) {
            QueryIntent::FindSecurity
        } else if performance_patterns.iter().any(|p| p.is_match(query)) {
            QueryIntent::FindPerformance
        } else if documentation_patterns.iter().any(|p| p.is_match(query)) {
            QueryIntent::FindDocumentation
        } else if similar_patterns.iter().any(|p| p.is_match(query)) {
            QueryIntent::FindSimilar
        } else {
            QueryIntent::General
        }
    }
    
    /// Extract key terms from query
    fn extract_key_terms(&self, query: &str) -> Vec<String> {
        // Simple term extraction - remove stop words and extract meaningful terms
        let stop_words = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "up", "about", "into", "through", "during", "before", "after", "above",
            "below", "between", "among", "is", "are", "was", "were", "be", "been", "being", "have",
            "has", "had", "do", "does", "did", "will", "would", "could", "should", "may", "might",
            "must", "can", "this", "that", "these", "those", "i", "you", "he", "she", "it", "we",
            "they", "me", "him", "her", "us", "them", "my", "your", "his", "her", "its", "our",
            "their", "what", "when", "where", "why", "how", "which", "who", "whom"
        ];
        
        query.split_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .filter(|word| !word.is_empty() && !stop_words.contains(&word.as_str()) && word.len() > 2)
            .collect()
    }
    
    /// Expand query terms based on intent
    fn expand_query_terms(&self, key_terms: &[String], intent: &QueryIntent) -> Vec<String> {
        let mut expanded = Vec::new();
        
        match intent {
            QueryIntent::FindImplementation => {
                expanded.extend(vec!["code".to_string(), "function".to_string(), "method".to_string()]);
            },
            QueryIntent::FindBestPractices => {
                expanded.extend(vec!["pattern".to_string(), "guideline".to_string(), "standard".to_string()]);
            },
            QueryIntent::FindExamples => {
                expanded.extend(vec!["sample".to_string(), "demo".to_string(), "tutorial".to_string()]);
            },
            QueryIntent::FindArchitecture => {
                expanded.extend(vec!["design".to_string(), "structure".to_string(), "component".to_string()]);
            },
            QueryIntent::FindSecurity => {
                expanded.extend(vec!["secure".to_string(), "protection".to_string(), "auth".to_string()]);
            },
            QueryIntent::FindPerformance => {
                expanded.extend(vec!["optimization".to_string(), "speed".to_string(), "efficient".to_string()]);
            },
            QueryIntent::FindDocumentation => {
                expanded.extend(vec!["guide".to_string(), "manual".to_string(), "explanation".to_string()]);
            },
            _ => {}
        }
        
        // Add domain-specific expansions for key terms
        for term in key_terms {
            match term.as_str() {
                "auth" | "authentication" => expanded.push("login".to_string()),
                "db" | "database" => expanded.push("storage".to_string()),
                "api" => expanded.push("endpoint".to_string()),
                "test" => expanded.push("testing".to_string()),
                _ => {}
            }
        }
        
        expanded
    }
    
    /// Suggest content types based on intent
    fn suggest_content_types(&self, intent: &QueryIntent) -> Vec<ContextType> {
        match intent {
            QueryIntent::FindImplementation => vec![
                ContextType::CodePattern,
                ContextType::ArchitecturalDecision,
                ContextType::FeatureContext,
            ],
            QueryIntent::FindBestPractices => vec![
                ContextType::ProjectConvention,
                ContextType::CodePattern,
                ContextType::ArchitecturalDecision,
            ],
            QueryIntent::FindExamples => vec![
                ContextType::CodePattern,
                ContextType::TestCase,
                ContextType::Documentation,
            ],
            QueryIntent::FindArchitecture => vec![
                ContextType::ArchitecturalDecision,
                ContextType::ApiSpecification,
                ContextType::DatabaseSchema,
            ],
            QueryIntent::FindSecurity => vec![
                ContextType::SecurityPolicy,
                ContextType::ArchitecturalDecision,
                ContextType::BusinessRule,
            ],
            QueryIntent::FindPerformance => vec![
                ContextType::PerformanceRequirement,
                ContextType::ArchitecturalDecision,
                ContextType::CodePattern,
            ],
            QueryIntent::FindDocumentation => vec![
                ContextType::Documentation,
                ContextType::FeatureContext,
                ContextType::BusinessRule,
            ],
            _ => vec![],
        }
    }
    
    /// Calculate confidence in intent detection
    fn calculate_intent_confidence(&self, query: &str, intent: &QueryIntent) -> f32 {
        // Simple confidence calculation based on pattern matches
        let word_count = query.split_whitespace().count() as f32;
        let base_confidence = match intent {
            QueryIntent::General => 0.5, // Low confidence for general queries
            _ => 0.8, // Higher confidence for specific intents
        };
        
        // Adjust confidence based on query length and specificity
        let length_factor = (word_count / 10.0).min(1.0); // Longer queries are more specific
        (base_confidence * (0.5 + 0.5 * length_factor)).min(1.0)
    }
    
    /// Extract searchable text from context content
    fn extract_searchable_text(&self, context: &EnhancedContextItem) -> String {
        let mut text_parts = Vec::new();
        
        // Add title and description
        text_parts.push(context.content.title.clone());
        text_parts.push(context.content.description.clone());
        
        // Add semantic tags
        for tag in &context.semantic_tags {
            text_parts.push(tag.tag.clone());
        }
        
        // Add metadata tags
        text_parts.extend(context.metadata.tags.clone());
        
        // Extract relevant data from the JSON content
        if let Some(data_str) = context.content.data.as_str() {
            text_parts.push(data_str.to_string());
        } else if let Some(obj) = context.content.data.as_object() {
            for (key, value) in obj {
                text_parts.push(format!("{}: {}", key, value));
            }
        }
        
        text_parts.join(" ")
    }
    
    /// Generate embedding for context with caching
    async fn generate_context_embedding(&self, context: &EnhancedContextItem) -> Result<ContextEmbedding, SemanticSearchError> {
        let searchable_text = self.extract_searchable_text(context);
        let content_type = context.content.content_type.as_str();
        
        let mut embedding = self.embedding_service
            .generate_embedding(&searchable_text, content_type)
            .await?;
        
        embedding.context_id = context.id.clone();
        
        Ok(embedding)
    }
    
    /// Get or generate query embedding with caching
    async fn get_query_embedding(&self, query_text: &str) -> Result<Vec<f32>, SemanticSearchError> {
        if self.config.cache_query_embeddings {
            let cache_key = format!("{:x}", md5::compute(query_text.as_bytes()));
            
            // Check cache first
            {
                let cache = self.query_cache.lock().await;
                if let Some(cached_embedding) = cache.get(&cache_key) {
                    debug!("Using cached query embedding");
                    return Ok(cached_embedding.clone());
                }
            }
            
            // Generate new embedding
            let embedding = self.embedding_service
                .generate_embedding(query_text, "query")
                .await?;
            
            // Cache the result
            {
                let mut cache = self.query_cache.lock().await;
                cache.insert(cache_key, embedding.embedding_vector.clone());
            }
            
            Ok(embedding.embedding_vector)
        } else {
            let embedding = self.embedding_service
                .generate_embedding(query_text, "query")
                .await?;
            Ok(embedding.embedding_vector)
        }
    }
    
    /// Apply result reranking if enabled
    fn rerank_results(&self, mut results: Vec<VectorSearchResult>, _query: &VectorSearchQuery) -> Vec<VectorSearchResult> {
        if !self.config.enable_result_reranking {
            return results;
        }
        
        // Simple reranking based on quality score and recency
        results.sort_by(|a, b| {
            let score_a = a.similarity_score * 0.8; // Weight similarity
            let score_b = b.similarity_score * 0.8;
            
            // Add quality bonus (would need to be passed from context)
            // let quality_a = score_a + (quality_score_a * 0.2);
            // let quality_b = score_b + (quality_score_b * 0.2);
            
            score_b.partial_cmp(&score_a).unwrap()
        });
        
        // Update ranks
        for (i, result) in results.iter_mut().enumerate() {
            result.rank = i + 1;
        }
        
        results
    }
    
    /// Enhanced reranking with recency, usage, and quality factors
    fn rerank_results_enhanced(&self, mut results: Vec<VectorSearchResult>, processed_query: &ProcessedQuery) -> Vec<VectorSearchResult> {
        if !self.config.enable_result_reranking {
            return results;
        }
        
        // Calculate enhanced scores based on multiple factors
        for result in &mut results {
            let mut enhanced_score = result.similarity_score;
            
            // Apply intent-based boosting
            enhanced_score *= self.calculate_intent_boost(&result.metadata.content_type, &processed_query.intent);
            
            // Note: In a full implementation, you would get recency, usage, and quality data
            // from the context repository and apply the configured weights:
            // enhanced_score += recency_score * self.config.recency_weight;
            // enhanced_score += usage_score * self.config.usage_weight;
            // enhanced_score += quality_score * self.config.quality_weight;
            
            result.similarity_score = enhanced_score.min(1.0);
        }
        
        // Sort by enhanced score
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        
        // Update ranks
        for (i, result) in results.iter_mut().enumerate() {
            result.rank = i + 1;
        }
        
        results
    }
    
    /// Calculate intent-based boost for content types
    fn calculate_intent_boost(&self, content_type: &str, intent: &QueryIntent) -> f32 {
        let boost_factor = match (intent, content_type) {
            (QueryIntent::FindImplementation, "code_pattern") => 1.2,
            (QueryIntent::FindImplementation, "architectural_decision") => 1.1,
            (QueryIntent::FindBestPractices, "project_convention") => 1.2,
            (QueryIntent::FindBestPractices, "code_pattern") => 1.1,
            (QueryIntent::FindExamples, "code_pattern") => 1.2,
            (QueryIntent::FindExamples, "test_case") => 1.1,
            (QueryIntent::FindArchitecture, "architectural_decision") => 1.2,
            (QueryIntent::FindArchitecture, "api_specification") => 1.1,
            (QueryIntent::FindSecurity, "security_policy") => 1.2,
            (QueryIntent::FindSecurity, "business_rule") => 1.1,
            (QueryIntent::FindPerformance, "performance_requirement") => 1.2,
            (QueryIntent::FindPerformance, "architectural_decision") => 1.1,
            (QueryIntent::FindDocumentation, "documentation") => 1.2,
            (QueryIntent::FindDocumentation, "feature_context") => 1.1,
            _ => 1.0, // No boost for non-matching combinations
        };
        
        boost_factor
    }
    
    /// Generate query suggestions based on indexed content
    async fn generate_query_suggestions(&self, partial_query: &str, _project_id: Option<&str>) -> Result<Vec<String>, SemanticSearchError> {
        // This is a simplified implementation
        // In a full implementation, you would analyze indexed content and common query patterns
        
        let suggestions = vec![
            format!("{} implementation", partial_query),
            format!("{} best practices", partial_query),
            format!("{} examples", partial_query),
            format!("{} patterns", partial_query),
            format!("{} architecture", partial_query),
        ];
        
        Ok(suggestions.into_iter()
            .filter(|s| s.len() > partial_query.len())
            .take(5)
            .collect())
    }
}

#[async_trait]
impl SemanticSearchService for SemanticSearchServiceImpl {
    async fn index_context(&self, context: &EnhancedContextItem) -> Result<(), SemanticSearchError> {
        debug!("Indexing context: {}", context.id);
        
        let embedding = self.generate_context_embedding(context).await?;
        self.embedding_repository.store_embedding(&embedding).await?;
        
        info!("Successfully indexed context: {}", context.id);
        Ok(())
    }
    
    async fn index_contexts_batch(&self, contexts: &[EnhancedContextItem]) -> Result<(), SemanticSearchError> {
        info!("Indexing batch of {} contexts", contexts.len());
        
        let mut embeddings = Vec::new();
        
        for context in contexts {
            let embedding = self.generate_context_embedding(context).await?;
            embeddings.push(embedding);
        }
        
        self.embedding_repository.store_embeddings_batch(&embeddings).await?;
        
        info!("Successfully indexed {} contexts in batch", contexts.len());
        Ok(())
    }
    
    async fn search(&self, query: &VectorSearchQuery) -> Result<Vec<EnhancedSearchResult>, SemanticSearchError> {
        let start_time = std::time::Instant::now();
        
        debug!("Performing semantic search for query: {}", query.query_text);
        
        // Process query with intent detection and preprocessing
        let processed_query = if self.config.enable_intent_detection {
            self.process_query(&query.query_text)
        } else {
            ProcessedQuery {
                original_query: query.query_text.clone(),
                processed_query: query.query_text.clone(),
                intent: QueryIntent::General,
                key_terms: Vec::new(),
                expanded_terms: Vec::new(),
                content_type_hints: Vec::new(),
                confidence: 1.0,
            }
        };
        
        debug!("Query processed - Intent: {:?}, Confidence: {:.2}", 
               processed_query.intent, processed_query.confidence);
        
        // Generate query embedding using processed query
        let embedding_start = std::time::Instant::now();
        let query_embedding = self.get_query_embedding(&processed_query.processed_query).await?;
        let embedding_time = embedding_start.elapsed().as_millis() as u64;
        
        // Create enhanced query with embedding and filters based on intent
        let mut enhanced_query = query.clone();
        enhanced_query.query_embedding = Some(query_embedding);
        
        // Apply content type filters based on intent
        if !processed_query.content_type_hints.is_empty() {
            let content_type_strings: Vec<String> = processed_query.content_type_hints
                .iter()
                .map(|ct| ct.as_str().to_string())
                .collect();
            
            if enhanced_query.filters.content_types.is_none() {
                enhanced_query.filters.content_types = Some(content_type_strings);
            } else {
                // Merge with existing filters
                enhanced_query.filters.content_types.as_mut().unwrap().extend(content_type_strings);
            }
        }
        
        // Perform vector search
        let search_start = std::time::Instant::now();
        let project_filter = query.filters.project_ids.as_ref()
            .and_then(|ids| ids.first())
            .map(|s| s.as_str());
        
        let mut vector_results = self.embedding_repository
            .find_similar_embeddings(&enhanced_query, project_filter)
            .await?;
        
        let search_time = search_start.elapsed().as_millis() as u64;
        
        // Apply enhanced reranking with recency, usage, and quality factors
        vector_results = self.rerank_results_enhanced(vector_results, &processed_query);
        
        // Convert to enhanced results
        let mut enhanced_results = Vec::new();
        let mut filters_applied = vec!["similarity_threshold".to_string()];
        
        if !processed_query.content_type_hints.is_empty() {
            filters_applied.push("content_type_intent".to_string());
        }
        
        for vector_result in vector_results {
            let search_metadata = SearchMetadata {
                query_processing_time_ms: start_time.elapsed().as_millis() as u64,
                embedding_generation_time_ms: embedding_time,
                similarity_calculation_time_ms: search_time,
                total_candidates_evaluated: 0, // Would be tracked in repository
                filters_applied: filters_applied.clone(),
                ranking_method_used: format!("{}+intent", query.ranking_method.as_str()),
            };
            
            let relevance_explanation = format!(
                "Semantic similarity: {:.3}, Intent: {}, Confidence: {:.2}, Content type: {}",
                vector_result.similarity_score,
                processed_query.intent.as_str(),
                processed_query.confidence,
                vector_result.metadata.content_type
            );
            
            enhanced_results.push(EnhancedSearchResult {
                relevance_explanation,
                context_item: None, // Would be populated by higher-level service
                vector_result,
                search_metadata,
            });
        }
        
        info!("Search completed: {} results in {}ms (Intent: {:?})", 
              enhanced_results.len(), 
              start_time.elapsed().as_millis(),
              processed_query.intent);
        
        Ok(enhanced_results)
    }
    
    async fn find_similar_contexts(&self, context_id: &str, max_results: usize) -> Result<Vec<EnhancedSearchResult>, SemanticSearchError> {
        debug!("Finding similar contexts to: {}", context_id);
        
        // Get the embedding for the source context
        let source_embedding = self.embedding_repository
            .get_embedding_by_context_id(context_id)
            .await?
            .ok_or_else(|| SemanticSearchError::QueryProcessingError {
                message: format!("No embedding found for context: {}", context_id)
            })?;
        
        // Create a search query using the source embedding
        let query = VectorSearchQuery {
            query_text: "similar_context_search".to_string(),
            query_embedding: Some(source_embedding.embedding_vector),
            similarity_threshold: 0.5, // Lower threshold for similarity search
            max_results,
            filters: Default::default(),
            ranking_method: crate::models::embedding::RankingMethod::CosineSimilarity,
        };
        
        let results = self.search(&query).await?;
        
        // Filter out the source context itself
        let filtered_results: Vec<_> = results.into_iter()
            .filter(|r| r.vector_result.context_id != context_id)
            .collect();
        
        info!("Found {} similar contexts to {}", filtered_results.len(), context_id);
        Ok(filtered_results)
    }
    
    async fn suggest_queries(&self, partial_query: &str, project_id: Option<&str>) -> Result<Vec<String>, SemanticSearchError> {
        debug!("Generating query suggestions for: {}", partial_query);
        
        let suggestions = self.generate_query_suggestions(partial_query, project_id).await?;
        
        debug!("Generated {} query suggestions", suggestions.len());
        Ok(suggestions)
    }
    
    async fn update_context_index(&self, context: &EnhancedContextItem) -> Result<(), SemanticSearchError> {
        debug!("Updating index for context: {}", context.id);
        
        // Check if embedding already exists
        let exists = self.embedding_repository
            .embedding_exists(&context.id)
            .await?;
        
        if exists {
            // Remove old embedding
            self.embedding_repository
                .delete_embedding(&context.id)
                .await?;
        }
        
        // Add new embedding
        self.index_context(context).await?;
        
        info!("Updated index for context: {}", context.id);
        Ok(())
    }
    
    async fn remove_from_index(&self, context_id: &str) -> Result<(), SemanticSearchError> {
        debug!("Removing context from index: {}", context_id);
        
        self.embedding_repository
            .delete_embedding(context_id)
            .await?;
        
        info!("Removed context from index: {}", context_id);
        Ok(())
    }
    
    async fn get_index_stats(&self, project_id: Option<&str>) -> Result<SearchIndexStats, SemanticSearchError> {
        debug!("Getting index statistics");
        
        let embedding_stats = self.embedding_repository
            .get_embedding_stats(project_id)
            .await?;
        
        let stats = SearchIndexStats {
            total_indexed_items: embedding_stats.total_embeddings as usize,
            items_by_content_type: HashMap::new(), // Would be calculated from embeddings
            items_by_project: HashMap::new(), // Would be calculated from embeddings
            average_embedding_quality: 0.8, // Would be calculated from quality scores
            index_freshness_score: 0.9, // Would be calculated based on update times
            last_updated: embedding_stats.newest_embedding.unwrap_or_else(chrono::Utc::now),
        };
        
        debug!("Index stats: {} total items", stats.total_indexed_items);
        Ok(stats)
    }
    
    async fn rebuild_index(&self, project_id: &str, contexts: &[EnhancedContextItem]) -> Result<(), SemanticSearchError> {
        info!("Rebuilding search index for project: {}", project_id);
        
        // Delete existing embeddings for the project
        self.embedding_repository
            .delete_embeddings_by_project(project_id)
            .await?;
        
        // Reindex all contexts
        self.index_contexts_batch(contexts).await?;
        
        info!("Successfully rebuilt index for project: {}", project_id);
        Ok(())
    }
}