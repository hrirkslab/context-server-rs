// Repository layer interfaces following Dependency Inversion Principle

pub mod project_repository;
pub mod flutter_repository;
pub mod development_phase_repository;
pub mod business_rule_repository;
pub mod architectural_decision_repository;
pub mod performance_requirement_repository;
pub mod security_policy_repository;
pub mod project_convention_repository;
pub mod feature_context_repository;
pub mod extended_repositories;

// Re-export repository traits
pub use project_repository::ProjectRepository;
pub use flutter_repository::FlutterRepository;
pub use development_phase_repository::DevelopmentPhaseRepository;
pub use business_rule_repository::BusinessRuleRepository;
pub use architectural_decision_repository::ArchitecturalDecisionRepository;
pub use performance_requirement_repository::PerformanceRequirementRepository;
pub use security_policy_repository::SecurityPolicyRepository;
pub use project_convention_repository::ProjectConventionRepository;
pub use feature_context_repository::FeatureContextRepository;
pub use extended_repositories::{
    PrivacyRuleRepository, PrivacyViolationRepository, ArchitectureLayerRepository,
    ModelContextRepository, CodeTemplateRepository
};
