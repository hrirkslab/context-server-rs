use crate::models::specification::{ProjectSpecification, Requirement, Task};
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

/// Repository interface for ProjectSpecification operations (DIP - Dependency Inversion)
#[async_trait]
pub trait SpecificationRepository: Send + Sync {
    async fn create_specification(&self, spec: &ProjectSpecification) -> Result<ProjectSpecification, McpError>;
    async fn find_specification_by_id(&self, id: &str) -> Result<Option<ProjectSpecification>, McpError>;
    async fn find_specifications_by_project(&self, project_id: &str) -> Result<Vec<ProjectSpecification>, McpError>;
    async fn find_specifications_by_type(&self, project_id: &str, spec_type: &str) -> Result<Vec<ProjectSpecification>, McpError>;
    async fn update_specification(&self, spec: &ProjectSpecification) -> Result<ProjectSpecification, McpError>;
    async fn delete_specification(&self, id: &str) -> Result<bool, McpError>;
    
    // Requirement operations
    async fn create_requirement(&self, requirement: &Requirement) -> Result<Requirement, McpError>;
    async fn find_requirement_by_id(&self, id: &str) -> Result<Option<Requirement>, McpError>;
    async fn find_requirements_by_spec(&self, spec_id: &str) -> Result<Vec<Requirement>, McpError>;
    async fn update_requirement(&self, requirement: &Requirement) -> Result<Requirement, McpError>;
    async fn delete_requirement(&self, id: &str) -> Result<bool, McpError>;
    
    // Task operations
    async fn create_task(&self, task: &Task) -> Result<Task, McpError>;
    async fn find_task_by_id(&self, id: &str) -> Result<Option<Task>, McpError>;
    async fn find_tasks_by_spec(&self, spec_id: &str) -> Result<Vec<Task>, McpError>;
    async fn find_tasks_by_status(&self, spec_id: &str, status: &str) -> Result<Vec<Task>, McpError>;
    async fn update_task(&self, task: &Task) -> Result<Task, McpError>;
    async fn delete_task(&self, id: &str) -> Result<bool, McpError>;
    
    // Relationship operations
    async fn link_requirement_to_context(&self, requirement_id: &str, context_id: &str) -> Result<(), McpError>;
    async fn link_task_to_context(&self, task_id: &str, context_id: &str) -> Result<(), McpError>;
    async fn link_task_to_requirement(&self, task_id: &str, requirement_id: &str) -> Result<(), McpError>;
    async fn unlink_requirement_from_context(&self, requirement_id: &str, context_id: &str) -> Result<(), McpError>;
    async fn unlink_task_from_context(&self, task_id: &str, context_id: &str) -> Result<(), McpError>;
    async fn unlink_task_from_requirement(&self, task_id: &str, requirement_id: &str) -> Result<(), McpError>;
}