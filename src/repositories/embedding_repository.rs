use crate::models::embedding::{ContextEmbedding, VectorSearchQuery, VectorSearchResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqliteResult, Row};
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Error types for embedding repository operations
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingRepositoryError {
    #[error("Database error: {message}")]
    DatabaseError { message: String },
    
    #[error("Serialization error: {source}")]
    SerializationError { source: serde_json::Error },
    
    #[error("Embedding not found: {id}")]
    EmbeddingNotFound { id: String },
    
    #[error("Invalid vector data: {message}")]
    InvalidVectorData { message: String },
}

impl From<rusqlite::Error> for EmbeddingRepositoryError {
    fn from(error: rusqlite::Error) -> Self {
        EmbeddingRepositoryError::DatabaseError {
            message: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for EmbeddingRepositoryError {
    fn from(error: serde_json::Error) -> Self {
        EmbeddingRepositoryError::SerializationError { source: error }
    }
}

/// Repository trait for embedding operations
#[async_trait]
pub trait EmbeddingRepository: Send + Sync {
    /// Store a single embedding
    async fn store_embedding(&self, embedding: &ContextEmbedding) -> Result<(), EmbeddingRepositoryError>;
    
    /// Store multiple embeddings in batch
    async fn store_embeddings_batch(&self, embeddings: &[ContextEmbedding]) -> Result<(), EmbeddingRepositoryError>;
    
    /// Retrieve embedding by context ID
    async fn get_embedding_by_context_id(&self, context_id: &str) -> Result<Option<ContextEmbedding>, EmbeddingRepositoryError>;
    
    /// Retrieve all embeddings for a project
    async fn get_embeddings_by_project(&self, project_id: &str) -> Result<Vec<ContextEmbedding>, EmbeddingRepositoryError>;
    
    /// Find embeddings by similarity (brute force for SQLite)
    async fn find_similar_embeddings(&self, query: &VectorSearchQuery, project_id: Option<&str>) -> Result<Vec<VectorSearchResult>, EmbeddingRepositoryError>;
    
    /// Update embedding
    async fn update_embedding(&self, embedding: &ContextEmbedding) -> Result<(), EmbeddingRepositoryError>;
    
    /// Delete embedding by context ID
    async fn delete_embedding(&self, context_id: &str) -> Result<(), EmbeddingRepositoryError>;
    
    /// Delete all embeddings for a project
    async fn delete_embeddings_by_project(&self, project_id: &str) -> Result<(), EmbeddingRepositoryError>;
    
    /// Get embedding statistics
    async fn get_embedding_stats(&self, project_id: Option<&str>) -> Result<EmbeddingStats, EmbeddingRepositoryError>;
    
    /// Check if embedding exists for context
    async fn embedding_exists(&self, context_id: &str) -> Result<bool, EmbeddingRepositoryError>;
}

/// Statistics about embeddings
#[derive(Debug, Clone)]
pub struct EmbeddingStats {
    pub total_embeddings: u64,
    pub embeddings_by_model: std::collections::HashMap<String, u64>,
    pub average_vector_dimension: f64,
    pub oldest_embedding: Option<DateTime<Utc>>,
    pub newest_embedding: Option<DateTime<Utc>>,
}

/// SQLite implementation of EmbeddingRepository
pub struct SqliteEmbeddingRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteEmbeddingRepository {
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }
    
    /// Initialize the embeddings table
    pub async fn initialize(&self) -> Result<(), EmbeddingRepositoryError> {
        let conn = self.connection.lock().await;
        
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS context_embeddings (
                id TEXT PRIMARY KEY,
                context_id TEXT NOT NULL,
                project_id TEXT,
                embedding_vector TEXT NOT NULL, -- JSON array of floats
                embedding_model TEXT NOT NULL,
                embedding_version TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                content_type TEXT,
                content_length INTEGER,
                tokenization_method TEXT,
                preprocessing_steps TEXT, -- JSON array
                quality_score REAL,
                custom_metadata TEXT, -- JSON object
                created_at TEXT NOT NULL,
                updated_at TEXT DEFAULT (datetime('now')),
                FOREIGN KEY (project_id) REFERENCES projects(id),
                UNIQUE(context_id, embedding_model, embedding_version)
            );
            
            CREATE INDEX IF NOT EXISTS idx_embeddings_context_id ON context_embeddings(context_id);
            CREATE INDEX IF NOT EXISTS idx_embeddings_project_id ON context_embeddings(project_id);
            CREATE INDEX IF NOT EXISTS idx_embeddings_model ON context_embeddings(embedding_model);
            CREATE INDEX IF NOT EXISTS idx_embeddings_content_hash ON context_embeddings(content_hash);
            CREATE INDEX IF NOT EXISTS idx_embeddings_created_at ON context_embeddings(created_at);
            "#,
        )?;
        
        info!("Embedding repository initialized successfully");
        Ok(())
    }
    
    /// Convert database row to ContextEmbedding
    fn row_to_embedding(&self, row: &Row) -> SqliteResult<ContextEmbedding> {
        let embedding_vector_json: String = row.get("embedding_vector")?;
        let embedding_vector: Vec<f32> = serde_json::from_str(&embedding_vector_json)
            .map_err(|e| rusqlite::Error::InvalidColumnType(
                0, 
                "embedding_vector".to_string(), 
                rusqlite::types::Type::Text
            ))?;
        
        let preprocessing_steps_json: String = row.get("preprocessing_steps")?;
        let preprocessing_steps: Vec<String> = serde_json::from_str(&preprocessing_steps_json)
            .unwrap_or_default();
        
        let custom_metadata_json: String = row.get("custom_metadata")?;
        let custom_fields: std::collections::HashMap<String, serde_json::Value> = 
            serde_json::from_str(&custom_metadata_json).unwrap_or_default();
        
        let created_at_str: String = row.get("created_at")?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                0, 
                "created_at".to_string(), 
                rusqlite::types::Type::Text
            ))?
            .with_timezone(&Utc);
        
