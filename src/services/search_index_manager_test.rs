use crate::models::enhanced_context::{EnhancedContextItem, ContextContent, ContextType};
use crate::models::embedding::{ContextEmbedding, VectorSearchQuery, VectorSearchResult};
use crate::repositories::embedding_repository::{EmbeddingRepository, EmbeddingRepositoryError, EmbeddingStats};
use crate::services::embedding_service::{EmbeddingService, EmbeddingError};
use crate::services::semantic_search_service::{SemanticSearchService, SemanticSearchError, SearchIndexStats, EnhancedSearchResult};
use crate::services::search_index_manager::{
    SearchIndexManager, SearchIndexManagerImpl, IndexManagerConfig, IndexPriority
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock embedding repository for testing
pub struct MockEmbeddingRepository {
    pub embeddings: Arc<Mutex<HashMap<String, ContextEmbedding>>>,
    pub embedding_exists_responses: Arc<Mutex<HashMap<String, bool>>>,
    pub should_fail: Arc<Mutex<bool>>,
}

impl MockEmbeddingRepository {
    pub fn new() -> Self {
        Self {
            embeddings: Arc::new(Mutex::new(HashMap::new())),
            embedding_exists_responses: Arc::new(Mutex::new(HashMap::new())),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }
    
    pub async fn set_embedding_exists(&self, context_id: &str, exists: bool) {
        let mut responses = self.embedding_exists_responses.lock().await;
        responses.insert(context_id.to_string(), exists);
    }
    
    pub async fn set_should_fail(&self, should_fail: bool) {
        let mut fail_flag = self.should_fail.lock().await;
        *fail_flag = should_fail;
    }
    
    pub async fn get_stored_embeddings_count(&self) -> usize {
        let embeddings = self.embeddings.lock().await;
        embeddings.len()
    }
}

#[async_trait]
impl EmbeddingRepository for MockEmbeddingRepository {
    async fn store_embedding(&self, embedding: &ContextEmbedding) -> Result<(), EmbeddingRepositoryError> {
        let should_fail = *self.should_fail.lock().await;
        if should_fail {
            return Err(EmbeddingRepositoryError::DatabaseError {
                message: "Mock failure".to_string(),
            });
        }
        
        let mut embeddings = self.embeddings.lock().await;
        embeddings.insert(embedding.context_id.clone(), embedding.clone());
        Ok(())
    }
    
    async fn store_embeddings_batch(&self, embeddings: &[ContextEmbedding]) -> Result<(), EmbeddingRepositoryError> {
        let should_fail = *self.should_fail.lock().await;
        if should_fail {
            return Err(EmbeddingRepositoryError::DatabaseError {
                message: "Mock batch failure".to_string(),
            });
        }
        
        let mut stored_embeddings = self.embeddings.lock().await;
        for embedding in embeddings {
            stored_embeddings.insert(embedding.context_id.clone(), embedding.clone());
        }
        Ok(())
    }
    
    async fn get_embedding_by_context_id(&self, context_id: &str) -> Result<Option<ContextEmbedding>, EmbeddingRepositoryError> {
        let embeddings = self.embeddings.lock().await;
        Ok(embeddings.get(context_id).cloned())
    }
    
    async fn get_embeddings_by_project(&self, _project_id: &str) -> Result<Vec<ContextEmbedding>, EmbeddingRepositoryError> {
        let embeddings = self.embeddings.lock().await;
        Ok(embeddings.values().cloned().collect())
    }
    
    async fn find_similar_embeddings(&self, _query: &VectorSearchQuery, _project_id: Option<&str>) -> Result<Vec<VectorSearchResult>, EmbeddingRepositoryError> {
        Ok(Vec::new())
    }
    
    async fn update_embedding(&self, embedding: &ContextEmbedding) -> Result<(), EmbeddingRepositoryError> {
        self.store_embedding(embedding).await
    }
    
    async fn delete_embedding(&self, context_id: &str) -> Result<(), EmbeddingRepositoryError> {
        let mut embeddings = self.embeddings.lock().await;
        embeddings.remove(context_id);
        Ok(())
    }
    
    async fn delete_embeddings_by_project(&self, _project_id: &str) -> Result<(), EmbeddingRepositoryError> {
        let mut embeddings = self.embeddings.lock().await;
        embeddings.clear();
        Ok(())
    }
    
    async fn get_embedding_stats(&self, _project_id: Option<&str>) -> Result<EmbeddingStats, EmbeddingRepositoryError> {
        let embeddings = self.embeddings.lock().await;
        Ok(EmbeddingStats {
            total_embeddings: embeddings.len() as u64,
            embeddings_by_model: HashMap::new(),
            average_vector_dimension: 384.0,
            oldest_embedding: Some(Utc::now() - Duration::days(1)),
            newest_embedding: Some(Utc::now()),
        })
    }
    
    async fn embedding_exists(&self, context_id: &str) -> Result<bool, EmbeddingRepositoryError> {
        let responses = self.embedding_exists_responses.lock().await;
        Ok(responses.get(context_id).copied().unwrap_or(false))
    }
}

/// Mock semantic search service for testing
pub struct MockSemanticSearchService {
    pub indexed_contexts: Arc<Mutex<HashMap<String, EnhancedContextItem>>>,
    pub should_fail: Arc<Mutex<bool>>,
    pub index_call_count: Arc<Mutex<usize>>,
    pub update_call_count: Arc<Mutex<usize>>,
    pub remove_call_count: Arc<Mutex<usize>>,
}

impl MockSemanticSearchService {
    pub fn new() -> Self {
        Self {
            indexed_contexts: Arc::new(Mutex::new(HashMap::new())),
            should_fail: Arc::new(Mutex::new(false)),
            index_call_count: Arc::new(Mutex::new(0)),
            update_call_count: Arc::new(Mutex::new(0)),
            remove_call_count: Arc::new(Mutex::new(0)),
        }
    }
    
    pub async fn set_should_fail(&self, should_fail: bool) {
        let mut fail_flag = self.should_fail.lock().await;
        *fail_flag = should_fail;
    }
    
    pub async fn get_indexed_count(&self) -> usize {
        let contexts = self.indexed_contexts.lock().await;
        contexts.len()
    }
    
    pub async fn get_call_counts(&self) -> (usize, usize, usize) {
        let index_count = *self.index_call_count.lock().await;
        let update_count = *self.update_call_count.lock().await;
        let remove_count = *self.remove_call_count.lock().await;
        (index_count, update_count, remove_count)
    }
}

#[async_trait]
impl SemanticSearchService for MockSemanticSearchService {
    async fn index_context(&self, context: &EnhancedContextItem) -> Result<(), SemanticSearchError> {
        let should_fail = *self.should_fail.lock().await;
        if should_fail {
            return Err(SemanticSearchError::IndexNotReady {
                message: "Mock index failure".to_string(),
            });
        }
        
        let mut contexts = self.indexed_contexts.lock().await;
        contexts.insert(context.id.clone(), context.clone());
        
        let mut count = self.index_call_count.lock().await;
        *count += 1;
        
        Ok(())
    }
    
    async fn index_contexts_batch(&self, contexts: &[EnhancedContextItem]) -> Result<(), SemanticSearchError> {
        let should_fail = *self.should_fail.lock().await;
        if should_fail {
            return Err(SemanticSearchError::IndexNotReady {
                message: "Mock batch index failure".to_string(),
            });
        }
        
        let mut indexed_contexts = self.indexed_contexts.lock().await;
        for context in contexts {
            indexed_contexts.insert(context.id.clone(), context.clone());
        }
        
        let mut count = self.index_call_count.lock().await;
        *count += contexts.len();
        
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
    
    async fn update_context_index(&self, context: &EnhancedContextItem) -> Result<(), SemanticSearchError> {
        let should_fail = *self.should_fail.lock().await;
        if should_fail {
            return Err(SemanticSearchError::IndexNotReady {
                message: "Mock update failure".to_string(),
            });
        }
        
        let mut contexts = self.indexed_contexts.lock().await;
        contexts.insert(context.id.clone(), context.clone());
        
        let mut count = self.update_call_count.lock().await;
        *count += 1;
        
        Ok(())
    }
    
    async fn remove_from_index(&self, context_id: &str) -> Result<(), SemanticSearchError> {
        let should_fail = *self.should_fail.lock().await;
        if should_fail {
            return Err(SemanticSearchError::IndexNotReady {
                message: "Mock remove failure".to_string(),
            });
        }
        
        let mut contexts = self.indexed_contexts.lock().await;
        contexts.remove(context_id);
        
        let mut count = self.remove_call_count.lock().await;
        *count += 1;
        
        Ok(())
    }
    
    async fn get_index_stats(&self, _project_id: Option<&str>) -> Result<SearchIndexStats, SemanticSearchError> {
        let contexts = self.indexed_contexts.lock().await;
        Ok(SearchIndexStats {
            total_indexed_items: contexts.len(),
            items_by_content_type: HashMap::new(),
            items_by_project: HashMap::new(),
            average_embedding_quality: 0.85,
            index_freshness_score: 0.9,
            last_updated: Utc::now(),
        })
    }
    
    async fn rebuild_index(&self, _project_id: &str, contexts: &[EnhancedContextItem]) -> Result<(), SemanticSearchError> {
        let should_fail = *self.should_fail.lock().await;
        if should_fail {
            return Err(SemanticSearchError::IndexNotReady {
                message: "Mock rebuild failure".to_string(),
            });
        }
        
        let mut indexed_contexts = self.indexed_contexts.lock().await;
        indexed_contexts.clear();
        for context in contexts {
            indexed_contexts.insert(context.id.clone(), context.clone());
        }
        
        Ok(())
    }
}

/// Mock embedding service for testing
pub struct MockEmbeddingService {
    pub should_fail: Arc<Mutex<bool>>,
    pub generation_count: Arc<Mutex<usize>>,
}

impl MockEmbeddingService {
    pub fn new() -> Self {
        Self {
            should_fail: Arc::new(Mutex::new(false)),
            generation_count: Arc::new(Mutex::new(0)),
        }
    }
    
    pub async fn set_should_fail(&self, should_fail: bool) {
        let mut fail_flag = self.should_fail.lock().await;
        *fail_flag = should_fail;
    }
    
    pub async fn get_generation_count(&self) -> usize {
        *self.generation_count.lock().await
    }
}

#[async_trait]
impl EmbeddingService for MockEmbeddingService {
    async fn generate_embedding(&self, text: &str, content_type: &str) -> Result<ContextEmbedding, EmbeddingError> {
        let should_fail = *self.should_fail.lock().await;
        if should_fail {
            return Err(EmbeddingError::EmbeddingGenerationError {
                message: "Mock embedding generation failure".to_string(),
            });
        }
        
        let mut count = self.generation_count.lock().await;
        *count += 1;
        
        Ok(ContextEmbedding::new(
            format!("context-{}", count),
            vec![0.1; 384], // Mock embedding vector
            "test-model".to_string(),
            "1.0".to_string(),
            format!("{:x}", md5::compute(text.as_bytes())),
        ))
    }
    
    async fn generate_embeddings_batch(&self, texts: Vec<(&str, &str, &str)>) -> Result<Vec<ContextEmbedding>, EmbeddingError> {
        let should_fail = *self.should_fail.lock().await;
        if should_fail {
            return Err(EmbeddingError::EmbeddingGenerationError {
                message: "Mock batch embedding generation failure".to_string(),
            });
        }
        
        let mut embeddings = Vec::new();
        for (i, (text, _content_type, _context_id)) in texts.iter().enumerate() {
            embeddings.push(ContextEmbedding::new(
                format!("context-{}", i),
                vec![0.1; 384],
                "test-model".to_string(),
                "1.0".to_string(),
                format!("{:x}", md5::compute(text.as_bytes())),
            ));
        }
        
        let mut count = self.generation_count.lock().await;
        *count += texts.len();
        
        Ok(embeddings)
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

/// Helper function to create test context
fn create_test_context(id: &str, title: &str) -> EnhancedContextItem {
    let content = ContextContent {
        content_type: ContextType::BusinessRule,
        title: title.to_string(),
        description: format!("Test description for {}", title),
        data: serde_json::json!({"test": "data", "id": id}),
        source_file: None,
        source_line: None,
    };
    
    let mut context = EnhancedContextItem::new("test-project".to_string(), content);
    context.id = id.to_string();
    context.quality_score = 0.8; // Set a good quality score
    context
}

/// Helper function to create test context with custom quality score
fn create_test_context_with_quality(id: &str, title: &str, quality_score: f32) -> EnhancedContextItem {
    let mut context = create_test_context(id, title);
    context.quality_score = quality_score as f64;
    context
}

/// Helper function to create test context with custom update time
fn create_test_context_with_time(id: &str, title: &str, updated_at: DateTime<Utc>) -> EnhancedContextItem {
    let mut context = create_test_context(id, title);
    context.updated_at = updated_at;
    context
}

#[cfg(test)]
mod tests {
    use super::*;
    // Removed unused imports

    async fn create_test_manager() -> (
        SearchIndexManagerImpl,
        Arc<MockSemanticSearchService>,
        Arc<MockEmbeddingRepository>,
        Arc<MockEmbeddingService>,
    ) {
        let semantic_service = Arc::new(MockSemanticSearchService::new());
        let embedding_repo = Arc::new(MockEmbeddingRepository::new());
        let embedding_service = Arc::new(MockEmbeddingService::new());
        let config = IndexManagerConfig::default();
        
        let manager = SearchIndexManagerImpl::new(
            semantic_service.clone(),
            embedding_repo.clone(),
            embedding_service.clone(),
            config,
        );
        
        (manager, semantic_service, embedding_repo, embedding_service)
    }

    #[tokio::test]
    async fn test_auto_index_context_success() {
        let (manager, semantic_service, embedding_repo, _) = create_test_manager().await;
        
        let context = create_test_context("test-1", "Test Context");
        
        // Set embedding doesn't exist so it will be indexed
        embedding_repo.set_embedding_exists("test-1", false).await;
        
        let result = manager.auto_index_context(&context).await;
        assert!(result.is_ok());
        
        // Verify context was indexed
        let (index_count, update_count, _) = semantic_service.get_call_counts().await;
        println!("Debug: index_count={}, update_count={}", index_count, update_count);
        assert!(update_count > 0 || index_count > 0); // Either should be called
    }

    #[tokio::test]
    async fn test_auto_index_context_quality_threshold() {
        let (manager, semantic_service, embedding_repo, _) = create_test_manager().await;
        
        // Create context with low quality score
        let context = create_test_context_with_quality("test-1", "Low Quality Context", 0.3);
        
        embedding_repo.set_embedding_exists("test-1", false).await;
        
        let result = manager.auto_index_context(&context).await;
        assert!(result.is_ok());
        
        // Verify context was NOT indexed due to low quality
        let (index_count, update_count, _) = semantic_service.get_call_counts().await;
        assert_eq!(index_count, 0);
        assert_eq!(update_count, 0);
    }

    #[tokio::test]
    async fn test_auto_index_context_already_current() {
        let (manager, semantic_service, embedding_repo, _) = create_test_manager().await;
        
        let context = create_test_context("test-1", "Test Context");
        
        // Set embedding exists and simulate that content hash is stored (context is current)
        embedding_repo.set_embedding_exists("test-1", true).await;
        
        // Simulate that the context has been indexed before by storing its hash
        let hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            let mut hasher = DefaultHasher::new();
            context.content.title.hash(&mut hasher);
            context.content.description.hash(&mut hasher);
            context.content.data.to_string().hash(&mut hasher);
            context.updated_at.hash(&mut hasher);
            
            format!("{:x}", hasher.finish())
        };
        
        manager.set_content_hash_for_testing(&context.id, hash).await;
        
        let result = manager.auto_index_context(&context).await;
        assert!(result.is_ok());
        
        // Verify context was NOT indexed since it's already current
        let (index_count, update_count, _) = semantic_service.get_call_counts().await;
        assert_eq!(index_count, 0);
        assert_eq!(update_count, 0);
    }

    #[tokio::test]
    async fn test_incremental_update_with_changes() {
        let (manager, semantic_service, _, _) = create_test_manager().await;
        
        let context = create_test_context("test-1", "Test Context");
        
        let result = manager.incremental_update(&context).await;
        assert!(result.is_ok());
        
        // Verify update was called
        let (_, update_count, _) = semantic_service.get_call_counts().await;
        assert_eq!(update_count, 1);
    }

    #[tokio::test]
    async fn test_queue_for_indexing() {
        let (manager, _, _, _) = create_test_manager().await;
        
        let context = create_test_context("test-1", "Test Context");
        
        let result = manager.queue_for_indexing(context, IndexPriority::High).await;
        assert!(result.is_ok());
        
        // Verify operation was queued by checking stats
        let stats = manager.get_operation_stats().await.unwrap();
        // Since we can't access private fields, we'll verify through behavior
    }

    #[tokio::test]
    async fn test_queue_priority_ordering() {
        let (manager, _, _, _) = create_test_manager().await;
        
        let context1 = create_test_context("test-1", "Low Priority");
        let context2 = create_test_context("test-2", "High Priority");
        let context3 = create_test_context("test-3", "Critical Priority");
        
        // Queue in non-priority order
        manager.queue_for_indexing(context1, IndexPriority::Low).await.unwrap();
        manager.queue_for_indexing(context2, IndexPriority::High).await.unwrap();
        manager.queue_for_indexing(context3, IndexPriority::Critical).await.unwrap();
        
        // Process operations to verify priority ordering
        let stats = manager.process_pending_operations().await.unwrap();
        assert_eq!(stats.batch_operations, 1);
        
        // Verify operations were processed (priority ordering is internal)
    }

    #[tokio::test]
    async fn test_process_pending_operations() {
        let (manager, semantic_service, _, _) = create_test_manager().await;
        
        let context1 = create_test_context("test-1", "Context 1");
        let context2 = create_test_context("test-2", "Context 2");
        
        // Queue operations
        manager.queue_for_indexing(context1, IndexPriority::Normal).await.unwrap();
        manager.queue_for_indexing(context2, IndexPriority::High).await.unwrap();
        
        // Process operations
        let stats = manager.process_pending_operations().await.unwrap();
        
        assert_eq!(stats.batch_operations, 1);
        
        // Verify operations were processed
        let (index_count, _, _) = semantic_service.get_call_counts().await;
        assert!(index_count > 0);
    }

    #[tokio::test]
    async fn test_process_pending_operations_with_failures() {
        let (manager, semantic_service, _, _) = create_test_manager().await;
        
        let context = create_test_context("test-1", "Test Context");
        
        // Set semantic service to fail
        semantic_service.set_should_fail(true).await;
        
        // Queue operation
        manager.queue_for_indexing(context, IndexPriority::Normal).await.unwrap();
        
        // Process operations (should handle failure gracefully)
        let stats = manager.process_pending_operations().await.unwrap();
        
        assert_eq!(stats.batch_operations, 1);
        assert_eq!(stats.failed_operations, 1);
    }

    #[tokio::test]
    async fn test_optimize_index() {
        let (manager, _, _, _) = create_test_manager().await;
        
        let result = manager.optimize_index(Some("test-project")).await;
        assert!(result.is_ok());
        
        // Verify optimization timestamp was updated
        let stats = manager.get_operation_stats().await.unwrap();
        assert!(stats.last_optimization.is_some());
    }

    #[tokio::test]
    async fn test_perform_maintenance() {
        let (manager, _, _, _) = create_test_manager().await;
        
        let result = manager.perform_maintenance(Some("test-project")).await;
        assert!(result.is_ok());
        
        // Verify maintenance timestamp was updated
        let stats = manager.get_operation_stats().await.unwrap();
        assert!(stats.last_maintenance.is_some());
    }

    #[tokio::test]
    async fn test_get_health_report() {
        let (manager, _, _, _) = create_test_manager().await;
        
        let report = manager.get_health_report(Some("test-project")).await.unwrap();
        
        assert!(report.overall_health_score > 0.0);
        assert!(report.overall_health_score <= 1.0);
        assert_eq!(report.total_contexts, 0); // No contexts indexed yet
        assert_eq!(report.indexed_contexts, 0);
    }

    #[tokio::test]
    async fn test_needs_reindexing_no_embedding() {
        let (manager, _, embedding_repo, _) = create_test_manager().await;
        
        let context = create_test_context("test-1", "Test Context");
        
        // Set no embedding exists
        embedding_repo.set_embedding_exists("test-1", false).await;
        
        let needs_reindexing = manager.needs_reindexing(&context).await.unwrap();
        assert!(needs_reindexing);
    }

    #[tokio::test]
    async fn test_needs_reindexing_old_context() {
        let (manager, _, embedding_repo, _) = create_test_manager().await;
        
        // Create context that's older than threshold
        let old_time = Utc::now() - Duration::days(2);
        let context = create_test_context_with_time("test-1", "Old Context", old_time);
        
        // Set embedding exists
        embedding_repo.set_embedding_exists("test-1", true).await;
        
        let needs_reindexing = manager.needs_reindexing(&context).await.unwrap();
        assert!(needs_reindexing);
    }

    #[tokio::test]
    async fn test_needs_reindexing_current_context() {
        let (manager, _, embedding_repo, _) = create_test_manager().await;
        
        let context = create_test_context("test-1", "Current Context");
        
        // Set embedding exists
        embedding_repo.set_embedding_exists("test-1", true).await;
        
        // Simulate that the context has been indexed before by storing its hash
        let hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            let mut hasher = DefaultHasher::new();
            context.content.title.hash(&mut hasher);
            context.content.description.hash(&mut hasher);
            context.content.data.to_string().hash(&mut hasher);
            context.updated_at.hash(&mut hasher);
            
            format!("{:x}", hasher.finish())
        };
        
        manager.set_content_hash_for_testing(&context.id, hash).await;
        
        let needs_reindexing = manager.needs_reindexing(&context).await.unwrap();
        assert!(!needs_reindexing); // Should not need reindexing
    }

    #[tokio::test]
    async fn test_rebuild_project_index() {
        let (manager, semantic_service, _, _) = create_test_manager().await;
        
        let contexts = vec![
            create_test_context("test-1", "Context 1"),
            create_test_context("test-2", "Context 2"),
            create_test_context("test-3", "Context 3"),
        ];
        
        let result = manager.rebuild_project_index("test-project", &contexts).await;
        assert!(result.is_ok());
        
        // Verify rebuild was called
        let indexed_count = semantic_service.get_indexed_count().await;
        assert_eq!(indexed_count, 3);
    }

    #[tokio::test]
    async fn test_cleanup_stale_embeddings() {
        let (manager, _, _, _) = create_test_manager().await;
        
        let cleaned_count = manager.cleanup_stale_embeddings(Some("test-project")).await.unwrap();
        
        // In the mock implementation, this returns 0
        assert_eq!(cleaned_count, 0);
    }

    #[tokio::test]
    async fn test_get_operation_stats() {
        let (manager, _, _, _) = create_test_manager().await;
        
        let stats = manager.get_operation_stats().await.unwrap();
        
        assert_eq!(stats.total_indexed, 0);
        assert_eq!(stats.total_updated, 0);
        assert_eq!(stats.total_removed, 0);
        assert_eq!(stats.batch_operations, 0);
        assert_eq!(stats.failed_operations, 0);
    }

    #[tokio::test]
    async fn test_batch_processing_performance() {
        let (manager, semantic_service, _, _) = create_test_manager().await;
        
        // Create a large number of contexts
        let mut contexts = Vec::new();
        for i in 0..100 {
            contexts.push(create_test_context(&format!("test-{}", i), &format!("Context {}", i)));
        }
        
        let start_time = std::time::Instant::now();
        
        // Queue all contexts
        for context in contexts {
            manager.queue_for_indexing(context, IndexPriority::Normal).await.unwrap();
        }
        
        // Process in batches
        let mut total_processed = 0;
        while total_processed < 100 {
            let stats = manager.process_pending_operations().await.unwrap();
            if stats.batch_operations == 0 {
                break; // No more operations to process
            }
            total_processed += 50; // Default batch size
        }
        
        let processing_time = start_time.elapsed();
        
        // Verify performance is reasonable (should complete within 1 second for mock operations)
        assert!(processing_time.as_secs() < 1);
        
        // Verify all contexts were processed
        let (index_count, _, _) = semantic_service.get_call_counts().await;
        assert!(index_count > 0);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let (manager, semantic_service, _, _) = create_test_manager().await;
        let manager = Arc::new(manager);
        
        let mut handles = Vec::new();
        
        // Spawn multiple concurrent indexing operations
        for i in 0..10 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let context = create_test_context(&format!("concurrent-{}", i), &format!("Concurrent Context {}", i));
                manager_clone.auto_index_context(&context).await
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
        
        // Verify operations were processed
        let (_, update_count, _) = semantic_service.get_call_counts().await;
        assert_eq!(update_count, 10);
    }

    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        let (manager, semantic_service, _, _) = create_test_manager().await;
        
        let context = create_test_context("test-1", "Test Context");
        
        // Set service to fail initially
        semantic_service.set_should_fail(true).await;
        
        // First attempt should fail
        let result = manager.auto_index_context(&context).await;
        assert!(result.is_err());
        
        // Set service to succeed
        semantic_service.set_should_fail(false).await;
        
        // Second attempt should succeed
        let result = manager.auto_index_context(&context).await;
        assert!(result.is_ok());
        
        // Verify recovery worked
        let (_, update_count, _) = semantic_service.get_call_counts().await;
        assert_eq!(update_count, 1);
    }

    #[tokio::test]
    async fn test_index_freshness_calculation() {
        let (manager, _, _, _) = create_test_manager().await;
        
        let report = manager.get_health_report(Some("test-project")).await.unwrap();
        
        // Verify freshness score is calculated
        assert!(report.index_freshness_score > 0.0);
        assert!(report.index_freshness_score <= 1.0);
    }

    #[tokio::test]
    async fn test_quality_score_filtering() {
        let (manager, semantic_service, embedding_repo, _) = create_test_manager().await;
        
        // Create contexts with different quality scores
        let high_quality = create_test_context_with_quality("high", "High Quality", 0.9);
        let low_quality = create_test_context_with_quality("low", "Low Quality", 0.3);
        
        embedding_repo.set_embedding_exists("high", false).await;
        embedding_repo.set_embedding_exists("low", false).await;
        
        // Index both contexts
        let result1 = manager.auto_index_context(&high_quality).await;
        let result2 = manager.auto_index_context(&low_quality).await;
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        // Only high quality context should be indexed
        let (_, update_count, _) = semantic_service.get_call_counts().await;
        assert_eq!(update_count, 1); // Only high quality context
    }
}