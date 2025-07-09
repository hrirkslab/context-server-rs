use async_trait::async_trait;
use crate::models::flutter::{DevelopmentPhase, PhaseStatus};
use crate::repositories::DevelopmentPhaseRepository;
use rmcp::model::ErrorData as McpError;
use uuid::Uuid;

/// Service for Development Phase operations following Single Responsibility Principle
#[async_trait]
#[allow(dead_code)]
pub trait DevelopmentPhaseService: Send + Sync {
    async fn create_phase(
        &self, 
        project_id: &str, 
        phase_name: &str, 
        phase_order: i32, 
        description: Option<&str>
    ) -> Result<DevelopmentPhase, McpError>;
    
    async fn get_phase(&self, id: &str) -> Result<Option<DevelopmentPhase>, McpError>;
    async fn list_phases(&self, project_id: &str) -> Result<Vec<DevelopmentPhase>, McpError>;
    async fn update_phase(&self, phase: &DevelopmentPhase) -> Result<DevelopmentPhase, McpError>;
    async fn delete_phase(&self, id: &str) -> Result<bool, McpError>;
    async fn start_phase(&self, id: &str) -> Result<DevelopmentPhase, McpError>;
    async fn complete_phase(&self, id: &str) -> Result<DevelopmentPhase, McpError>;
}

/// Implementation of DevelopmentPhaseService
pub struct DevelopmentPhaseServiceImpl<R: DevelopmentPhaseRepository> {
    repository: R,
}

impl<R: DevelopmentPhaseRepository> DevelopmentPhaseServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: DevelopmentPhaseRepository> DevelopmentPhaseService for DevelopmentPhaseServiceImpl<R> {
    async fn create_phase(
        &self, 
        project_id: &str, 
        phase_name: &str, 
        phase_order: i32, 
        description: Option<&str>
    ) -> Result<DevelopmentPhase, McpError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        let phase = DevelopmentPhase {
            id,
            project_id: project_id.to_string(),
            phase_name: phase_name.to_string(),
            phase_order,
            status: PhaseStatus::Pending,
            description: description.map(|s| s.to_string()),
            completion_criteria: Vec::new(),
            dependencies: Vec::new(),
            started_at: None,
            completed_at: None,
            created_at: Some(now),
        };

        self.repository.create(&phase).await
    }
    
    async fn get_phase(&self, id: &str) -> Result<Option<DevelopmentPhase>, McpError> {
        self.repository.find_by_id(id).await
    }

    async fn list_phases(&self, project_id: &str) -> Result<Vec<DevelopmentPhase>, McpError> {
        self.repository.find_by_project_id(project_id).await
    }

    async fn update_phase(&self, phase: &DevelopmentPhase) -> Result<DevelopmentPhase, McpError> {
        self.repository.update(phase).await
    }

    async fn delete_phase(&self, id: &str) -> Result<bool, McpError> {
        self.repository.delete(id).await
    }

    async fn start_phase(&self, id: &str) -> Result<DevelopmentPhase, McpError> {
        let mut phase = self.repository.find_by_id(id).await?
            .ok_or_else(|| McpError::invalid_params("Phase not found", None))?;
        
        phase.status = PhaseStatus::InProgress;
        phase.started_at = Some(chrono::Utc::now().to_rfc3339());
        
        self.repository.update(&phase).await
    }

    async fn complete_phase(&self, id: &str) -> Result<DevelopmentPhase, McpError> {
        let mut phase = self.repository.find_by_id(id).await?
            .ok_or_else(|| McpError::invalid_params("Phase not found", None))?;
        
        phase.status = PhaseStatus::Completed;
        phase.completed_at = Some(chrono::Utc::now().to_rfc3339());
        
        self.repository.update(&phase).await
    }
}
