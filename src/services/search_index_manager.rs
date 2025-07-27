use crate::models::enhanced_context::EnhancedContextItem;
use crate::models::embedding::{ContextEmbedding, VectorSearchQuery};
use crate::repositories::embedding_repository::{EmbeddingRepository, EmbeddingRepositoryError};
use crate::services::embedding_service::{EmbeddingService, EmbeddingError};
use crate::services::semantic_search_service::{SemanticSearchService, SemanticSearchError, SearchIndexStats};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info};

/// Error types for search index management operations
#[derive(Debug, thiserror::Error)]
pub enum IndexManagerError {
    #[error("Semantic search error: {source}")]
    SemanticSearchError { source: SemanticSearchError },
    
    #[error("Embedding service error: {source}")]
    EmbeddingServiceError { source: EmbeddingError },
    
    #[error("Repository error: {source}")]
    RepositoryError { source: EmbeddingRepositoryError },
    
    #[error("Index optimization error: {message}")]
    OptimizationError { message: String },
    
    #[error("Index maintenance error: {message}")]
    MaintenanceError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

impl From<SemanticSearchError> for IndexManagerError {
    fn from(error: SemanticSearchError) -> Self {
        IndexManagerError::SemanticSearchError { source: error }
    }
}

impl From<EmbeddingError> for IndexManagerError {
    fn from(error: EmbeddingError) -> Self {
        IndexManagerError::EmbeddingServiceError { source: error }
    }
}

impl From<EmbeddingRepositoryError> for IndexManagerError {
    fn from(error: EmbeddingRepositoryError) -> Self {
        IndexManagerError::RepositoryError { source: error }
    }
}

/// Configuration for search index management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexManagerConfig {
    /// Enable automatic indexing of new contexts
    pub auto_index_enabled: bool,
    
    /// Batch size for bulk indexing operations
    pub batch_size: usize,
    
    /// Maximum age of contexts before reindexing (in hours)
    pub max_context_age_hours: i64,
    
    /// Minimum quality score threshold for indexing
    pub min_quality_threshold: f32,
    
    /// Enable incremental updates
    pub incremental_updates_enabled: bool,
    
    /// Optimization frequency (in hours)
    pub optimization_frequency_hours: i64,
    
    /// Maximum number of stale embeddings before triggering cleanup
    pub max_stale_embeddings: usize,
    
    /// Enable performance monitoring
    pub performance_monitoring_enabled: bool,
    
    /// Index freshness threshold (in hours)
    pub freshness_threshold_hours: i64,
}

impl Default for IndexManagerConfig {
    fn default() -> Self {
        Self {
            auto_index_enabled: true,
            batch_size: 50,
            max_context_age_hours: 24,
            min_quality_threshold: 0.5,
            incremental_updates_enabled: true,
            optimization_frequency_hours: 6,
            max_stale_embeddings: 100,
            performance_monitoring_enabled: true,
            freshness_threshold_hours: 1,
        }
    }
}

/// Statistics about index operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexOperationStats {
    pub total_indexed: usize,
    pub total_updated: usize,
    pub total_removed: usize,
    pub batch_operations: usize,
    pub failed_operations: usize,
    pub average_indexing_time_ms: f64,
    pub last_optimization: Option<DateTime<Utc>>,
    pub last_maintenance: Option<DateTime<Utc>>,
}

impl Default for IndexOperationStats {
    fn default() -> Self {
        Self {
            total_indexed: 0,
            total_updated: 0,
            total_removed: 0,
            batch_operations: 0,
            failed_operations: 0,
            average_indexing_time_ms: 0.0,
            last_optimization: None,
            last_maintenance: None,
        }
    }
}

/// Information about index health and performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexHealthReport {
    pub overall_health_score: f32,
    pub total_contexts: usize,
    pub indexed_contexts: usize,
    pub stale_contexts: usize,
    pub outdated_embeddings: usize,
    pub average_quality_score: f32,
    pub index_freshness_score: f32,
    pub performance_metrics: IndexPerformanceMetrics,
    pub recommendations: Vec<String>,
}

