use crate::models::embedding::{EmbeddingConfig, VectorSearchQuery};
use crate::models::enhanced_context::{
    EnhancedContextItem, ContextContent, ContextType, ContextMetadata,
};
use crate::repositories::embedding_repository::SqliteEmbeddingRepository;
use crate::services::embedding_service::EmbeddingServiceFactory;
use crate::services::semantic_search_service::{SemanticSearchServiceImpl, SemanticSearchConfig, SemanticSearchService};
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, debug};

/// Integration service that demonstrates the complete vector embedding workflow
pub struct VectorEmbeddingIntegration {
    semantic_search_service: SemanticSearchServiceImpl,
    embedding_repository: Arc<SqliteEmbeddingRepository>,
}

impl VectorEmbeddingIntegration {
    /// Create a new integration instance with in-memory database for testing
    pub async fn new_for_testing() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Create in-memory database and initialize it
        let conn = crate::db::init::init_db(":memory:")?;
        let conn = Arc::new(Mutex::new(conn));
        
        // Initialize embedding repository
        let embedding_repository = Arc::new(SqliteEmbeddingRepository::new(conn.clone()));
        embedding_repository.initialize().await?;
        
        // Create embedding service
        let embedding_config = EmbeddingConfig::default();
        let embedding_service = EmbeddingServiceFactory::create_initialized_service(embedding_config.clone()).await?;
        
        // Create semantic search service
        let search_config = SemanticSearchConfig {
            embedding_config,
            ..Default::default()
        };
        
        let semantic_search_service = SemanticSearchServiceImpl::new(
            Arc::from(embedding_service),
            embedding_repository.clone(),
            search_config,
        );
        
