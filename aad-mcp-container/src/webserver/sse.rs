use std::convert::Infallible;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use uuid::Uuid;
use tracing::{info, warn};

use crate::state::AppState;
use super::rpc::handle_json_rpc;

#[derive(Deserialize)]
pub struct MessageQuery {
    pub session_id: Option<String>,
}

pub async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let session_id = Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel::<String>(64);

    state
        .sse_sessions
        .write()
        .await
        .insert(session_id.clone(), tx);

    info!("New MCP SSE connection opened. Session ID: {}", session_id);

    let prefix = state.config.webservice.api_prefix.trim_end_matches('/');
    let endpoint_url = format!("{}/v1/message?session_id={}", prefix, session_id);

    let initial_event = Event::default().event("endpoint").data(endpoint_url);

    let stream = ReceiverStream::new(rx).map(|msg| Ok(Event::default().event("message").data(msg)));

    let combined = tokio_stream::once(Ok(initial_event)).chain(stream);



    Sse::new(combined).keep_alive(KeepAlive::default())
}

pub async fn message_handler(
    State(state): State<AppState>,
    Query(query): Query<MessageQuery>,
    Json(payload): Json<Value>,
) -> Response {
    let session_id = match query.session_id {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing session_id query parameter" })),
            )
                .into_response();
        }
    };

    let tx = {
        let sessions = state.sse_sessions.read().await;
        sessions.get(&session_id).cloned()
    };

    let tx = match tx {
        Some(t) => t,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Session not found" })),
            )
                .into_response();
        }
    };

    let rpc_response = handle_json_rpc(&state, payload).await;

    if let Some(resp) = rpc_response {
        let resp_str = serde_json::to_string(&resp).unwrap_or_default();
        if let Err(e) = tx.send(resp_str).await {
            warn!("Failed to deliver response to SSE stream for session {}: {}", session_id, e);
        }
    }

    (StatusCode::ACCEPTED, "Accepted").into_response()
}

pub async fn direct_rpc_handler(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Response {
    let rpc_response = handle_json_rpc(&state, payload).await;
    match rpc_response {
        Some(resp) => (StatusCode::OK, Json(resp)).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}
