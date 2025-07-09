// Service layer modules following SOLID principles

pub mod project_service;
pub mod flutter_service;
pub mod development_phase_service;
pub mod context_query_service;
pub mod architecture_validation_service;

// Re-export service traits
pub use project_service::ProjectService;
pub use flutter_service::FlutterService;
pub use development_phase_service::DevelopmentPhaseService;
pub use context_query_service::ContextQueryService;
pub use architecture_validation_service::ArchitectureValidationService;
