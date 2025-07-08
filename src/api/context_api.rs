use axum::{extract::State, routing::{get, post}, Json, Router};
use serde_json::json;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

use crate::models::context::BusinessRule;
use crate::models::api::{ContextQuery, ContextResponse};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}

pub fn create_router_with_state(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/business_rules", get(list_business_rules).post(create_business_rule))
        .route("/context/query", post(context_query))
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

async fn context_query(Json(_query): Json<ContextQuery>) -> Json<ContextResponse> {
    // TODO: Implement actual query logic
    Json(ContextResponse {
        business_rules: vec![],
        architectural_guidance: vec![],
        performance_requirements: vec![],
        security_policies: vec![],
        conventions: vec![],
    })
}
