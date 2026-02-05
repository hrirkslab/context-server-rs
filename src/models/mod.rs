pub mod api;
pub mod architecture;
pub mod audit_log;
pub mod constraint;
pub mod context;
pub mod context_conversion;
pub mod development;
pub mod embedding;
pub mod enhanced_context;
pub mod flutter;
pub mod framework;
pub mod plugin;
pub mod specification;
pub mod tagging;

// Re-export commonly used types
pub use audit_log::{AuditEventType, AuditTrail};
pub use constraint::{ComponentDependency, Constraint, ConstraintType, DependencyType};
pub use tagging::{ContextTag, TaggedEntity};