        let tokenization_method_str: String = row.get("tokenization_method")?;
        let tokenization_method = match tokenization_method_str.as_str() {
            "standard" => crate::models::embedding::TokenizationMethod::Standard,
            "code_aware" => crate::models::embedding::TokenizationMethod::CodeAware,
            "hybrid" => crate::models::embedding::TokenizationMethod::Hybrid,
            custom => crate::models::embedding::TokenizationMethod::Custom(custom.to_string()),
        };
        
        let metadata = crate::models::embedding::EmbeddingMetadata {
            content_type: row.get("content_type")?,
            content_length: row.get::<_, i64>("content_length")? as usize,
            tokenization_method,
            preprocessing_steps,
            quality_score: row.get("quality_score")?,
            custom_fields,
        };
        
        Ok(ContextEmbedding {
            id: row.get("id")?,
            context_id: row.get("context_id")?,
            embedding_vector,
            embedding_model: row.get("embedding_model")?,
            embedding_version: row.get("embedding_version")?,
            content_hash: row.get("content_hash")?,
            metadata,
            created_at,
        })
    }
    
    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }

        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

#[async_trait]
impl EmbeddingRepository for SqliteEmbeddingRepository {
    async fn store_embedding(&self, embedding: &ContextEmbedding) -> Result<(), EmbeddingRepositoryError> {
        let conn = self.connection.lock().await;
        
        let embedding_vector_json = serde_json::to_string(&embedding.embedding_vector)?;
        let preprocessing_steps_json = serde_json::to_string(&embedding.metadata.preprocessing_steps)?;
        let custom_metadata_json = serde_json::to_string(&embedding.metadata.custom_fields)?;
        
        // Extract project_id from context (this would need to be passed or looked up)
        // For now, we'll leave it as None and handle it in the service layer
        let project_id: Option<String> = None;
        
        conn.execute(
            r#"
            INSERT OR REPLACE INTO context_embeddings (
                id, context_id, project_id, embedding_vector, embedding_model, 
                embedding_version, content_hash, content_type, content_length,
                tokenization_method, preprocessing_steps, quality_score, 
                custom_metadata, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                embedding.id,
                embedding.context_id,
                project_id,
                embedding_vector_json,
                embedding.embedding_model,
                embedding.embedding_version,
                embedding.content_hash,
                embedding.metadata.content_type,
                embedding.metadata.content_length as i64,
                embedding.metadata.tokenization_method.as_str(),
                preprocessing_steps_json,
                embedding.metadata.quality_score,
                custom_metadata_json,
                embedding.created_at.to_rfc3339(),
            ],
        )?;
        
        debug!("Stored embedding for context: {}", embedding.context_id);
        Ok(())
    }
    
