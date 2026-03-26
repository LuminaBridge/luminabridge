//! Server module for LuminaBridge

use axum::{
    routing::get,
    Router,
    extract::State,
    response::Json,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

use crate::config::Config;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::routes;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<Database>,
}

pub struct Server {
    pub config: Arc<Config>,
    pub db: Arc<Database>,
}

impl Server {
    pub fn new(config: Arc<Config>, db: Arc<Database>) -> Self {
        Server { config, db }
    }
    
    pub async fn run(&self) -> Result<()> {
        let state = AppState {
            config: self.config.clone(),
            db: self.db.clone(),
        };
        
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/ready", get(ready_check))
            .nest("/api/v1", routes::api_v1_routes(state.clone()))
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .with_state(state);
        
        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| Error::Server(format!("Failed to bind to {}: {}", addr, e)))?;
        
        info!("🌉 Server listening on http://{}", addr);
        
        axum::serve(listener, app)
            .await
            .map_err(|e| Error::Server(format!("Server error: {}", e)))
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "luminabridge"
    }))
}

async fn ready_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    state.db.health_check().await?;
    Ok(Json(json!({
        "status": "ready",
        "database": "connected"
    })))
}