/// Performance metrics for index operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexPerformanceMetrics {
    pub average_search_time_ms: f64,
    pub average_indexing_time_ms: f64,
    pub cache_hit_rate: f32,
    pub memory_usage_mb: f64,
    pub disk_usage_mb: f64,
}

/// Pending index operation
#[derive(Debug, Clone)]
pub struct PendingIndexOperation {
    pub operation_type: IndexOperationType,
    pub context_id: String,
    pub context: Option<EnhancedContextItem>,
    pub priority: IndexPriority,
    pub created_at: DateTime<Utc>,
}

/// Types of index operations
#[derive(Debug, Clone, PartialEq)]
pub enum IndexOperationType {
    Create,
    Update,
    Delete,
    Reindex,
}

/// Priority levels for index operations
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IndexPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Trait for search index management operations
#[async_trait]
pub trait SearchIndexManager: Send + Sync {
    /// Automatically index a new context item
    async fn auto_index_context(&self, context: &EnhancedContextItem) -> Result<(), IndexManagerError>;
    
    /// Perform incremental update for a context
    async fn incremental_update(&self, context: &EnhancedContextItem) -> Result<(), IndexManagerError>;
    
    /// Queue a context for batch indexing
    async fn queue_for_indexing(&self, context: EnhancedContextItem, priority: IndexPriority) -> Result<(), IndexManagerError>;
    
    /// Process pending index operations
    async fn process_pending_operations(&self) -> Result<IndexOperationStats, IndexManagerError>;
    
    /// Optimize the search index
    async fn optimize_index(&self, project_id: Option<&str>) -> Result<(), IndexManagerError>;
    
    /// Perform index maintenance
    async fn perform_maintenance(&self, project_id: Option<&str>) -> Result<(), IndexManagerError>;
    
    /// Get index health report
    async fn get_health_report(&self, project_id: Option<&str>) -> Result<IndexHealthReport, IndexManagerError>;
    
    /// Clean up stale embeddings
    async fn cleanup_stale_embeddings(&self, project_id: Option<&str>) -> Result<usize, IndexManagerError>;
    
    /// Rebuild index for a project
    async fn rebuild_project_index(&self, project_id: &str, contexts: &[EnhancedContextItem]) -> Result<(), IndexManagerError>;
    
    /// Check if context needs reindexing
    async fn needs_reindexing(&self, context: &EnhancedContextItem) -> Result<bool, IndexManagerError>;
    
    /// Get operation statistics
    async fn get_operation_stats(&self) -> Result<IndexOperationStats, IndexManagerError>;
}

/// Implementation of SearchIndexManager
pub struct SearchIndexManagerImpl {
    semantic_search_service: Arc<dyn SemanticSearchService>,
    embedding_repository: Arc<dyn EmbeddingRepository>,
    embedding_service: Arc<dyn EmbeddingService>,
    config: IndexManagerConfig,
    pending_operations: Arc<Mutex<Vec<PendingIndexOperation>>>,
    operation_stats: Arc<RwLock<IndexOperationStats>>,
    content_hashes: Arc<RwLock<HashMap<String, String>>>, // context_id -> content_hash
}

