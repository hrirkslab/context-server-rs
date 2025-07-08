mod db;
mod api;

use db::init::init_db;
use api::create_router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Initialize SQLite DB
    let db_path = "context.db";
    match init_db(db_path) {
        Ok(_) => println!("Database initialized at {}", db_path),
        Err(e) => eprintln!("Failed to initialize DB: {}", e),
    }
    // Start Axum server (Axum 0.7)
    let app = create_router();
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("Server running at http://{}/", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
