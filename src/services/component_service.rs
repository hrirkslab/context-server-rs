use async_trait::async_trait;
use crate::models::framework::FrameworkComponent;
use crate::repositories::ComponentRepository;
use rmcp::model::ErrorData as McpError;
use uuid::Uuid;

/// Service for generic component operations following Single Responsibility Principle
#[async_trait]
pub trait ComponentService: Send + Sync {
    async fn create_component(
        &self, 
        project_id: &str, 
        component_name: &str, 
        component_type: &str, 
        architecture_layer: &str, 
        file_path: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<FrameworkComponent, McpError>;
    
    async fn get_component(&self, id: &str) -> Result<Option<FrameworkComponent>, McpError>;
    async fn list_components(&self, project_id: &str) -> Result<Vec<FrameworkComponent>, McpError>;
    async fn list_components_by_layer(&self, project_id: &str, layer: &str) -> Result<Vec<FrameworkComponent>, McpError>;
    async fn update_component(&self, component: &FrameworkComponent) -> Result<FrameworkComponent, McpError>;
    async fn delete_component(&self, id: &str) -> Result<bool, McpError>;
}

/// Implementation of ComponentService
pub struct ComponentServiceImpl<R: ComponentRepository> {
    repository: R,
}

impl<R: ComponentRepository> ComponentServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: ComponentRepository> ComponentService for ComponentServiceImpl<R> {
    async fn create_component(
        &self, 
        project_id: &str, 
        component_name: &str, 
        component_type: &str, 
        architecture_layer: &str, 
        file_path: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<FrameworkComponent, McpError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        let component = FrameworkComponent {
            id,
            project_id: project_id.to_string(),
            component_name: component_name.to_string(),
            component_type: component_type.to_string(),
            architecture_layer: architecture_layer.to_string(),
            file_path: file_path.map(|s| s.to_string()),
            dependencies: Vec::new(),
            metadata,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        };

        self.repository.create(&component).await
    }
    
    async fn get_component(&self, id: &str) -> Result<Option<FrameworkComponent>, McpError> {
        self.repository.find_by_id(id).await
    }

    async fn list_components(&self, project_id: &str) -> Result<Vec<FrameworkComponent>, McpError> {
        self.repository.find_by_project_id(project_id).await
    }

    async fn list_components_by_layer(&self, project_id: &str, layer: &str) -> Result<Vec<FrameworkComponent>, McpError> {
        self.repository.find_by_architecture_layer(project_id, layer).await
    }

    async fn update_component(&self, component: &FrameworkComponent) -> Result<FrameworkComponent, McpError> {
        self.repository.update(component).await
    }

    async fn delete_component(&self, id: &str) -> Result<bool, McpError> {
        self.repository.delete(id).await
    }
}
