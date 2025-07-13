use crate::models::architecture::ArchitectureLayer;
use crate::models::framework::FrameworkComponent;
use crate::services::FrameworkService;
use async_trait::async_trait;
use rmcp::model::ErrorData as McpError;
use std::str::FromStr;

/// Service for validating architecture rules following Single Responsibility Principle
#[async_trait]
pub trait ArchitectureValidationService: Send + Sync {
    async fn validate_architecture(&self, project_id: &str) -> Result<Vec<String>, McpError>;
    async fn validate_component_dependencies(
        &self,
        component: &FrameworkComponent,
    ) -> Result<Vec<String>, McpError>;
}

/// Implementation of ArchitectureValidationService
pub struct ArchitectureValidationServiceImpl<FS: FrameworkService> {
    framework_service: FS,
}

impl<FS: FrameworkService> ArchitectureValidationServiceImpl<FS> {
    pub fn new(framework_service: FS) -> Self {
        Self { framework_service }
    }

    /// Validate presentation layer dependencies (OCP - can be extended with new rules)
    fn validate_presentation_layer(&self, component: &FrameworkComponent) -> Vec<String> {
        let mut violations = Vec::new();

        // Presentation layer should not directly import from data layer
        for dep in &component.dependencies {
            if dep.contains("data/") && !dep.contains("domain/") {
                violations.push(format!(
                    "Architecture violation: {} (presentation) directly imports from data layer: {}",
                    component.component_name, dep
                ));
            }
        }

        violations
    }

    /// Validate domain layer dependencies (OCP - can be extended with new rules)
    fn validate_domain_layer(&self, component: &FrameworkComponent) -> Vec<String> {
        let mut violations = Vec::new();

        // Domain layer should not import from presentation or data layers
        for dep in &component.dependencies {
            if dep.contains("presentation/") || dep.contains("data/") {
                violations.push(format!(
                    "Architecture violation: {} (domain) imports from {}: {}",
                    component.component_name,
                    if dep.contains("presentation/") {
                        "presentation"
                    } else {
                        "data"
                    },
                    dep
                ));
            }
        }

        violations
    }

    /// Validate data layer dependencies (OCP - can be extended with new rules)
    fn validate_data_layer(&self, _component: &FrameworkComponent) -> Vec<String> {
        let violations = Vec::new();

        // Data layer validation rules would go here
        // For now, data layer has fewer restrictions

        violations
    }

    /// Validate core layer dependencies (OCP - can be extended with new rules)
    fn validate_core_layer(&self, _component: &FrameworkComponent) -> Vec<String> {
        let violations = Vec::new();

        // Core layer validation rules would go here
        // Core layer should be independent of all other layers

        violations
    }
}

#[async_trait]
impl<FS: FrameworkService> ArchitectureValidationService for ArchitectureValidationServiceImpl<FS> {
    async fn validate_architecture(&self, project_id: &str) -> Result<Vec<String>, McpError> {
        let mut violations = Vec::new();

        // Get all components for the project
        let components = self.framework_service.list_components(project_id).await?;

        // Validate each component's dependencies
        for component in &components {
            let component_violations = self.validate_component_dependencies(component).await?;
            violations.extend(component_violations);
        }

        Ok(violations)
    }

    async fn validate_component_dependencies(
        &self,
        component: &FrameworkComponent,
    ) -> Result<Vec<String>, McpError> {
        let violations = match ArchitectureLayer::from_str(&component.architecture_layer) {
            Ok(ArchitectureLayer::Presentation) => self.validate_presentation_layer(component),
            Ok(ArchitectureLayer::Domain) => self.validate_domain_layer(component),
            Ok(ArchitectureLayer::Data) => self.validate_data_layer(component),
            Ok(ArchitectureLayer::Core) => self.validate_core_layer(component),
            Err(err) => vec![err],
        };

        Ok(violations)
    }
}
