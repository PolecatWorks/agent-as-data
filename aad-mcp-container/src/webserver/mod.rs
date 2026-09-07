pub mod rpc;
pub mod sse;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::WebServiceConfig;
use crate::state::AppState;
use self::sse::{direct_rpc_handler, message_handler, sse_handler};

pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let prefix = state.config.webservice.api_prefix.trim_end_matches('/');
    let sse_prefixed = format!("{}/v1/sse", prefix);
    let message_prefixed = format!("{}/v1/message", prefix);
    let rpc_prefixed = format!("{}/v1/rpc", prefix);

    Router::new()
        // Prefixed endpoints (e.g. /api/v1/sse)
        .route(&sse_prefixed, get(sse_handler))
        .route(&message_prefixed, post(message_handler))
        .route(&rpc_prefixed, post(direct_rpc_handler))
        // Root aliases for standard MCP client discovery (/sse, /message, /rpc)
        .route("/sse", get(sse_handler))
        .route("/message", post(message_handler))
        .route("/rpc", post(direct_rpc_handler))
        .route("/healthz", get(|| async { "ok" }))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

pub async fn start_webserver(
    state: AppState,
    config: &WebServiceConfig,
    ct: CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_app(state);
    let addr: SocketAddr = config.address.parse()
        .map_err(|e| format!("Failed to parse webserver address '{}': {}", config.address, e))?;

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