    async fn store_embeddings_batch(&self, embeddings: &[ContextEmbedding]) -> Result<(), EmbeddingRepositoryError> {
        let conn = self.connection.lock().await;
        
        let tx = conn.unchecked_transaction()?;
        
        for embedding in embeddings {
            let embedding_vector_json = serde_json::to_string(&embedding.embedding_vector)?;
            let preprocessing_steps_json = serde_json::to_string(&embedding.metadata.preprocessing_steps)?;
            let custom_metadata_json = serde_json::to_string(&embedding.metadata.custom_fields)?;
            
            let project_id: Option<String> = None; // Would be determined by service layer
            
            tx.execute(
                r#"
                INSERT OR REPLACE INTO context_embeddings (
                    id, context_id, project_id, embedding_vector, embedding_model, 
                    embedding_version, content_hash, content_type, content_length,
                    tokenization_method, preprocessing_steps, quality_score, 
                    custom_metadata, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                "#,
                params![
                    embedding.id,
                    embedding.context_id,
                    project_id,
                    embedding_vector_json,
                    embedding.embedding_model,
                    embedding.embedding_version,
                    embedding.content_hash,
                    embedding.metadata.content_type,
                    embedding.metadata.content_length as i64,
                    embedding.metadata.tokenization_method.as_str(),
                    preprocessing_steps_json,
                    embedding.metadata.quality_score,
                    custom_metadata_json,
                    embedding.created_at.to_rfc3339(),
                ],
            )?;
        }
        
        tx.commit()?;
        info!("Stored {} embeddings in batch", embeddings.len());
        Ok(())
    }
    
    async fn get_embedding_by_context_id(&self, context_id: &str) -> Result<Option<ContextEmbedding>, EmbeddingRepositoryError> {
        let conn = self.connection.lock().await;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, context_id, embedding_vector, embedding_model, embedding_version,
                   content_hash, content_type, content_length, tokenization_method,
                   preprocessing_steps, quality_score, custom_metadata, created_at
            FROM context_embeddings 
            WHERE context_id = ?1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )?;
        
        let embedding_iter = stmt.query_map([context_id], |row| {
            self.row_to_embedding(row)
        })?;
        
        for embedding in embedding_iter {
            return Ok(Some(embedding?));
        }
        
        Ok(None)
    }
    
    async fn get_embeddings_by_project(&self, project_id: &str) -> Result<Vec<ContextEmbedding>, EmbeddingRepositoryError> {
        let conn = self.connection.lock().await;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, context_id, embedding_vector, embedding_model, embedding_version,
                   content_hash, content_type, content_length, tokenization_method,
                   preprocessing_steps, quality_score, custom_metadata, created_at
            FROM context_embeddings 
            WHERE project_id = ?1
            ORDER BY created_at DESC
            "#,
        )?;
        
        let embedding_iter = stmt.query_map([project_id], |row| {
            self.row_to_embedding(row)
        })?;
        
        let mut embeddings = Vec::new();
        for embedding in embedding_iter {
            embeddings.push(embedding?);
        }
        
