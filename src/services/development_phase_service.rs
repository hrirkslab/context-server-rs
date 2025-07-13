use async_trait::async_trait;
use crate::models::development::{DevelopmentPhase, PhaseStatus};
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
        // First get the phase from the repository
        let phase_opt = self.repository.find_by_id(id).await?;
        let mut phase = match phase_opt {
            Some(p) => p,
            None => return Err(McpError::invalid_params("Phase not found", None)),
        };
        
        // Create a new development phase with InProgress status
        let mut updated_phase = DevelopmentPhase {
            id: phase.id,
            project_id: phase.project_id,
            phase_name: phase.phase_name,
            phase_order: phase.phase_order,
            status: PhaseStatus::InProgress,
            description: phase.description,
            completion_criteria: phase.completion_criteria,
            dependencies: phase.dependencies,
            started_at: Some(chrono::Utc::now().to_rfc3339()),
            completed_at: phase.completed_at,
            created_at: phase.created_at,
        };
        
        // Update the phase in the repository
        self.repository.update(&updated_phase).await
    }

    async fn complete_phase(&self, id: &str) -> Result<DevelopmentPhase, McpError> {
        // First get the phase from the repository
        let phase_opt = self.repository.find_by_id(id).await?;
        let mut phase = match phase_opt {
            Some(p) => p,
            None => return Err(McpError::invalid_params("Phase not found", None)),
        };
        
        // Create a new development phase with Completed status
        let mut updated_phase = DevelopmentPhase {
            id: phase.id,
            project_id: phase.project_id,
            phase_name: phase.phase_name,
            phase_order: phase.phase_order,
            status: PhaseStatus::Completed,
            description: phase.description,
            completion_criteria: phase.completion_criteria,
            dependencies: phase.dependencies,
            started_at: phase.started_at,
            completed_at: Some(chrono::Utc::now().to_rfc3339()),
            created_at: phase.created_at,
        };
        
        // Update the phase in the repository
        self.repository.update(&updated_phase).await
    }
}
