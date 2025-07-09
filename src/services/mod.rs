// Service layer modules following SOLID principles

pub mod project_service;
pub mod flutter_service;
pub mod development_phase_service;
pub mod context_query_service;
pub mod architecture_validation_service;
pub mod context_crud_service;
pub mod extended_context_crud_service;
pub mod flutter_advanced_crud_service;

// Re-export service traits
pub use project_service::ProjectService;
pub use flutter_service::FlutterService;
pub use development_phase_service::DevelopmentPhaseService;
pub use context_query_service::ContextQueryService;
pub use architecture_validation_service::ArchitectureValidationService;
// The following services are currently commented out because their corresponding endpoints
// have not yet been implemented. These services will be re-enabled once the necessary
// functionality is added to the application. The expected timeline for implementation
// is tracked in the project roadmap. Please refer to the roadmap for updates.
