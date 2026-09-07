use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};

use crate::config::WebServiceConfig;
use crate::state::AppState;
use crate::tools::AadMcpServer;

pub fn create_app(state: AppState, ct: CancellationToken) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let prefix = state.config.webservice.api_prefix.trim_end_matches('/');
    let mcp_prefixed = format!("{}/v1/mcp", prefix);
    let sse_prefixed = format!("{}/v1/sse", prefix);
    let message_prefixed = format!("{}/v1/message", prefix);

    let server = state.server.clone();
    let mcp_service: StreamableHttpService<AadMcpServer, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(server.clone()),
            Default::default(),
            StreamableHttpServerConfig::default()
                .with_cancellation_token(ct.child_token()),
        );

    Router::new()
        .nest_service(&mcp_prefixed, mcp_service.clone())
        .nest_service(&sse_prefixed, mcp_service.clone())
        .nest_service(&message_prefixed, mcp_service.clone())
        .nest_service("/mcp", mcp_service.clone())
        .nest_service("/sse", mcp_service.clone())
        .nest_service("/message", mcp_service)
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
