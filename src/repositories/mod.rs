// Repository layer interfaces following Dependency Inversion Principle

pub mod architectural_decision_repository;
pub mod business_rule_repository;
pub mod development_phase_repository;
pub mod embedding_repository;
pub mod enhanced_context_repository;
pub mod extended_repositories;
pub mod feature_context_repository;
pub mod framework_repository;
pub mod performance_requirement_repository;
pub mod project_convention_repository;
pub mod project_repository;
pub mod security_policy_repository;
pub mod specification_repository;
// Note: component_repository was removed as it was identical to framework_repository

// Re-export repository traits
pub use architectural_decision_repository::ArchitecturalDecisionRepository;
pub use business_rule_repository::BusinessRuleRepository;
pub use development_phase_repository::DevelopmentPhaseRepository;
pub use embedding_repository::EmbeddingRepository;
pub use enhanced_context_repository::EnhancedContextRepository;
pub use feature_context_repository::FeatureContextRepository;
pub use performance_requirement_repository::PerformanceRequirementRepository;
pub use project_convention_repository::ProjectConventionRepository;
pub use project_repository::ProjectRepository;
pub use security_policy_repository::SecurityPolicyRepository;
pub use specification_repository::SpecificationRepository;
// pub use extended_repositories::{}; // Uncomment when needed
pub use framework_repository::FrameworkRepository;
