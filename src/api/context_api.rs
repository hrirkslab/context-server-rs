use axum::{extract::State, routing::{get, post}, Json, Router};
use serde_json::json;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

use crate::models::context::BusinessRule;
use crate::models::api::{ContextQuery, ContextResponse};
use crate::models::context::{
    ArchitecturalDecision, PerformanceRequirement, SecurityPolicy, ProjectConvention, FeatureContext, Project
};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}

pub fn create_router_with_state(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/:project_id", get(get_project).delete(delete_project))
        .route("/business_rules", get(list_business_rules).post(create_business_rule))
        .route("/business_rules/project/:project_id", get(list_business_rules_by_project))
        .route("/context/query", get(get_context_query).post(context_query))
        .route("/features", get(list_features))
        .route("/architectural_decisions", get(list_architectural_decisions).post(create_architectural_decision))
        .route("/architectural_decisions/project/:project_id", get(list_architectural_decisions_by_project))
        .route("/performance_requirements", get(list_performance_requirements).post(create_performance_requirement))
        .route("/performance_requirements/project/:project_id", get(list_performance_requirements_by_project))
        .route("/security_policies", get(list_security_policies).post(create_security_policy))
        .route("/security_policies/project/:project_id", get(list_security_policies_by_project))
        .route("/project_conventions", get(list_project_conventions).post(create_project_convention))
        .route("/project_conventions/project/:project_id", get(list_project_conventions_by_project))
        .route("/feature_contexts", get(list_feature_contexts).post(create_feature_context))
        .route("/feature_contexts/project/:project_id", get(list_feature_contexts_by_project))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_business_rules(State(state): State<AppState>) -> Json<Vec<BusinessRule>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at FROM business_rules").unwrap();
    let rules = stmt
        .query_map([], |row| {
            Ok(BusinessRule {
                id: row.get(0)?,
                project_id: row.get(1)?,
                rule_name: row.get(2)?,
                description: row.get(3).ok(),
                domain_area: row.get(4).ok(),
                implementation_pattern: row.get(5).ok(),
                constraints: row.get(6).ok(),
                examples: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(rules)
}

async fn create_business_rule(State(state): State<AppState>, Json(rule): Json<BusinessRule>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO business_rules (id, project_id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            rule.id,
            rule.project_id,
            rule.rule_name,
            rule.description,
            rule.domain_area,
            rule.implementation_pattern,
            rule.constraints,
            rule.examples,
            rule.created_at
        ],
    );
    Json(json!({"status": "ok"}))
}

// More forgiving POST handler for /context/query
pub async fn context_query(
    State(state): State<AppState>,
    maybe_query: Result<axum::Json<crate::models::api::ContextQuery>, axum::extract::rejection::JsonRejection>
) -> Result<axum::Json<crate::models::api::ContextResponse>, axum::http::StatusCode> {
    // If the body is missing or invalid, just return an empty/default response
    let query = match maybe_query {
        Ok(axum::Json(query)) => query,
        Err(_) => {
            // Log or handle the error as needed
            return Ok(axum::Json(crate::models::api::ContextResponse {
                business_rules: vec![],
                architectural_guidance: vec![],
                performance_requirements: vec![],
                security_policies: vec![],
                conventions: vec![],
            }));
        }
    };

    let db = state.db.lock().unwrap();
    
    // Query for business rules related to the feature area in this project
    let mut stmt = db.prepare("SELECT id, project_id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at FROM business_rules WHERE project_id = ?1 AND domain_area LIKE ?2").unwrap();
    let search_term = format!("%{}%", query.feature_area);
    let business_rules = stmt
        .query_map(rusqlite::params![query.project_id, search_term], |row| {
            Ok(BusinessRule {
                id: row.get(0)?,
                project_id: row.get(1)?,
                rule_name: row.get(2)?,
                description: row.get(3).ok(),
                domain_area: row.get(4).ok(),
                implementation_pattern: row.get(5).ok(),
                constraints: row.get(6).ok(),
                examples: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
        
    // Query for architectural decisions related to this project
    let mut stmt = db.prepare("SELECT id, project_id, decision_title, context, decision, consequences, alternatives_considered, status, created_at FROM architectural_decisions WHERE project_id = ?1").unwrap();
    let architectural_guidance = stmt
        .query_map(rusqlite::params![query.project_id], |row| {
            Ok(ArchitecturalDecision {
                id: row.get(0)?,
                project_id: row.get(1)?,
                decision_title: row.get(2)?,
                context: row.get(3).ok(),
                decision: row.get(4).ok(),
                consequences: row.get(5).ok(),
                alternatives_considered: row.get(6).ok(),
                status: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
        
    // Query for performance requirements related to the components in this project
    let component_search_term = format!("%{}%", query.components.join("%"));
    let mut stmt = db.prepare("SELECT id, project_id, component_area, requirement_type, target_value, optimization_patterns, avoid_patterns, created_at FROM performance_requirements WHERE project_id = ?1 AND component_area LIKE ?2").unwrap();
    let performance_requirements = stmt
        .query_map(rusqlite::params![query.project_id, component_search_term], |row| {
            Ok(PerformanceRequirement {
                id: row.get(0)?,
                project_id: row.get(1)?,
                component_area: row.get(2).ok(),
                requirement_type: row.get(3).ok(),
                target_value: row.get(4).ok(),
                optimization_patterns: row.get(5).ok(),
                avoid_patterns: row.get(6).ok(),
                created_at: row.get(7).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
        
    // Query for security policies in this project
    let mut stmt = db.prepare("SELECT id, project_id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at FROM security_policies WHERE project_id = ?1").unwrap();
    let security_policies = stmt
        .query_map(rusqlite::params![query.project_id], |row| {
            Ok(SecurityPolicy {
                id: row.get(0)?,
                project_id: row.get(1)?,
                policy_name: row.get(2)?,
                policy_area: row.get(3).ok(),
                requirements: row.get(4).ok(),
                implementation_pattern: row.get(5).ok(),
                forbidden_patterns: row.get(6).ok(),
                compliance_notes: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
        
    // Query for project conventions in this project
    let mut stmt = db.prepare("SELECT id, project_id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at FROM project_conventions WHERE project_id = ?1").unwrap();
    let conventions = stmt
        .query_map(rusqlite::params![query.project_id], |row| {
            Ok(ProjectConvention {
                id: row.get(0)?,
                project_id: row.get(1)?,
                convention_type: row.get(2).ok(),
                convention_rule: row.get(3).ok(),
                good_examples: row.get(4).ok(),
                bad_examples: row.get(5).ok(),
                rationale: row.get(6).ok(),
                created_at: row.get(7).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    
    Ok(axum::Json(crate::models::api::ContextResponse {
        business_rules,
        architectural_guidance,
        performance_requirements,
        security_policies,
        conventions,
    }))
}

// New: GET /features endpoint to list available features
pub async fn list_features() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "features": [
            "business_rules",
            "architectural_decisions",
            "performance_requirements",
            "security_policies",
            "project_conventions",
            "feature_context",
            "context_query"
        ]
    }))
}

// GET handler for /context/query?project_id={id} 
async fn get_context_query(
    State(state): State<AppState>, 
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>
) -> Result<axum::Json<crate::models::api::ContextResponse>, axum::http::StatusCode> {
    let project_id = match params.get("project_id") {
        Some(id) => id,
        None => return Err(axum::http::StatusCode::BAD_REQUEST),
    };

    let db = state.db.lock().unwrap();
    
    // Get business rules for this project
    let mut stmt = db.prepare("SELECT id, project_id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at FROM business_rules WHERE project_id = ?1").unwrap();
    let business_rules = stmt
        .query_map([project_id], |row| {
            Ok(BusinessRule {
                id: row.get(0)?,
                project_id: row.get(1)?,
                rule_name: row.get(2)?,
                description: row.get(3).ok(),
                domain_area: row.get(4).ok(),
                implementation_pattern: row.get(5).ok(),
                constraints: row.get(6).ok(),
                examples: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
        
    // Get architectural decisions for this project
    let mut stmt = db.prepare("SELECT id, project_id, decision_title, context, decision, consequences, alternatives_considered, status, created_at FROM architectural_decisions WHERE project_id = ?1").unwrap();
    let architectural_guidance = stmt
        .query_map([project_id], |row| {
            Ok(ArchitecturalDecision {
                id: row.get(0)?,
                project_id: row.get(1)?,
                decision_title: row.get(2)?,
                context: row.get(3).ok(),
                decision: row.get(4).ok(),
                consequences: row.get(5).ok(),
                alternatives_considered: row.get(6).ok(),
                status: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
        
    // Get performance requirements for this project
    let mut stmt = db.prepare("SELECT id, project_id, component_area, requirement_type, target_value, optimization_patterns, avoid_patterns, created_at FROM performance_requirements WHERE project_id = ?1").unwrap();
    let performance_requirements = stmt
        .query_map([project_id], |row| {
            Ok(PerformanceRequirement {
                id: row.get(0)?,
                project_id: row.get(1)?,
                component_area: row.get(2).ok(),
                requirement_type: row.get(3).ok(),
                target_value: row.get(4).ok(),
                optimization_patterns: row.get(5).ok(),
                avoid_patterns: row.get(6).ok(),
                created_at: row.get(7).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
        
    // Get security policies for this project
    let mut stmt = db.prepare("SELECT id, project_id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at FROM security_policies WHERE project_id = ?1").unwrap();
    let security_policies = stmt
        .query_map([project_id], |row| {
            Ok(SecurityPolicy {
                id: row.get(0)?,
                project_id: row.get(1)?,
                policy_name: row.get(2)?,
                policy_area: row.get(3).ok(),
                requirements: row.get(4).ok(),
                implementation_pattern: row.get(5).ok(),
                forbidden_patterns: row.get(6).ok(),
                compliance_notes: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
        
    // Get project conventions for this project
    let mut stmt = db.prepare("SELECT id, project_id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at FROM project_conventions WHERE project_id = ?1").unwrap();
    let conventions = stmt
        .query_map([project_id], |row| {
            Ok(ProjectConvention {
                id: row.get(0)?,
                project_id: row.get(1)?,
                convention_type: row.get(2).ok(),
                convention_rule: row.get(3).ok(),
                good_examples: row.get(4).ok(),
                bad_examples: row.get(5).ok(),
                rationale: row.get(6).ok(),
                created_at: row.get(7).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    
    Ok(axum::Json(crate::models::api::ContextResponse {
        business_rules,
        architectural_guidance,
        performance_requirements,
        security_policies,
        conventions,
    }))
}

// Architectural Decisions CRUD
async fn list_architectural_decisions(State(state): State<AppState>) -> Json<Vec<ArchitecturalDecision>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, decision_title, context, decision, consequences, alternatives_considered, status, created_at FROM architectural_decisions").unwrap();
    let items = stmt
        .query_map([], |row| {
            Ok(ArchitecturalDecision {
                id: row.get(0)?,
                project_id: row.get(1)?,
                decision_title: row.get(2)?,
                context: row.get(3).ok(),
                decision: row.get(4).ok(),
                consequences: row.get(5).ok(),
                alternatives_considered: row.get(6).ok(),
                status: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(items)
}

async fn create_architectural_decision(State(state): State<AppState>, Json(item): Json<ArchitecturalDecision>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO architectural_decisions (id, project_id, decision_title, context, decision, consequences, alternatives_considered, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            item.id,
            item.project_id,
            item.decision_title,
            item.context,
            item.decision,
            item.consequences,
            item.alternatives_considered,
            item.status,
            item.created_at
        ]
    );
    Json(json!({"status": "ok"}))
}

// Performance Requirements CRUD (corrected)
async fn list_performance_requirements(State(state): State<AppState>) -> Json<Vec<PerformanceRequirement>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, component_area, requirement_type, target_value, optimization_patterns, avoid_patterns, created_at FROM performance_requirements").unwrap();
    let items = stmt
        .query_map([], |row| {
            Ok(PerformanceRequirement {
                id: row.get(0)?,
                project_id: row.get(1)?,
                component_area: row.get(2).ok(),
                requirement_type: row.get(3).ok(),
                target_value: row.get(4).ok(),
                optimization_patterns: row.get(5).ok(),
                avoid_patterns: row.get(6).ok(),
                created_at: row.get(7).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(items)
}

async fn create_performance_requirement(State(state): State<AppState>, Json(item): Json<PerformanceRequirement>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO performance_requirements (id, project_id, component_area, requirement_type, target_value, optimization_patterns, avoid_patterns, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            item.id,
            item.project_id,
            item.component_area,
            item.requirement_type,
            item.target_value,
            item.optimization_patterns,
            item.avoid_patterns,
            item.created_at
        ]
    );
    Json(json!({"status": "ok"}))
}

// Security Policies CRUD (corrected)
async fn list_security_policies(State(state): State<AppState>) -> Json<Vec<SecurityPolicy>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at FROM security_policies").unwrap();
    let items = stmt
        .query_map([], |row| {
            Ok(SecurityPolicy {
                id: row.get(0)?,
                project_id: row.get(1)?,
                policy_name: row.get(2)?,
                policy_area: row.get(3).ok(),
                requirements: row.get(4).ok(),
                implementation_pattern: row.get(5).ok(),
                forbidden_patterns: row.get(6).ok(),
                compliance_notes: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(items)
}

async fn create_security_policy(State(state): State<AppState>, Json(item): Json<SecurityPolicy>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO security_policies (id, project_id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            item.id,
            item.project_id,
            item.policy_name,
            item.policy_area,
            item.requirements,
            item.implementation_pattern,
            item.forbidden_patterns,
            item.compliance_notes,
            item.created_at
        ]
    );
    Json(json!({"status": "ok"}))
}

// Project Conventions CRUD (corrected)
async fn list_project_conventions(State(state): State<AppState>) -> Json<Vec<ProjectConvention>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at FROM project_conventions").unwrap();
    let items = stmt
        .query_map([], |row| {
            Ok(ProjectConvention {
                id: row.get(0)?,
                project_id: row.get(1)?,
                convention_type: row.get(2).ok(),
                convention_rule: row.get(3).ok(),
                good_examples: row.get(4).ok(),
                bad_examples: row.get(5).ok(),
                rationale: row.get(6).ok(),
                created_at: row.get(7).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(items)
}

async fn create_project_convention(State(state): State<AppState>, Json(item): Json<ProjectConvention>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO project_conventions (id, project_id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            item.id,
            item.project_id,
            item.convention_type,
            item.convention_rule,
            item.good_examples,
            item.bad_examples,
            item.rationale,
            item.created_at
        ]
    );
    Json(json!({"status": "ok"}))
}

// Feature Context CRUD (corrected)
async fn list_feature_contexts(State(state): State<AppState>) -> Json<Vec<FeatureContext>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, feature_name, business_purpose, user_personas, key_workflows, integration_points, edge_cases, created_at FROM feature_context").unwrap();
    let items = stmt
        .query_map([], |row| {
            Ok(FeatureContext {
                id: row.get(0)?,
                project_id: row.get(1)?,
                feature_name: row.get(2)?,
                business_purpose: row.get(3).ok(),
                user_personas: row.get(4).ok(),
                key_workflows: row.get(5).ok(),
                integration_points: row.get(6).ok(),
                edge_cases: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(items)
}

async fn create_feature_context(State(state): State<AppState>, Json(item): Json<FeatureContext>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO feature_context (id, project_id, feature_name, business_purpose, user_personas, key_workflows, integration_points, edge_cases, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            item.id,
            item.project_id,
            item.feature_name,
            item.business_purpose,
            item.user_personas,
            item.key_workflows,
            item.integration_points,
            item.edge_cases,
            item.created_at
        ]
    );
    Json(json!({"status": "ok"}))
}

// Project management handlers
async fn list_projects(State(state): State<AppState>) -> Json<Vec<Project>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, name, description, repository_url, created_at, updated_at FROM projects").unwrap();
    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2).ok(),
                repository_url: row.get(3).ok(),
                created_at: row.get(4).ok(),
                updated_at: row.get(5).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(projects)
}

async fn create_project(State(state): State<AppState>, Json(project): Json<Project>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO projects (id, name, description, repository_url, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            project.id,
            project.name,
            project.description,
            project.repository_url,
            project.created_at,
            project.updated_at
        ],
    );
    Json(json!({"status": "ok"}))
}

async fn get_project(State(state): State<AppState>, axum::extract::Path(project_id): axum::extract::Path<String>) -> Json<Option<Project>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, name, description, repository_url, created_at, updated_at FROM projects WHERE id = ?1").unwrap();
    let project = stmt
        .query_map([project_id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2).ok(),
                repository_url: row.get(3).ok(),
                created_at: row.get(4).ok(),
                updated_at: row.get(5).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .next();
    Json(project)
}

async fn delete_project(State(state): State<AppState>, axum::extract::Path(project_id): axum::extract::Path<String>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();
    let _ = db.execute("DELETE FROM projects WHERE id = ?1", [project_id]);
    Json(json!({"status": "ok"}))
}

// Project-specific list handlers
async fn list_business_rules_by_project(
    State(state): State<AppState>, 
    axum::extract::Path(project_id): axum::extract::Path<String>
) -> Json<Vec<BusinessRule>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at FROM business_rules WHERE project_id = ?1").unwrap();
    let rules = stmt
        .query_map([project_id], |row| {
            Ok(BusinessRule {
                id: row.get(0)?,
                project_id: row.get(1)?,
                rule_name: row.get(2)?,
                description: row.get(3).ok(),
                domain_area: row.get(4).ok(),
                implementation_pattern: row.get(5).ok(),
                constraints: row.get(6).ok(),
                examples: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(rules)
}

async fn list_architectural_decisions_by_project(
    State(state): State<AppState>, 
    axum::extract::Path(project_id): axum::extract::Path<String>
) -> Json<Vec<ArchitecturalDecision>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, decision_title, context, decision, consequences, alternatives_considered, status, created_at FROM architectural_decisions WHERE project_id = ?1").unwrap();
    let decisions = stmt
        .query_map([project_id], |row| {
            Ok(ArchitecturalDecision {
                id: row.get(0)?,
                project_id: row.get(1)?,
                decision_title: row.get(2)?,
                context: row.get(3).ok(),
                decision: row.get(4).ok(),
                consequences: row.get(5).ok(),
                alternatives_considered: row.get(6).ok(),
                status: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(decisions)
}

async fn list_performance_requirements_by_project(
    State(state): State<AppState>, 
    axum::extract::Path(project_id): axum::extract::Path<String>
) -> Json<Vec<PerformanceRequirement>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, component_area, requirement_type, target_value, optimization_patterns, avoid_patterns, created_at FROM performance_requirements WHERE project_id = ?1").unwrap();
    let requirements = stmt
        .query_map([project_id], |row| {
            Ok(PerformanceRequirement {
                id: row.get(0)?,
                project_id: row.get(1)?,
                component_area: row.get(2).ok(),
                requirement_type: row.get(3).ok(),
                target_value: row.get(4).ok(),
                optimization_patterns: row.get(5).ok(),
                avoid_patterns: row.get(6).ok(),
                created_at: row.get(7).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(requirements)
}

async fn list_security_policies_by_project(
    State(state): State<AppState>, 
    axum::extract::Path(project_id): axum::extract::Path<String>
) -> Json<Vec<SecurityPolicy>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at FROM security_policies WHERE project_id = ?1").unwrap();
    let policies = stmt
        .query_map([project_id], |row| {
            Ok(SecurityPolicy {
                id: row.get(0)?,
                project_id: row.get(1)?,
                policy_name: row.get(2)?,
                policy_area: row.get(3).ok(),
                requirements: row.get(4).ok(),
                implementation_pattern: row.get(5).ok(),
                forbidden_patterns: row.get(6).ok(),
                compliance_notes: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(policies)
}

async fn list_project_conventions_by_project(
    State(state): State<AppState>, 
    axum::extract::Path(project_id): axum::extract::Path<String>
) -> Json<Vec<ProjectConvention>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at FROM project_conventions WHERE project_id = ?1").unwrap();
    let conventions = stmt
        .query_map([project_id], |row| {
            Ok(ProjectConvention {
                id: row.get(0)?,
                project_id: row.get(1)?,
                convention_type: row.get(2).ok(),
                convention_rule: row.get(3).ok(),
                good_examples: row.get(4).ok(),
                bad_examples: row.get(5).ok(),
                rationale: row.get(6).ok(),
                created_at: row.get(7).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(conventions)
}

async fn list_feature_contexts_by_project(
    State(state): State<AppState>, 
    axum::extract::Path(project_id): axum::extract::Path<String>
) -> Json<Vec<FeatureContext>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, project_id, feature_name, business_purpose, user_personas, key_workflows, integration_points, edge_cases, created_at FROM feature_context WHERE project_id = ?1").unwrap();
    let features = stmt
        .query_map([project_id], |row| {
            Ok(FeatureContext {
                id: row.get(0)?,
                project_id: row.get(1)?,
                feature_name: row.get(2)?,
                business_purpose: row.get(3).ok(),
                user_personas: row.get(4).ok(),
                key_workflows: row.get(5).ok(),
                integration_points: row.get(6).ok(),
                edge_cases: row.get(7).ok(),
                created_at: row.get(8).ok(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    Json(features)
}
