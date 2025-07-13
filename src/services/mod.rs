// Service layer modules following SOLID principles

pub mod architecture_validation_service;
pub mod context_crud_service;
pub mod context_query_service;
pub mod development_phase_service;
pub mod extended_context_crud_service;
pub mod framework_service;
pub mod project_service;
// Note: component_service removed as it was identical to framework_service
// Note: flutter_service and flutter_advanced_crud_service modules don't exist yet

// Re-export service traits
pub use architecture_validation_service::ArchitectureValidationService;
pub use context_query_service::ContextQueryService;
pub use development_phase_service::DevelopmentPhaseService;
pub use framework_service::FrameworkService;
pub use project_service::ProjectService;
// Note: ComponentService removed as it was identical to FrameworkService
// The following services are currently commented out because their corresponding endpoints
// have not yet been implemented. These services will be re-enabled once the necessary
// functionality is added to the application. The expected timeline for implementation
// is tracked in the project roadmap. Please refer to the roadmap for updates.
