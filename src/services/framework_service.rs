use crate::models::framework::FrameworkComponent;
use crate::repositories::FrameworkRepository;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;

/// Service for generic framework/language component operations
#[async_trait]
pub trait FrameworkService: Send + Sync {
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
    async fn list_components_by_layer(
        &self,
        project_id: &str,
        layer: &str,
    ) -> Result<Vec<FrameworkComponent>, McpError>;
    async fn update_component(
        &self,
        component: &FrameworkComponent,
    ) -> Result<FrameworkComponent, McpError>;
    async fn delete_component(&self, id: &str) -> Result<bool, McpError>;
}

/// Implementation of FrameworkService
pub struct FrameworkServiceImpl<R: FrameworkRepository> {
    repository: R,
}

impl<R: FrameworkRepository> FrameworkServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: FrameworkRepository + Send + Sync> FrameworkService for FrameworkServiceImpl<R> {
    async fn create_component(
        &self,
        project_id: &str,
        component_name: &str,
        component_type: &str,
        architecture_layer: &str,
        file_path: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<FrameworkComponent, McpError> {
        let component = FrameworkComponent {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            component_name: component_name.to_string(),
            component_type: component_type.to_string(),
            architecture_layer: architecture_layer.to_string(),
            file_path: file_path.map(|s| s.to_string()),
            dependencies: Vec::new(),
            metadata,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
        };

        self.repository.create(&component).await
    }

    async fn get_component(&self, id: &str) -> Result<Option<FrameworkComponent>, McpError> {
        self.repository.find_by_id(id).await
    }

    async fn list_components(&self, project_id: &str) -> Result<Vec<FrameworkComponent>, McpError> {
        self.repository.find_by_project_id(project_id).await
    }

    async fn list_components_by_layer(
        &self,
        project_id: &str,
        layer: &str,
    ) -> Result<Vec<FrameworkComponent>, McpError> {
        self.repository
            .find_by_architecture_layer(project_id, layer)
            .await
    }

    async fn update_component(
        &self,
        component: &FrameworkComponent,
    ) -> Result<FrameworkComponent, McpError> {
        let mut updated_component = component.clone();
        updated_component.updated_at = Some(chrono::Utc::now().to_rfc3339());
        self.repository.update(&updated_component).await
    }

    async fn delete_component(&self, id: &str) -> Result<bool, McpError> {
        self.repository.delete(id).await
    }
}