        Ok(embeddings)
    }
    
    async fn find_similar_embeddings(&self, query: &VectorSearchQuery, project_id: Option<&str>) -> Result<Vec<VectorSearchResult>, EmbeddingRepositoryError> {
        let conn = self.connection.lock().await;
        
        // Build query with optional project filter
        let sql = if project_id.is_some() {
            r#"
            SELECT id, context_id, embedding_vector, embedding_model, embedding_version,
                   content_hash, content_type, content_length, tokenization_method,
                   preprocessing_steps, quality_score, custom_metadata, created_at
            FROM context_embeddings 
            WHERE project_id = ?1
            "#
        } else {
            r#"
            SELECT id, context_id, embedding_vector, embedding_model, embedding_version,
                   content_hash, content_type, content_length, tokenization_method,
                   preprocessing_steps, quality_score, custom_metadata, created_at
            FROM context_embeddings
            "#
        };
        
        let mut stmt = conn.prepare(sql)?;
        
        let embeddings: Result<Vec<_>, _> = if let Some(pid) = project_id {
            stmt.query_map([pid], |row| self.row_to_embedding(row))?
                .collect()
        } else {
            stmt.query_map([], |row| self.row_to_embedding(row))?
                .collect()
        };
        
        let embeddings = embeddings?;
        
        let mut results = Vec::new();
        
        // Get query embedding (this would be provided by the service layer)
        let query_embedding = query.query_embedding.as_ref()
            .ok_or_else(|| EmbeddingRepositoryError::InvalidVectorData {
                message: "Query embedding not provided".to_string()
            })?;
        
        for embedding in embeddings {
            
            // Calculate similarity
            let similarity = self.cosine_similarity(query_embedding, &embedding.embedding_vector);
            
            if similarity >= query.similarity_threshold {
                let distance = 1.0 - similarity;
                
                results.push(VectorSearchResult {
                    context_id: embedding.context_id,
                    similarity_score: similarity,
                    distance,
                    rank: 0, // Will be set after sorting
                    metadata: crate::models::embedding::ResultMetadata {
                        content_type: embedding.metadata.content_type,
                        content_preview: "Preview not available".to_string(),
                        match_explanation: format!("Cosine similarity: {:.3}", similarity),
                        quality_indicators: vec![
                            format!("Quality: {:.2}", embedding.metadata.quality_score),
                            format!("Model: {}", embedding.embedding_model),
                        ],
                    },
                });
            }
        }
        
        // Sort by similarity (descending)
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        
        // Set ranks and limit results
        for (i, result) in results.iter_mut().enumerate() {
            result.rank = i + 1;
        }
        
        results.truncate(query.max_results);
        
        debug!("Found {} similar embeddings", results.len());
        Ok(results)
    }
    
    async fn update_embedding(&self, embedding: &ContextEmbedding) -> Result<(), EmbeddingRepositoryError> {
        // For SQLite, update is the same as store (INSERT OR REPLACE)
        self.store_embedding(embedding).await
    }
    
    async fn delete_embedding(&self, context_id: &str) -> Result<(), EmbeddingRepositoryError> {
        let conn = self.connection.lock().await;
        
        let rows_affected = conn.execute(
            "DELETE FROM context_embeddings WHERE context_id = ?1",
            [context_id],
        )?;
        
        if rows_affected == 0 {
            warn!("No embedding found to delete for context: {}", context_id);
        } else {
            debug!("Deleted embedding for context: {}", context_id);
        }
        
        Ok(())
    }
    
    async fn delete_embeddings_by_project(&self, project_id: &str) -> Result<(), EmbeddingRepositoryError> {
        let conn = self.connection.lock().await;
        
        let rows_affected = conn.execute(
            "DELETE FROM context_embeddings WHERE project_id = ?1",
            [project_id],
        )?;
        
        info!("Deleted {} embeddings for project: {}", rows_affected, project_id);
        Ok(())
    }
    
    async fn get_embedding_stats(&self, project_id: Option<&str>) -> Result<EmbeddingStats, EmbeddingRepositoryError> {
        let conn = self.connection.lock().await;
        
        let (where_clause, params): (&str, Vec<&str>) = if let Some(pid) = project_id {
            ("WHERE project_id = ?1", vec![pid])
        } else {
            ("", vec![])
        };
        
        // Get total count
        let total_embeddings: u64 = if let Some(pid) = project_id {
            conn.query_row(
                "SELECT COUNT(*) FROM context_embeddings WHERE project_id = ?1",
                [pid],
                |row| row.get::<_, i64>(0)
            )? as u64
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM context_embeddings",
                [],
                |row| row.get::<_, i64>(0)
            )? as u64
        };
        
        // Get embeddings by model
        let mut embeddings_by_model = std::collections::HashMap::new();
        if let Some(pid) = project_id {
            let mut stmt = conn.prepare(
                "SELECT embedding_model, COUNT(*) FROM context_embeddings WHERE project_id = ?1 GROUP BY embedding_model"
            )?;
            let model_iter = stmt.query_map([pid], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
            })?;
            
            for result in model_iter {
                let (model, count) = result?;
                embeddings_by_model.insert(model, count);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT embedding_model, COUNT(*) FROM context_embeddings GROUP BY embedding_model"
            )?;
            let model_iter = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
            })?;
            
            for result in model_iter {
                let (model, count) = result?;
                embeddings_by_model.insert(model, count);
            }
        }
        
        // Get date range
        let (oldest_str, newest_str): (Option<String>, Option<String>) = if let Some(pid) = project_id {
            conn.query_row(
                "SELECT MIN(created_at), MAX(created_at) FROM context_embeddings WHERE project_id = ?1",
                [pid],
                |row| {
                    Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?))
                }
            )?
        } else {
            conn.query_row(
                "SELECT MIN(created_at), MAX(created_at) FROM context_embeddings",
                [],
                |row| {
                    Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?))
                }
            )?
        };
        
        let oldest_embedding = oldest_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let newest_embedding = newest_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        // Calculate average dimension (this is a rough estimate)
        let average_vector_dimension = 384.0; // Default for most sentence transformers
        
        Ok(EmbeddingStats {
            total_embeddings,
            embeddings_by_model,
            average_vector_dimension,
            oldest_embedding,
            newest_embedding,
        })
    }
    
    async fn embedding_exists(&self, context_id: &str) -> Result<bool, EmbeddingRepositoryError> {
        let conn = self.connection.lock().await;
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM context_embeddings WHERE context_id = ?1",
            [context_id],
            |row| row.get(0),
        )?;
        
        Ok(count > 0)
    }
}