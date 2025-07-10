// Infrastructure layer - SQLite implementations of repositories

pub mod sqlite_project_repository;
pub mod sqlite_flutter_repository;
pub mod sqlite_development_phase_repository;
pub mod sqlite_business_rule_repository;
pub mod sqlite_architectural_decision_repository;
pub mod sqlite_performance_requirement_repository;
pub mod sqlite_framework_repository;
// TODO: Fix error handling in these files
// pub mod sqlite_security_policy_repository;
// pub mod sqlite_extended_repositories;

// Re-export implementations
pub use sqlite_project_repository::SqliteProjectRepository;
pub use sqlite_development_phase_repository::SqliteDevelopmentPhaseRepository;
pub use sqlite_business_rule_repository::SqliteBusinessRuleRepository;
pub use sqlite_architectural_decision_repository::SqliteArchitecturalDecisionRepository;
pub use sqlite_performance_requirement_repository::SqlitePerformanceRequirementRepository;
pub use sqlite_framework_repository::SqliteFrameworkRepository;
// TODO: Re-enable when fixed
// pub use sqlite_security_policy_repository::SqliteSecurityPolicyRepository;
// pub use sqlite_extended_repositories::{SqliteProjectConventionRepository, SqliteFeatureContextRepository};
