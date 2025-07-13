// Implementation of the ComponentService trait
use async_trait::async_trait;
use crate::models::framework::FrameworkComponent;
use crate::repositories::ComponentRepository;
use rmcp::model::ErrorData as McpError;
use uuid::Uuid;
use super::component_service::{ComponentService, ComponentServiceImpl};

// Re-export the ComponentServiceImpl for use in container.rs
pub use super::component_service::ComponentServiceImpl;
