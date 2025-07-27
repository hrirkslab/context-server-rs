use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vector embedding for context content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEmbedding {
    pub id: String,
    pub context_id: String,
    pub embedding_vector: Vec<f32>,
    pub embedding_model: String,
    pub embedding_version: String,
    pub content_hash: String,
    pub metadata: EmbeddingMetadata,
    pub created_at: DateTime<Utc>,
}

impl ContextEmbedding {
    pub fn new(
        context_id: String,
        embedding_vector: Vec<f32>,
        embedding_model: String,
        embedding_version: String,
        content_hash: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            context_id,
            embedding_vector,
            embedding_model,
            embedding_version,
            content_hash,
            metadata: EmbeddingMetadata::default(),
            created_at: Utc::now(),
        }
    }

    /// Calculate cosine similarity with another embedding
    pub fn cosine_similarity(&self, other: &ContextEmbedding) -> f32 {
        if self.embedding_vector.len() != other.embedding_vector.len() {
            return 0.0;
        }

        let dot_product: f32 = self
            .embedding_vector
            .iter()
            .zip(other.embedding_vector.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = self.embedding_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.embedding_vector.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Calculate euclidean distance with another embedding
    pub fn euclidean_distance(&self, other: &ContextEmbedding) -> f32 {
        if self.embedding_vector.len() != other.embedding_vector.len() {
            return f32::INFINITY;
        }

        self.embedding_vector
            .iter()
            .zip(other.embedding_vector.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

/// Metadata for embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    pub content_type: String,
    pub content_length: usize,
    pub tokenization_method: TokenizationMethod,
    pub preprocessing_steps: Vec<String>,
    pub quality_score: f32,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl Default for EmbeddingMetadata {
    fn default() -> Self {
        Self {
            content_type: "text".to_string(),
            content_length: 0,
            tokenization_method: TokenizationMethod::Standard,
            preprocessing_steps: Vec::new(),
            quality_score: 1.0,
            custom_fields: HashMap::new(),
        }
    }
}

/// Methods for tokenizing content before embedding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenizationMethod {
    Standard,
    CodeAware,
    Hybrid,
    Custom(String),
}

impl TokenizationMethod {
    pub fn as_str(&self) -> &str {
        match self {
            TokenizationMethod::Standard => "standard",
            TokenizationMethod::CodeAware => "code_aware",
            TokenizationMethod::Hybrid => "hybrid",
            TokenizationMethod::Custom(name) => name,
        }
    }
}

/// Search query for vector similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchQuery {
    pub query_text: String,
    pub query_embedding: Option<Vec<f32>>,
    pub similarity_threshold: f32,
    pub max_results: usize,
    pub filters: SearchFilters,
    pub ranking_method: RankingMethod,
}

impl Default for VectorSearchQuery {
    fn default() -> Self {
        Self {
            query_text: String::new(),
            query_embedding: None,
            similarity_threshold: 0.7,
            max_results: 10,
            filters: SearchFilters::default(),
            ranking_method: RankingMethod::CosineSimilarity,
        }
    }
}

/// Filters for vector search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub project_ids: Option<Vec<String>>,
    pub content_types: Option<Vec<String>>,
    pub date_range: Option<DateRange>,
    pub quality_threshold: Option<f32>,
    pub tags: Option<Vec<String>>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            project_ids: None,
            content_types: None,
            date_range: None,
            quality_threshold: None,
            tags: None,
        }
    }
}

/// Date range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Methods for ranking search results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RankingMethod {
    CosineSimilarity,
    EuclideanDistance,
    Hybrid,
    Custom(String),
}

impl RankingMethod {
    pub fn as_str(&self) -> &str {
        match self {
            RankingMethod::CosineSimilarity => "cosine_similarity",
            RankingMethod::EuclideanDistance => "euclidean_distance",
            RankingMethod::Hybrid => "hybrid",
            RankingMethod::Custom(name) => name,
        }
    }
}

/// Result of vector similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub context_id: String,
    pub similarity_score: f32,
    pub distance: f32,
    pub rank: usize,
    pub metadata: ResultMetadata,
}

/// Metadata for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMetadata {
    pub content_type: String,
    pub content_preview: String,
    pub match_explanation: String,
    pub quality_indicators: Vec<String>,
}

/// Batch of embeddings for efficient processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingBatch {
    pub embeddings: Vec<ContextEmbedding>,
    pub batch_id: String,
    pub model_info: ModelInfo,
    pub processing_stats: BatchStats,
    pub created_at: DateTime<Utc>,
}

/// Information about the embedding model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_name: String,
    pub model_version: String,
    pub embedding_dimension: usize,
    pub max_sequence_length: usize,
    pub model_type: ModelType,
}

/// Types of embedding models
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    SentenceTransformer,
    BERT,
    CodeBERT,
    Custom(String),
}

/// Statistics for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStats {
    pub total_items: usize,
    pub successful_embeddings: usize,
    pub failed_embeddings: usize,
    pub processing_time_ms: u64,
    pub average_embedding_time_ms: f64,
}

/// Configuration for embedding generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub model_path: Option<String>,
    pub embedding_dimension: usize,
    pub max_sequence_length: usize,
    pub batch_size: usize,
    pub tokenization_method: TokenizationMethod,
    pub preprocessing_enabled: bool,
    pub cache_embeddings: bool,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "all-MiniLM-L6-v2".to_string(),
            model_path: None,
            embedding_dimension: 384,
            max_sequence_length: 512,
            batch_size: 32,
            tokenization_method: TokenizationMethod::Hybrid,
            preprocessing_enabled: true,
            cache_embeddings: true,
        }
    }
}