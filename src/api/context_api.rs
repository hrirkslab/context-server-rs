use axum::{extract::State, routing::{get, post}, Json, Router};
use serde_json::json;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

use crate::models::context::BusinessRule;
use crate::models::api::{ContextQuery, ContextResponse};
use crate::models::context::{
    ArchitecturalDecision, PerformanceRequirement, SecurityPolicy, ProjectConvention, FeatureContext
};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}

pub fn create_router_with_state(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/business_rules", get(list_business_rules).post(create_business_rule))
        .route("/context/query", get(get_context_query).post(context_query))
        .route("/features", get(list_features))
        .route("/architectural_decisions", get(list_architectural_decisions).post(create_architectural_decision))
        .route("/performance_requirements", get(list_performance_requirements).post(create_performance_requirement))
        .route("/security_policies", get(list_security_policies).post(create_security_policy))
        .route("/project_conventions", get(list_project_conventions).post(create_project_convention))
        .route("/feature_contexts", get(list_feature_contexts).post(create_feature_context))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_business_rules(State(state): State<AppState>) -> Json<Vec<BusinessRule>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at FROM business_rules").unwrap();
    let rules = stmt
        .query_map([], |row| {
            Ok(BusinessRule {
                id: row.get(0)?,
                rule_name: row.get(1)?,
                description: row.get(2).ok(),
                domain_area: row.get(3).ok(),
                implementation_pattern: row.get(4).ok(),
                constraints: row.get(5).ok(),
                examples: row.get(6).ok(),
                created_at: row.get(7).ok(),
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
        "INSERT INTO business_rules (id, rule_name, description, domain_area, implementation_pattern, constraints, examples, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            rule.id,
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
    maybe_query: Result<axum::Json<crate::models::api::ContextQuery>, axum::extract::rejection::JsonRejection>
) -> Result<axum::Json<crate::models::api::ContextResponse>, axum::http::StatusCode> {
    // If the body is missing or invalid, just return an empty/default response
    let _query = match maybe_query {
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

    // ...existing logic for valid queries...
    Ok(axum::Json(crate::models::api::ContextResponse {
        business_rules: vec![],
        architectural_guidance: vec![],
        performance_requirements: vec![],
        security_policies: vec![],
        conventions: vec![],
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

// Add GET handler for /context/query for Copilot Agent compatibility
pub async fn get_context_query(State(_state): State<AppState>) -> Json<ContextResponse> {
    // You can enhance this to return real data if desired
    Json(ContextResponse {
        business_rules: vec![],
        architectural_guidance: vec![],
        performance_requirements: vec![],
        security_policies: vec![],
        conventions: vec![],
    })
}

// Architectural Decisions CRUD
async fn list_architectural_decisions(State(state): State<AppState>) -> Json<Vec<ArchitecturalDecision>> {
    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, decision_title, context, decision, consequences, alternatives_considered, status, created_at FROM architectural_decisions").unwrap();
    let items = stmt
        .query_map([], |row| {
            Ok(ArchitecturalDecision {
                id: row.get(0)?,
                decision_title: row.get(1)?,
                context: row.get(2).ok(),
                decision: row.get(3).ok(),
                consequences: row.get(4).ok(),
                alternatives_considered: row.get(5).ok(),
                status: row.get(6).ok(),
                created_at: row.get(7).ok(),
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
        "INSERT INTO architectural_decisions (id, decision_title, context, decision, consequences, alternatives_considered, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            item.id,
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
    let mut stmt = db.prepare("SELECT id, component_area, requirement_type, target_value, optimization_patterns, avoid_patterns, created_at FROM performance_requirements").unwrap();
    let items = stmt
        .query_map([], |row| {
            Ok(PerformanceRequirement {
                id: row.get(0)?,
                component_area: row.get(1).ok(),
                requirement_type: row.get(2).ok(),
                target_value: row.get(3).ok(),
                optimization_patterns: row.get(4).ok(),
                avoid_patterns: row.get(5).ok(),
                created_at: row.get(6).ok(),
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
        "INSERT INTO performance_requirements (id, component_area, requirement_type, target_value, optimization_patterns, avoid_patterns, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            item.id,
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
    let mut stmt = db.prepare("SELECT id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at FROM security_policies").unwrap();
    let items = stmt
        .query_map([], |row| {
            Ok(SecurityPolicy {
                id: row.get(0)?,
                policy_name: row.get(1)?,
                policy_area: row.get(2).ok(),
                requirements: row.get(3).ok(),
                implementation_pattern: row.get(4).ok(),
                forbidden_patterns: row.get(5).ok(),
                compliance_notes: row.get(6).ok(),
                created_at: row.get(7).ok(),
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
        "INSERT INTO security_policies (id, policy_name, policy_area, requirements, implementation_pattern, forbidden_patterns, compliance_notes, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            item.id,
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
    let mut stmt = db.prepare("SELECT id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at FROM project_conventions").unwrap();
    let items = stmt
        .query_map([], |row| {
            Ok(ProjectConvention {
                id: row.get(0)?,
                convention_type: row.get(1).ok(),
                convention_rule: row.get(2).ok(),
                good_examples: row.get(3).ok(),
                bad_examples: row.get(4).ok(),
                rationale: row.get(5).ok(),
                created_at: row.get(6).ok(),
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
        "INSERT INTO project_conventions (id, convention_type, convention_rule, good_examples, bad_examples, rationale, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            item.id,
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
    let mut stmt = db.prepare("SELECT id, feature_name, business_purpose, user_personas, key_workflows, integration_points, edge_cases, created_at FROM feature_context").unwrap();
    let items = stmt
        .query_map([], |row| {
            Ok(FeatureContext {
                id: row.get(0)?,
                feature_name: row.get(1)?,
                business_purpose: row.get(2).ok(),
                user_personas: row.get(3).ok(),
                key_workflows: row.get(4).ok(),
                integration_points: row.get(5).ok(),
                edge_cases: row.get(6).ok(),
                created_at: row.get(7).ok(),
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
        "INSERT INTO feature_context (id, feature_name, business_purpose, user_personas, key_workflows, integration_points, edge_cases, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            item.id,
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
