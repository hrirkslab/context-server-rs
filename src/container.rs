// Dependency Injection Container following SOLID principles
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use anyhow::Result;

// Infrastructure layer
use crate::infrastructure::{
    SqliteProjectRepository,
    SqliteFlutterRepository,
    SqliteDevelopmentPhaseRepository,
    SqliteBusinessRuleRepository,
    SqliteArchitecturalDecisionRepository,
    SqlitePerformanceRequirementRepository,
};

// Service layer
use crate::services::{
    ProjectService, 
    project_service::ProjectServiceImpl,
    FlutterService, 
    flutter_service::FlutterServiceImpl,
    DevelopmentPhaseService, 
    development_phase_service::DevelopmentPhaseServiceImpl,
    ContextQueryService, 
    context_query_service::ContextQueryServiceImpl,
    ArchitectureValidationService, 
    architecture_validation_service::ArchitectureValidationServiceImpl,
    context_crud_service::{ContextCrudService, ContextCrudServiceImpl},
};

/// Application container holding all dependencies
pub struct AppContainer {
    // Services (following Dependency Inversion Principle)
    pub project_service: Box<dyn ProjectService>,
    pub flutter_service: Box<dyn FlutterService>,
    #[allow(dead_code)]
    pub development_phase_service: Box<dyn DevelopmentPhaseService>,
    pub context_query_service: Box<dyn ContextQueryService>,
    pub architecture_validation_service: Box<dyn ArchitectureValidationService>,
    pub context_crud_service: Box<dyn ContextCrudService>,
}

impl AppContainer {
    /// Create a new application container with all dependencies injected
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Arc::new(Mutex::new(conn));

        // Create repositories (infrastructure layer)
        let project_repository = SqliteProjectRepository::new(db.clone());
        let flutter_repository = SqliteFlutterRepository::new(db.clone());
        let development_phase_repository = SqliteDevelopmentPhaseRepository::new(db.clone());
        let business_rule_repository = SqliteBusinessRuleRepository::new(db.clone());
        let architectural_decision_repository = SqliteArchitecturalDecisionRepository::new(db.clone());
        let performance_requirement_repository = SqlitePerformanceRequirementRepository::new(db.clone());

        // Create services (application layer) - dependency injection
        let project_service = Box::new(ProjectServiceImpl::new(project_repository));
        
        let flutter_service = Box::new(FlutterServiceImpl::new(flutter_repository));
        
        let development_phase_service = Box::new(DevelopmentPhaseServiceImpl::new(development_phase_repository));
        
        let context_query_service = Box::new(ContextQueryServiceImpl::new(
            business_rule_repository,
            architectural_decision_repository,
            performance_requirement_repository,
        ));
        
        // Create a clone of flutter_service for architecture validation
        // Note: In a real application, you might want to use Arc<dyn FlutterService> instead
        let flutter_repository_for_validation = SqliteFlutterRepository::new(db.clone());
        let flutter_service_for_validation = FlutterServiceImpl::new(flutter_repository_for_validation);
        let architecture_validation_service = Box::new(ArchitectureValidationServiceImpl::new(flutter_service_for_validation));

        // Create CRUD services with their repositories
        let context_crud_service = Box::new(ContextCrudServiceImpl::new(
            SqliteBusinessRuleRepository::new(db.clone()),
            SqliteArchitecturalDecisionRepository::new(db.clone()),
            SqlitePerformanceRequirementRepository::new(db.clone()),
        ));

        Ok(AppContainer {
            project_service,
            flutter_service,
            development_phase_service,
            context_query_service,
            architecture_validation_service,
            context_crud_service,
        })
    }
}

/// Factory pattern for creating the container with proper error handling
#[allow(dead_code)]
pub struct ContainerFactory;

impl ContainerFactory {
    #[allow(dead_code)]
    pub fn create(db_path: &str) -> Result<AppContainer> {
        AppContainer::new(db_path)
    }
}
