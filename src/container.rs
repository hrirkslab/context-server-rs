// Dependency Injection Container following SOLID principles
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use anyhow::Result;

// Infrastructure layer
use crate::infrastructure::{
    SqliteProjectRepository,
    SqliteDevelopmentPhaseRepository,
    SqliteBusinessRuleRepository,
    SqliteArchitecturalDecisionRepository,
    SqlitePerformanceRequirementRepository,
    SqliteFrameworkRepository,
};

// Service layer
use crate::services::{
    ProjectService, 
    project_service::ProjectServiceImpl,
    DevelopmentPhaseService, 
    development_phase_service::DevelopmentPhaseServiceImpl,
    ContextQueryService, 
    context_query_service::ContextQueryServiceImpl,
    ArchitectureValidationService, 
    architecture_validation_service::ArchitectureValidationServiceImpl,
    context_crud_service::{ContextCrudService, ContextCrudServiceImpl},
    FrameworkService,
    framework_service::FrameworkServiceImpl,
};

/// Application container holding all dependencies
pub struct AppContainer {
    // Services (following Dependency Inversion Principle)
    pub project_service: Box<dyn ProjectService>,
    #[allow(dead_code)]
    pub development_phase_service: Box<dyn DevelopmentPhaseService>,
    pub context_query_service: Box<dyn ContextQueryService>,
    pub architecture_validation_service: Box<dyn ArchitectureValidationService>,
    pub context_crud_service: Box<dyn ContextCrudService>,
    pub framework_service: Box<dyn FrameworkService>,
}

impl AppContainer {
    /// Create a new application container with all dependencies injected
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Arc::new(Mutex::new(conn));

        // Create repositories (infrastructure layer)
        let project_repository = SqliteProjectRepository::new(db.clone());
        let development_phase_repository = SqliteDevelopmentPhaseRepository::new(db.clone());
        let business_rule_repository = SqliteBusinessRuleRepository::new(db.clone());
        let architectural_decision_repository = SqliteArchitecturalDecisionRepository::new(db.clone());
        let performance_requirement_repository = SqlitePerformanceRequirementRepository::new(db.clone());

        // Create services (application layer) - dependency injection
        let project_service = Box::new(ProjectServiceImpl::new(project_repository));
        
        let development_phase_service = Box::new(DevelopmentPhaseServiceImpl::new(development_phase_repository));
        
        let context_query_service = Box::new(ContextQueryServiceImpl::new(
            business_rule_repository,
            architectural_decision_repository,
            performance_requirement_repository,
        ));
        
        // Create framework service for architecture validation
        // Note: In a real application, you might want to use Arc<dyn FrameworkService> instead
        let framework_repository_for_validation = SqliteFrameworkRepository::new(db.clone());
        let framework_service_for_validation = FrameworkServiceImpl::new(framework_repository_for_validation);
        let architecture_validation_service = Box::new(ArchitectureValidationServiceImpl::new(framework_service_for_validation));

        // Create CRUD services with their repositories
        let context_crud_service = Box::new(ContextCrudServiceImpl::new(
            SqliteBusinessRuleRepository::new(db.clone()),
            SqliteArchitecturalDecisionRepository::new(db.clone()),
            SqlitePerformanceRequirementRepository::new(db.clone()),
        ));

        // Create framework service
        let framework_repository = SqliteFrameworkRepository::new(db.clone());
        let framework_service = Box::new(FrameworkServiceImpl::new(framework_repository));

        Ok(AppContainer {
            project_service,
            development_phase_service,
            context_query_service,
            architecture_validation_service,
            context_crud_service,
            framework_service,
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