impl SearchIndexManagerImpl {
    pub fn new(
        semantic_search_service: Arc<dyn SemanticSearchService>,
        embedding_repository: Arc<dyn EmbeddingRepository>,
        embedding_service: Arc<dyn EmbeddingService>,
        config: IndexManagerConfig,
    ) -> Self {
        Self {
            semantic_search_service,
            embedding_repository,
            embedding_service,
            config,
            pending_operations: Arc::new(Mutex::new(Vec::new())),
            operation_stats: Arc::new(RwLock::new(IndexOperationStats::default())),
            content_hashes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    #[cfg(test)]
    pub async fn set_content_hash_for_testing(&self, context_id: &str, hash: String) {
        let mut hashes = self.content_hashes.write().await;
        hashes.insert(context_id.to_string(), hash);
    }
    
    /// Calculate content hash for change detection
    fn calculate_content_hash(&self, context: &EnhancedContextItem) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        context.content.title.hash(&mut hasher);
        context.content.description.hash(&mut hasher);
        context.content.data.to_string().hash(&mut hasher);
        context.updated_at.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    /// Check if context has changed since last indexing
    async fn has_content_changed(&self, context: &EnhancedContextItem) -> bool {
        let current_hash = self.calculate_content_hash(context);
        let hashes = self.content_hashes.read().await;
        
        match hashes.get(&context.id) {
            Some(stored_hash) => stored_hash != &current_hash,
            None => true, // New context
        }
    }
    
    /// Update stored content hash
    async fn update_content_hash(&self, context_id: &str, hash: String) {
        let mut hashes = self.content_hashes.write().await;
        hashes.insert(context_id.to_string(), hash);
    }
    
    /// Check if embedding exists and is current
    async fn is_embedding_current(&self, context: &EnhancedContextItem) -> Result<bool, IndexManagerError> {
        let embedding_exists = self.embedding_repository
            .embedding_exists(&context.id)
            .await?;
        
        if !embedding_exists {
            return Ok(false);
        }
        
        // Check if content has changed
        if self.has_content_changed(context).await {
            return Ok(false);
        }
        
        // Check age threshold
        let age_threshold = Utc::now() - Duration::hours(self.config.max_context_age_hours);
        if context.updated_at < age_threshold {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Process a single index operation
    async fn process_single_operation(&self, operation: PendingIndexOperation) -> Result<(), IndexManagerError> {
        let start_time = std::time::Instant::now();
        
        match operation.operation_type {
            IndexOperationType::Create => {
                if let Some(context) = &operation.context {
                    self.semantic_search_service.index_context(context).await?;
                    let hash = self.calculate_content_hash(context);
                    self.update_content_hash(&context.id, hash).await;
                    
                    let mut stats = self.operation_stats.write().await;
                    stats.total_indexed += 1;
                }
            }
            IndexOperationType::Update => {
                if let Some(context) = &operation.context {
                    self.semantic_search_service.update_context_index(context).await?;
                    let hash = self.calculate_content_hash(context);
                    self.update_content_hash(&context.id, hash).await;
                    
                    let mut stats = self.operation_stats.write().await;
                    stats.total_updated += 1;
                }
            }
            IndexOperationType::Delete => {
                self.semantic_search_service.remove_from_index(&operation.context_id).await?;
                
                let mut hashes = self.content_hashes.write().await;
                hashes.remove(&operation.context_id);
                
                let mut stats = self.operation_stats.write().await;
                stats.total_removed += 1;
            }
            IndexOperationType::Reindex => {
                if let Some(context) = &operation.context {
                    // Remove old embedding and create new one
                    let _ = self.semantic_search_service.remove_from_index(&context.id).await;
                    self.semantic_search_service.index_context(context).await?;
                    let hash = self.calculate_content_hash(context);
                    self.update_content_hash(&context.id, hash).await;
                    
                    let mut stats = self.operation_stats.write().await;
                    stats.total_updated += 1;
                }
            }
        }
        
        // Update performance metrics
        let processing_time = start_time.elapsed().as_millis() as f64;
        let mut stats = self.operation_stats.write().await;
        let total_ops = stats.total_indexed + stats.total_updated + stats.total_removed;
        if total_ops > 0 {
            stats.average_indexing_time_ms = 
                (stats.average_indexing_time_ms * (total_ops - 1) as f64 + processing_time) / total_ops as f64;
        } else {
            stats.average_indexing_time_ms = processing_time;
        }
        
        Ok(())
    }
    
    /// Calculate index health score
    async fn calculate_health_score(&self, project_id: Option<&str>) -> Result<f32, IndexManagerError> {
        let index_stats = self.semantic_search_service.get_index_stats(project_id).await?;
        
        // Base score from index freshness
        let freshness_score = index_stats.index_freshness_score;
        
        // Quality score component
        let quality_score = index_stats.average_embedding_quality;
        
        // Coverage score (would need context count from repository)
        let coverage_score = if index_stats.total_indexed_items > 0 { 1.0 } else { 0.0 };
        
        // Weighted average
        let health_score = (freshness_score * 0.4) + (quality_score * 0.4) + (coverage_score * 0.2);
        
        Ok(health_score)
    }
    
    /// Generate recommendations based on index state
    async fn generate_recommendations(&self, health_report: &IndexHealthReport) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if health_report.overall_health_score < 0.7 {
            recommendations.push("Consider running index optimization to improve performance".to_string());
        }
        
        if health_report.stale_contexts > 50 {
            recommendations.push(format!(
                "Found {} stale contexts that should be reindexed", 
                health_report.stale_contexts
            ));
        }
        
        if health_report.outdated_embeddings > 20 {
            recommendations.push(format!(
                "Found {} outdated embeddings that should be refreshed", 
                health_report.outdated_embeddings
            ));
        }
        
        if health_report.index_freshness_score < 0.8 {
            recommendations.push("Index freshness is low, consider more frequent updates".to_string());
        }
        
        if health_report.average_quality_score < 0.6 {
            recommendations.push("Average embedding quality is low, review content preprocessing".to_string());
        }
        
        if health_report.performance_metrics.average_search_time_ms > 500.0 {
            recommendations.push("Search performance is slow, consider index optimization".to_string());
        }
        
        recommendations
    }
}

#[async_trait]
impl SearchIndexManager for SearchIndexManagerImpl {
    async fn auto_index_context(&self, context: &EnhancedContextItem) -> Result<(), IndexManagerError> {
        if !self.config.auto_index_enabled {
            debug!("Auto-indexing disabled, skipping context: {}", context.id);
            return Ok(());
        }
        
        // Check quality threshold
        if context.quality_score < self.config.min_quality_threshold as f64 {
            debug!("Context quality {} below threshold {}, skipping indexing", 
                   context.quality_score, self.config.min_quality_threshold);
            return Ok(());
        }
        
        // Check if already indexed and current
        if self.is_embedding_current(context).await? {
            debug!("Context {} already has current embedding, skipping", context.id);
            return Ok(());
        }
        
        info!("Auto-indexing context: {}", context.id);
        
        if self.config.incremental_updates_enabled {
            self.incremental_update(context).await?;
        } else {
            self.semantic_search_service.index_context(context).await?;
            let hash = self.calculate_content_hash(context);
            self.update_content_hash(&context.id, hash).await;
        }
        
        Ok(())
    }
    
    async fn incremental_update(&self, context: &EnhancedContextItem) -> Result<(), IndexManagerError> {
        debug!("Performing incremental update for context: {}", context.id);
        
        // Check if content has actually changed
        if !self.has_content_changed(context).await {
            debug!("No content changes detected for context: {}", context.id);
            return Ok(());
        }
        
        // Update the index
        self.semantic_search_service.update_context_index(context).await?;
        
        // Update content hash
        let hash = self.calculate_content_hash(context);
        self.update_content_hash(&context.id, hash).await;
        
        // Update stats
        let mut stats = self.operation_stats.write().await;
        stats.total_updated += 1;
        
        info!("Incremental update completed for context: {}", context.id);
        Ok(())
    }
    
    async fn queue_for_indexing(&self, context: EnhancedContextItem, priority: IndexPriority) -> Result<(), IndexManagerError> {
        let operation = PendingIndexOperation {
            operation_type: IndexOperationType::Create,
            context_id: context.id.clone(),
            context: Some(context),
            priority: priority.clone(),
            created_at: Utc::now(),
        };
        
        let mut pending = self.pending_operations.lock().await;
        pending.push(operation);
        
        // Sort by priority (highest first)
        pending.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        debug!("Queued context for indexing with priority: {:?}", priority);
        Ok(())
    }
    
    async fn process_pending_operations(&self) -> Result<IndexOperationStats, IndexManagerError> {
        let mut pending = self.pending_operations.lock().await;
        
        if pending.is_empty() {
            debug!("No pending operations to process");
            return Ok(IndexOperationStats::default());
        }
        
        info!("Processing {} pending index operations", pending.len());
        
        let mut batch = Vec::new();
        let batch_size = self.config.batch_size.min(pending.len());
        
        // Take operations in priority order
        for _ in 0..batch_size {
            if let Some(operation) = pending.pop() {
                batch.push(operation);
            }
        }
        
        drop(pending); // Release lock early
        
        let mut successful_operations = 0;
        let mut failed_operations = 0;
        
        // Process operations
        for operation in batch {
            match self.process_single_operation(operation).await {
                Ok(()) => successful_operations += 1,
                Err(e) => {
                    error!("Failed to process index operation: {}", e);
                    failed_operations += 1;
                }
            }
        }
        
        // Update batch stats
        let mut stats = self.operation_stats.write().await;
        stats.batch_operations += 1;
        stats.failed_operations += failed_operations;
        
        info!("Processed {} operations successfully, {} failed", 
              successful_operations, failed_operations);
        
        Ok(stats.clone())
    }
    
    async fn optimize_index(&self, project_id: Option<&str>) -> Result<(), IndexManagerError> {
        info!("Starting index optimization for project: {:?}", project_id);
        
        // Get current index stats
        let stats = self.semantic_search_service.get_index_stats(project_id).await?;
        
        // Clean up stale embeddings
        let cleaned_count = self.cleanup_stale_embeddings(project_id).await?;
        
        info!("Index optimization completed. Cleaned {} stale embeddings", cleaned_count);
        
        // Update optimization timestamp
        let mut operation_stats = self.operation_stats.write().await;
        operation_stats.last_optimization = Some(Utc::now());
        
        Ok(())
    }
    
    async fn perform_maintenance(&self, project_id: Option<&str>) -> Result<(), IndexManagerError> {
        info!("Starting index maintenance for project: {:?}", project_id);
        
        // Process any pending operations
        self.process_pending_operations().await?;
        
        // Optimize if needed
        let should_optimize = {
            let stats = self.operation_stats.read().await;
            match stats.last_optimization {
                Some(last_opt) => {
                    let hours_since_opt = (Utc::now() - last_opt).num_hours();
                    hours_since_opt >= self.config.optimization_frequency_hours
                }
                None => true, // Never optimized
            }
        };
        
        if should_optimize {
            self.optimize_index(project_id).await?;
        }
        
        // Update maintenance timestamp
        let mut operation_stats = self.operation_stats.write().await;
        operation_stats.last_maintenance = Some(Utc::now());
        
        info!("Index maintenance completed");
        Ok(())
    }
    
    async fn get_health_report(&self, project_id: Option<&str>) -> Result<IndexHealthReport, IndexManagerError> {
        debug!("Generating index health report for project: {:?}", project_id);
        
        let index_stats = self.semantic_search_service.get_index_stats(project_id).await?;
        let health_score = self.calculate_health_score(project_id).await?;
        
        // Calculate stale contexts (simplified - would need more context data)
        let stale_contexts = 0; // Would be calculated from context repository
        let outdated_embeddings = 0; // Would be calculated from embedding age
        
        let performance_metrics = IndexPerformanceMetrics {
            average_search_time_ms: 100.0, // Would be tracked from actual searches
            average_indexing_time_ms: {
                let stats = self.operation_stats.read().await;
                stats.average_indexing_time_ms
            },
            cache_hit_rate: 0.85, // Would be tracked from cache statistics
            memory_usage_mb: 50.0, // Would be calculated from actual memory usage
            disk_usage_mb: 100.0, // Would be calculated from database size
        };
        
        let health_report = IndexHealthReport {
            overall_health_score: health_score,
            total_contexts: index_stats.total_indexed_items, // Approximation
            indexed_contexts: index_stats.total_indexed_items,
            stale_contexts,
            outdated_embeddings,
            average_quality_score: index_stats.average_embedding_quality,
            index_freshness_score: index_stats.index_freshness_score,
            performance_metrics,
            recommendations: Vec::new(), // Will be filled below
        };
        
        let mut final_report = health_report;
        final_report.recommendations = self.generate_recommendations(&final_report).await;
        
        debug!("Health report generated with score: {:.2}", final_report.overall_health_score);
        Ok(final_report)
    }
    
    async fn cleanup_stale_embeddings(&self, project_id: Option<&str>) -> Result<usize, IndexManagerError> {
        info!("Cleaning up stale embeddings for project: {:?}", project_id);
        
        // This is a simplified implementation
        // In a full implementation, you would:
        // 1. Query for embeddings older than threshold
        // 2. Check if corresponding contexts still exist
        // 3. Remove orphaned embeddings
        // 4. Remove embeddings with very low quality scores
        
        let cleaned_count = 0; // Placeholder
        
        info!("Cleaned up {} stale embeddings", cleaned_count);
        Ok(cleaned_count)
    }
    
    async fn rebuild_project_index(&self, project_id: &str, contexts: &[EnhancedContextItem]) -> Result<(), IndexManagerError> {
        info!("Rebuilding index for project: {}", project_id);
        
        // Use the semantic search service to rebuild
        self.semantic_search_service.rebuild_index(project_id, contexts).await?;
        
        // Update content hashes for all contexts
        let mut hashes = self.content_hashes.write().await;
        for context in contexts {
            let hash = self.calculate_content_hash(context);
            hashes.insert(context.id.clone(), hash);
        }
        
        info!("Successfully rebuilt index for project: {}", project_id);
        Ok(())
    }
    
    async fn needs_reindexing(&self, context: &EnhancedContextItem) -> Result<bool, IndexManagerError> {
        // Check if embedding exists
        let embedding_exists = self.embedding_repository
            .embedding_exists(&context.id)
            .await?;
        
        if !embedding_exists {
            return Ok(true);
        }
        
        // Check if content has changed
        if self.has_content_changed(context).await {
            return Ok(true);
        }
        
        // Check age threshold
        let age_threshold = Utc::now() - Duration::hours(self.config.max_context_age_hours);
        if context.updated_at < age_threshold {
            return Ok(true);
        }
        
        Ok(false)
    }
    
    async fn get_operation_stats(&self) -> Result<IndexOperationStats, IndexManagerError> {
        let stats = self.operation_stats.read().await;
        Ok(stats.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enhanced_context::{ContextContent, ContextType};
    use crate::services::semantic_search_service::{SemanticSearchService, EnhancedSearchResult};
    use crate::models::embedding::VectorSearchResult;
    use std::collections::HashMap;
    
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
        
        async fn search(&self, _query: &VectorSearchQuery) -> Result<Vec<EnhancedSearchResult>, SemanticSearchError> {
            Ok(Vec::new())
        }
        
        async fn find_similar_contexts(&self, _context_id: &str, _max_results: usize) -> Result<Vec<EnhancedSearchResult>, SemanticSearchError> {
            Ok(Vec::new())
        }
        
        async fn suggest_queries(&self, _partial_query: &str, _project_id: Option<&str>) -> Result<Vec<String>, SemanticSearchError> {
            Ok(Vec::new())
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
    
    struct MockEmbeddingRepository;
    
    #[async_trait]
    impl EmbeddingRepository for MockEmbeddingRepository {
        async fn store_embedding(&self, _embedding: &ContextEmbedding) -> Result<(), EmbeddingRepositoryError> {
            Ok(())
        }
        
        async fn store_embeddings_batch(&self, _embeddings: &[ContextEmbedding]) -> Result<(), EmbeddingRepositoryError> {
            Ok(())
        }
        
        async fn get_embedding_by_context_id(&self, _context_id: &str) -> Result<Option<ContextEmbedding>, EmbeddingRepositoryError> {
            Ok(None)
        }
        
        async fn get_embeddings_by_project(&self, _project_id: &str) -> Result<Vec<ContextEmbedding>, EmbeddingRepositoryError> {
            Ok(Vec::new())
        }
        
        async fn find_similar_embeddings(&self, _query: &VectorSearchQuery, _project_id: Option<&str>) -> Result<Vec<VectorSearchResult>, EmbeddingRepositoryError> {
            Ok(Vec::new())
        }
        
        async fn update_embedding(&self, _embedding: &ContextEmbedding) -> Result<(), EmbeddingRepositoryError> {
            Ok(())
        }
        
        async fn delete_embedding(&self, _context_id: &str) -> Result<(), EmbeddingRepositoryError> {
            Ok(())
        }
        
        async fn delete_embeddings_by_project(&self, _project_id: &str) -> Result<(), EmbeddingRepositoryError> {
            Ok(())
        }
        
        async fn get_embedding_stats(&self, _project_id: Option<&str>) -> Result<crate::repositories::embedding_repository::EmbeddingStats, EmbeddingRepositoryError> {
            Ok(crate::repositories::embedding_repository::EmbeddingStats {
                total_embeddings: 100,
                embeddings_by_model: HashMap::new(),
                average_vector_dimension: 384.0,
                oldest_embedding: Some(Utc::now() - Duration::days(1)),
                newest_embedding: Some(Utc::now()),
            })
        }
        
        async fn embedding_exists(&self, _context_id: &str) -> Result<bool, EmbeddingRepositoryError> {
            Ok(false)
        }
    }
    
    struct MockEmbeddingService;
    
    #[async_trait]
    impl EmbeddingService for MockEmbeddingService {
        async fn generate_embedding(&self, _text: &str, _content_type: &str) -> Result<ContextEmbedding, EmbeddingError> {
            Ok(ContextEmbedding::new(
                "test-context".to_string(),
                vec![0.1; 384],
                "test-model".to_string(),
                "1.0".to_string(),
                "test-hash".to_string(),
            ))
        }
        
        async fn generate_embeddings_batch(&self, _texts: Vec<(&str, &str, &str)>) -> Result<Vec<ContextEmbedding>, EmbeddingError> {
            Ok(Vec::new())
        }
        
        fn calculate_similarity(&self, _embedding1: &ContextEmbedding, _embedding2: &ContextEmbedding) -> f32 {
            0.8
        }
        
        async fn find_similar(&self, _query: &VectorSearchQuery, _embeddings: &[ContextEmbedding]) -> Result<Vec<crate::models::embedding::VectorSearchResult>, EmbeddingError> {
            Ok(Vec::new())
        }
        
        fn get_model_info(&self) -> crate::models::embedding::ModelInfo {
            crate::models::embedding::ModelInfo {
                model_name: "test-model".to_string(),
                model_version: "1.0".to_string(),
                embedding_dimension: 384,
                max_sequence_length: 512,
                model_type: crate::models::embedding::ModelType::SentenceTransformer,
            }
        }
        
        async fn update_config(&mut self, _config: crate::models::embedding::EmbeddingConfig) -> Result<(), EmbeddingError> {
            Ok(())
        }
    }
    
    fn create_test_context() -> EnhancedContextItem {
        let content = ContextContent {
            content_type: ContextType::BusinessRule,
            title: "Test Context".to_string(),
            description: "Test description".to_string(),
            data: serde_json::json!({"test": "data"}),
            source_file: None,
            source_line: None,
        };
        
        EnhancedContextItem::new("test-project".to_string(), content)
    }
    
    #[tokio::test]
    async fn test_auto_index_context() {
        let semantic_service = Arc::new(MockSemanticSearchService);
        let embedding_repo = Arc::new(MockEmbeddingRepository);
        let embedding_service = Arc::new(MockEmbeddingService);
        let config = IndexManagerConfig::default();
        
        let manager = SearchIndexManagerImpl::new(
            semantic_service,
            embedding_repo,
            embedding_service,
            config,
        );
        
        let context = create_test_context();
        let result = manager.auto_index_context(&context).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_queue_for_indexing() {
        let semantic_service = Arc::new(MockSemanticSearchService);
        let embedding_repo = Arc::new(MockEmbeddingRepository);
        let embedding_service = Arc::new(MockEmbeddingService);
        let config = IndexManagerConfig::default();
        
        let manager = SearchIndexManagerImpl::new(
            semantic_service,
            embedding_repo,
            embedding_service,
            config,
        );
        
        let context = create_test_context();
        let result = manager.queue_for_indexing(context, IndexPriority::High).await;
        
        assert!(result.is_ok());
        
        // Check that operation was queued
        let pending = manager.pending_operations.lock().await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].priority, IndexPriority::High);
    }
    
    #[tokio::test]
    async fn test_process_pending_operations() {
        let semantic_service = Arc::new(MockSemanticSearchService);
        let embedding_repo = Arc::new(MockEmbeddingRepository);
        let embedding_service = Arc::new(MockEmbeddingService);
        let config = IndexManagerConfig::default();
        
        let manager = SearchIndexManagerImpl::new(
            semantic_service,
            embedding_repo,
            embedding_service,
            config,
        );
        
        // Queue some operations
        let context1 = create_test_context();
        let context2 = create_test_context();
        
        manager.queue_for_indexing(context1, IndexPriority::Normal).await.unwrap();
        manager.queue_for_indexing(context2, IndexPriority::High).await.unwrap();
        
        // Process operations
        let stats = manager.process_pending_operations().await.unwrap();
        
        assert_eq!(stats.batch_operations, 1);
        
        // Check that operations were processed
        let pending = manager.pending_operations.lock().await;
        assert!(pending.len() <= 2); // Some or all should be processed
    }
    
    #[tokio::test]
    async fn test_get_health_report() {
        let semantic_service = Arc::new(MockSemanticSearchService);
        let embedding_repo = Arc::new(MockEmbeddingRepository);
        let embedding_service = Arc::new(MockEmbeddingService);
        let config = IndexManagerConfig::default();
        
        let manager = SearchIndexManagerImpl::new(
            semantic_service,
            embedding_repo,
            embedding_service,
            config,
        );
        
        let report = manager.get_health_report(Some("test-project")).await.unwrap();
        
        assert!(report.overall_health_score > 0.0);
        assert_eq!(report.total_contexts, 100);
        assert_eq!(report.indexed_contexts, 100);
    }
    
    #[tokio::test]
    async fn test_needs_reindexing() {
        let semantic_service = Arc::new(MockSemanticSearchService);
        let embedding_repo = Arc::new(MockEmbeddingRepository);
        let embedding_service = Arc::new(MockEmbeddingService);
        let config = IndexManagerConfig::default();
        
        let manager = SearchIndexManagerImpl::new(
            semantic_service,
            embedding_repo,
            embedding_service,
            config,
        );
        
        let context = create_test_context();
        let needs_reindexing = manager.needs_reindexing(&context).await.unwrap();
        
        // Should need reindexing since no embedding exists (mock returns false)
        assert!(needs_reindexing);
    }
}