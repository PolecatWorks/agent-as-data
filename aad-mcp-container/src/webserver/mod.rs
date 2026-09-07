use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::WebServiceConfig;
use crate::state::AppState;

pub mod rpc;

pub fn create_app(state: AppState, _ct: CancellationToken) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let prefix = state.config.webservice.api_prefix.trim_end_matches('/');
    let mcp_prefixed = format!("{}/v1/mcp", prefix);

    Router::new()
        .route(&mcp_prefixed, post(handle_http_rpc))
        .route("/mcp", post(handle_http_rpc))
        .route("/", post(handle_http_rpc))
        .route("/healthz", get(|| async { "ok" }))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

async fn handle_http_rpc(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let response = rpc::handle_json_rpc(&state, payload).await;
    match response {
        Some(resp) => (StatusCode::OK, Json(resp)).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

pub async fn start_webserver(
    state: AppState,
    config: &WebServiceConfig,
    ct: CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = config.address.parse()
        .map_err(|e| format!("Failed to parse webserver address '{}': {}", config.address, e))?;

    let app = create_app(state, ct.clone());

    info!("Starting MCP server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            ct.cancelled().await;
            info!("Webserver received cancellation signal, shutting down gracefully.");
        })
        .await?;

    Ok(())
}