        Ok(Self {
            semantic_search_service,
            embedding_repository,
        })
    }
    
    /// Demonstrate the complete workflow: index contexts and perform searches
    pub async fn demonstrate_workflow(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting vector embedding workflow demonstration");
        
        // Create sample context items
        let contexts = self.create_sample_contexts();
        
        // Index the contexts
        info!("Indexing {} sample contexts", contexts.len());
        self.semantic_search_service.index_contexts_batch(&contexts).await?;
        
        // Perform various searches
        self.demonstrate_searches().await?;
        
        // Show similarity search
        self.demonstrate_similarity_search(&contexts[0].id).await?;
        
        // Show index statistics
        self.show_index_statistics().await?;
        
        info!("Vector embedding workflow demonstration completed successfully");
        Ok(())
    }
    
    /// Create sample context items for testing
    fn create_sample_contexts(&self) -> Vec<EnhancedContextItem> {
        vec![
            self.create_context(
                "business-rule-1",
                "User Authentication",
                "Users must authenticate using email and password with optional 2FA",
                ContextType::BusinessRule,
                vec!["authentication".to_string(), "security".to_string()],
            ),
            self.create_context(
                "arch-decision-1",
                "Database Choice",
                "We chose PostgreSQL for its ACID compliance and JSON support",
                ContextType::ArchitecturalDecision,
                vec!["database".to_string(), "postgresql".to_string()],
            ),
            self.create_context(
                "perf-req-1",
                "API Response Time",
                "All API endpoints must respond within 200ms under normal load",
                ContextType::PerformanceRequirement,
                vec!["performance".to_string(), "api".to_string()],
            ),
            self.create_context(
                "code-pattern-1",
                "Error Handling Pattern",
                "Use Result<T, E> for all fallible operations with custom error types",
                ContextType::CodePattern,
                vec!["error-handling".to_string(), "rust".to_string()],
            ),
            self.create_context(
                "security-policy-1",
                "Data Encryption",
                "All sensitive data must be encrypted at rest using AES-256",
                ContextType::SecurityPolicy,
                vec!["encryption".to_string(), "security".to_string()],
            ),
        ]
    }
    
    /// Helper to create a context item
    fn create_context(
        &self,
        id: &str,
        title: &str,
        description: &str,
        content_type: ContextType,
        tags: Vec<String>,
    ) -> EnhancedContextItem {
        let content = ContextContent {
            content_type: content_type.clone(),
            title: title.to_string(),
            description: description.to_string(),
            data: serde_json::json!({
                "details": description,
                "type": content_type.as_str()
            }),
            source_file: None,
            source_line: None,
        };
        
        let mut metadata = ContextMetadata::default();
        metadata.tags = tags;
        
        let mut context = EnhancedContextItem::new("test-project".to_string(), content);
        context.id = id.to_string();
        context.metadata = metadata;
        
        context
    }
    
    /// Demonstrate various search queries
    async fn demonstrate_searches(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let search_queries = vec![
            ("authentication security", "Searching for authentication and security related content"),
            ("database performance", "Searching for database and performance content"),
            ("error handling patterns", "Searching for error handling patterns"),
            ("API response time", "Searching for API response time requirements"),
            ("encryption data security", "Searching for encryption and data security"),
        ];
        
        for (query_text, description) in search_queries {
            info!("{}", description);
            
            let query = VectorSearchQuery {
                query_text: query_text.to_string(),
                similarity_threshold: 0.3, // Lower threshold for demonstration
                max_results: 3,
                ..Default::default()
            };
            
            let results = self.semantic_search_service.search(&query).await?;
            
            debug!("Query: '{}' returned {} results", query_text, results.len());
            for (i, result) in results.iter().enumerate() {
                debug!(
                    "  {}. Context: {} (similarity: {:.3})",
                    i + 1,
                    result.vector_result.context_id,
                    result.vector_result.similarity_score
                );
            }
        }
        
        Ok(())
    }
    
    /// Demonstrate similarity search
    async fn demonstrate_similarity_search(&self, context_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Finding contexts similar to: {}", context_id);
        
        let similar_results = self.semantic_search_service
            .find_similar_contexts(context_id, 3)
            .await?;
        
        debug!("Found {} similar contexts", similar_results.len());
        for (i, result) in similar_results.iter().enumerate() {
            debug!(
                "  {}. Similar context: {} (similarity: {:.3})",
                i + 1,
                result.vector_result.context_id,
                result.vector_result.similarity_score
            );
        }
        
        Ok(())
    }
    
    /// Show index statistics
    async fn show_index_statistics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Retrieving index statistics");
        
        let stats = self.semantic_search_service.get_index_stats(None).await?;
        
        info!("Index Statistics:");
        info!("  Total indexed items: {}", stats.total_indexed_items);
        info!("  Average embedding quality: {:.2}", stats.average_embedding_quality);
        info!("  Index freshness score: {:.2}", stats.index_freshness_score);
        info!("  Last updated: {}", stats.last_updated);
        
        Ok(())
    }
    
    /// Test query suggestions
    pub async fn test_query_suggestions(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Testing query suggestions");
        
        let partial_queries = vec!["auth", "database", "performance", "security"];
        
        for partial in partial_queries {
            let suggestions = self.semantic_search_service
                .suggest_queries(partial, None)
                .await?;
            
            debug!("Suggestions for '{}': {:?}", partial, suggestions);
        }
        
        Ok(())
    }
    
    /// Test index updates
    pub async fn test_index_updates(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Testing index updates");
        
        // Create a new context
        let new_context = self.create_context(
            "test-update",
            "Updated Context",
            "This is a test context for update operations",
            ContextType::Documentation,
            vec!["test".to_string(), "update".to_string()],
        );
        
        // Index it
        self.semantic_search_service.index_context(&new_context).await?;
        
        // Update it
        let mut updated_context = new_context.clone();
        updated_context.content.description = "This is an updated test context".to_string();
        
        self.semantic_search_service.update_context_index(&updated_context).await?;
        
        // Remove it
        self.semantic_search_service.remove_from_index(&updated_context.id).await?;
        
        info!("Index update operations completed successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_vector_embedding_workflow() {
        // Initialize tracing for test output
        let _ = tracing_subscriber::fmt::try_init();
        
        let integration = VectorEmbeddingIntegration::new_for_testing()
            .await
            .expect("Failed to create integration instance");
        
        integration
            .demonstrate_workflow()
            .await
            .expect("Workflow demonstration failed");
    }
    
    #[tokio::test]
    async fn test_query_suggestions() {
        let integration = VectorEmbeddingIntegration::new_for_testing()
            .await
            .expect("Failed to create integration instance");
        
        integration
            .test_query_suggestions()
            .await
            .expect("Query suggestions test failed");
    }
    
    #[tokio::test]
    async fn test_index_operations() {
        let integration = VectorEmbeddingIntegration::new_for_testing()
            .await
            .expect("Failed to create integration instance");
        
        integration
            .test_index_updates()
            .await
            .expect("Index operations test failed");
    }
}