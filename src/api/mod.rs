pub mod context_api;
pub mod specification_context_linking_tools;

use axum::{Router};
use context_api::{create_router_with_state, AppState};
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

pub fn create_router() -> Router {
    // Open DB connection and share via state
    let db = Arc::new(Mutex::new(Connection::open("context.db").unwrap()));
    let state = AppState { db };
    create_router_with_state(state)
}
