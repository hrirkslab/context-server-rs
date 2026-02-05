use crate::models::enhanced_context::{EnhancedContextItem, ContextType, ContextId, ProjectId};
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

/// Repository interface for EnhancedContextItem operations
#[async_trait]
pub trait EnhancedContextRepository: Send + Sync {
    /// Create a new enhanced context item
    async fn create_context(&self, context: &EnhancedContextItem) -> Result<EnhancedContextItem, McpError>;
    
    /// Find context item by ID
    async fn find_context_by_id(&self, id: &str) -> Result<Option<EnhancedContextItem>, McpError>;
    
    /// Find all context items for a project
    async fn find_contexts_by_project(&self, project_id: &str) -> Result<Vec<EnhancedContextItem>, McpError>;
    
    /// Find context items by type
    async fn find_contexts_by_type(&self, project_id: &str, context_type: ContextType) -> Result<Vec<EnhancedContextItem>, McpError>;
    
    /// Find context items by keywords (simple text search)
    async fn find_contexts_by_keywords(&self, project_id: &str, keywords: &[String]) -> Result<Vec<EnhancedContextItem>, McpError>;
    
    /// Update an existing context item
    async fn update_context(&self, context: &EnhancedContextItem) -> Result<EnhancedContextItem, McpError>;
    
    /// Delete a context item
    async fn delete_context(&self, id: &str) -> Result<bool, McpError>;
    
    /// Find contexts linked to a specific requirement
    async fn find_contexts_linked_to_requirement(&self, requirement_id: &str) -> Result<Vec<EnhancedContextItem>, McpError>;
    
    /// Find contexts linked to a specific task
    async fn find_contexts_linked_to_task(&self, task_id: &str) -> Result<Vec<EnhancedContextItem>, McpError>;
    
    /// Get context items with relationships to a specific context
    async fn find_related_contexts(&self, context_id: &str) -> Result<Vec<EnhancedContextItem>, McpError>;
    
    /// Update context quality score
    async fn update_quality_score(&self, context_id: &str, score: f64) -> Result<(), McpError>;
    
    /// Record context usage
    async fn record_context_usage(&self, context_id: &str) -> Result<(), McpError>;
}