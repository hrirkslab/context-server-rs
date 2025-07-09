// Repository layer interfaces following Dependency Inversion Principle

pub mod project_repository;
pub mod flutter_repository;
pub mod development_phase_repository;
pub mod business_rule_repository;
pub mod architectural_decision_repository;
pub mod performance_requirement_repository;

// Re-export repository traits
pub use project_repository::ProjectRepository;
pub use flutter_repository::FlutterRepository;
pub use development_phase_repository::DevelopmentPhaseRepository;
pub use business_rule_repository::BusinessRuleRepository;
pub use architectural_decision_repository::ArchitecturalDecisionRepository;
pub use performance_requirement_repository::PerformanceRequirementRepository;
