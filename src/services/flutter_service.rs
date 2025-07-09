use async_trait::async_trait;
use crate::models::flutter::{FlutterComponent, ComponentType, ArchitectureLayer};
use crate::repositories::FlutterRepository;
use rmcp::model::ErrorData as McpError;
use uuid::Uuid;

/// Service for Flutter component operations following Single Responsibility Principle
#[async_trait]
pub trait FlutterService: Send + Sync {
    async fn create_component(
        &self, 
        project_id: &str, 
        component_name: &str, 
        component_type: &str, 
        architecture_layer: &str, 
        file_path: Option<&str>
    ) -> Result<FlutterComponent, McpError>;
    
    async fn get_component(&self, id: &str) -> Result<Option<FlutterComponent>, McpError>;
    async fn list_components(&self, project_id: &str) -> Result<Vec<FlutterComponent>, McpError>;
    async fn list_components_by_layer(&self, project_id: &str, layer: &str) -> Result<Vec<FlutterComponent>, McpError>;
    async fn update_component(&self, component: &FlutterComponent) -> Result<FlutterComponent, McpError>;
    async fn delete_component(&self, id: &str) -> Result<bool, McpError>;
}

/// Implementation of FlutterService
pub struct FlutterServiceImpl<R: FlutterRepository> {
    repository: R,
}

impl<R: FlutterRepository> FlutterServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    
    /// Convert string to ComponentType enum (OCP - Open for extension)
    fn parse_component_type(component_type: &str) -> ComponentType {
        match component_type {
            "widget" => ComponentType::Widget,
            "provider" => ComponentType::Provider,
            "service" => ComponentType::Service,
            "repository" => ComponentType::Repository,
            "model" => ComponentType::Model,
            "utility" => ComponentType::Utility,
            _ => ComponentType::Widget,
        }
    }
    
    /// Convert string to ArchitectureLayer enum (OCP - Open for extension)
    fn parse_architecture_layer(architecture_layer: &str) -> ArchitectureLayer {
        match architecture_layer {
            "presentation" => ArchitectureLayer::Presentation,
            "domain" => ArchitectureLayer::Domain,
            "data" => ArchitectureLayer::Data,
            "core" => ArchitectureLayer::Core,
            _ => ArchitectureLayer::Presentation,
        }
    }
}

#[async_trait]
impl<R: FlutterRepository> FlutterService for FlutterServiceImpl<R> {
    async fn create_component(
        &self, 
        project_id: &str, 
        component_name: &str, 
        component_type: &str, 
        architecture_layer: &str, 
        file_path: Option<&str>
    ) -> Result<FlutterComponent, McpError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        let component = FlutterComponent {
            id,
            project_id: project_id.to_string(),
            component_name: component_name.to_string(),
            component_type: Self::parse_component_type(component_type),
            architecture_layer: Self::parse_architecture_layer(architecture_layer),
            file_path: file_path.map(|s| s.to_string()),
            dependencies: Vec::new(),
            riverpod_scope: None,
            widget_type: None,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        };

        self.repository.create(&component).await
    }
    
    async fn get_component(&self, id: &str) -> Result<Option<FlutterComponent>, McpError> {
        self.repository.find_by_id(id).await
    }

    async fn list_components(&self, project_id: &str) -> Result<Vec<FlutterComponent>, McpError> {
        self.repository.find_by_project_id(project_id).await
    }

    async fn list_components_by_layer(&self, project_id: &str, layer: &str) -> Result<Vec<FlutterComponent>, McpError> {
        self.repository.find_by_architecture_layer(project_id, layer).await
    }

    async fn update_component(&self, component: &FlutterComponent) -> Result<FlutterComponent, McpError> {
        self.repository.update(component).await
    }

    async fn delete_component(&self, id: &str) -> Result<bool, McpError> {
        self.repository.delete(id).await
    }
}
