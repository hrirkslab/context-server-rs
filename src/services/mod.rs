// Service layer modules following SOLID principles

pub mod advanced_query_service;
pub mod architecture_validation_service;
pub mod context_crud_service;
pub mod context_intelligence_service;
pub mod context_quality_service;
pub mod context_query_service;
pub mod context_relationship_engine;
pub mod development_phase_service;
pub mod embedding_service;
pub mod extended_context_crud_service;
pub mod framework_service;
pub mod project_service;
pub mod semantic_search_service;
pub mod hybrid_search_service;
pub mod search_index_manager;
pub mod vector_embedding_integration;
pub mod websocket_manager;
pub mod websocket_server;
pub mod websocket_types;
pub mod change_broadcaster;
pub mod change_detection_service;
pub mod sync_engine;
pub mod conflict_resolution_engine;
pub mod conflict_resolution_ui;
// #[cfg(test)]
// pub mod advanced_query_service_test;
#[cfg(test)]
pub mod advanced_query_service_simple_test;
#[cfg(test)]
pub mod semantic_search_integration_test;
#[cfg(test)]
pub mod search_index_manager_test;
#[cfg(test)]
pub mod websocket_manager_test;
#[cfg(test)]
pub mod change_broadcaster_test;
#[cfg(test)]
pub mod change_broadcasting_integration_test;
#[cfg(test)]
pub mod conflict_resolution_integration_test;
// Note: component_service removed as it was identical to framework_service
// Note: flutter_service and flutter_advanced_crud_service modules don't exist yet

// Re-export service traits
// Temporarily commented out to debug compilation issues
// pub use advanced_query_service::AdvancedQueryConfig;
pub use architecture_validation_service::ArchitectureValidationService;
pub use context_intelligence_service::{ContextIntelligenceService, DefaultContextIntelligenceService};
pub use context_quality_service::{ContextQualityService, DefaultContextQualityService};
pub use context_query_service::ContextQueryService;
pub use context_relationship_engine::{ContextRelationshipEngine, DefaultContextRelationshipEngine};
pub use development_phase_service::DevelopmentPhaseService;
pub use embedding_service::{EmbeddingService, EmbeddingServiceFactory};
pub use framework_service::FrameworkService;
pub use project_service::ProjectService;
pub use semantic_search_service::SemanticSearchService;
pub use hybrid_search_service::{HybridSearchService, HybridSearchServiceImpl};
pub use search_index_manager::{SearchIndexManager, SearchIndexManagerImpl, IndexManagerConfig};
pub use websocket_manager::WebSocketManager;
pub use websocket_server::{WebSocketServer, WebSocketService, WebSocketConfig};
pub use websocket_types::*;
pub use change_broadcaster::{ChangeBroadcaster, ChangeEvent, BroadcastMetrics, QueuedChange};
pub use change_detection_service::{ChangeDetectionService, ChangeEmitter};
pub use sync_engine::{SyncEngine, SyncStream, SyncConflict, Resolution};
pub use conflict_resolution_engine::{ConflictResolutionEngine, ConflictInfo, ConflictType, ManualResolutionRequest, ConflictResolutionResult};
pub use conflict_resolution_ui::{ConflictResolutionUI, ConflictResolutionSession, StartResolutionRequest, StartResolutionResponse, UpdateUIStateRequest, UpdateUIStateResponse};
// Note: ComponentService removed as it was identical to FrameworkService
// The following services are currently commented out because their corresponding endpoints
// have not yet been implemented. These services will be re-enabled once the necessary
// functionality is added to the application. The expected timeline for implementation
// is tracked in the project roadmap. Please refer to the roadmap for updates.
