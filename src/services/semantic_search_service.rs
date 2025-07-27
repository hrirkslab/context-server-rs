use crate::models::embedding::{
    ContextEmbedding, EmbeddingConfig, VectorSearchQuery, VectorSearchResult,
};
use crate::models::enhanced_context::{EnhancedContextItem, ContextContent};
use crate::repositories::embedding_repository::{EmbeddingRepository, EmbeddingRepositoryError};
use crate::services::embedding_service::{EmbeddingService, EmbeddingError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

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
        }
    }
}

/// Enhanced search result with context information
#[derive(Debug, Clone)]
pub struct EnhancedSearchResult {
    pub vector_result: VectorSearchResult,
    pub context_item: Option<EnhancedContextItem>,
    pub relevance_explanation: String,
    pub search_metadata: SearchMetadata,
}

/// Metadata about the search process
#[derive(Debug, Clone)]
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
        
        // Generate query embedding
        let embedding_start = std::time::Instant::now();
        let query_embedding = self.get_query_embedding(&query.query_text).await?;
        let embedding_time = embedding_start.elapsed().as_millis() as u64;
        
        // Create enhanced query with embedding
        let mut enhanced_query = query.clone();
        enhanced_query.query_embedding = Some(query_embedding);
        
        // Perform vector search
        let search_start = std::time::Instant::now();
        let project_filter = query.filters.project_ids.as_ref()
            .and_then(|ids| ids.first())
            .map(|s| s.as_str());
        
        let mut vector_results = self.embedding_repository
            .find_similar_embeddings(&enhanced_query, project_filter)
            .await?;
        
        let search_time = search_start.elapsed().as_millis() as u64;
        
        // Apply reranking
        vector_results = self.rerank_results(vector_results, query);
        
        // Convert to enhanced results
        let mut enhanced_results = Vec::new();
        for vector_result in vector_results {
            let search_metadata = SearchMetadata {
                query_processing_time_ms: start_time.elapsed().as_millis() as u64,
                embedding_generation_time_ms: embedding_time,
                similarity_calculation_time_ms: search_time,
                total_candidates_evaluated: 0, // Would be tracked in repository
                filters_applied: vec!["similarity_threshold".to_string()],
                ranking_method_used: query.ranking_method.as_str().to_string(),
            };
            
            enhanced_results.push(EnhancedSearchResult {
                relevance_explanation: format!(
                    "Semantic similarity: {:.3}, matched on content type: {}",
                    vector_result.similarity_score,
                    vector_result.metadata.content_type
                ),
                context_item: None, // Would be populated by higher-level service
                vector_result,
                search_metadata,
            });
        }
        
        info!("Search completed: {} results in {}ms", 
              enhanced_results.len(), 
              start_time.elapsed().as_millis());
        
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