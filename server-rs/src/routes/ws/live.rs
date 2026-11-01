//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! - Session internals preserve authentication, bounded transport and permit ownership.
//!
//! ### 🔍 Debugging & Observability
//! - Session failures use the existing sanitized tracing targets.

use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::HeaderMap,
    response::IntoResponse,
    Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::OwnedSemaphorePermit;

use super::shared::{verify_origin, WS_CONNECTION_SEMAPHORE};
use super::{
    LIVE_ESTIMATED_COST_PER_SEC, LIVE_MAX_SESSION_BYTES, WS_CLOSE_QUOTA_EXHAUSTED,
    WS_CLOSE_SESSION_CAP_EXCEEDED, WS_LIVE_CONNECT_TIMEOUT,
};

/// Specialized WebSocket handler for Gemini Live Multimodal API.
/// Proxies client audio/setup to Google's backend to protect API keys.
#[tracing::instrument(
    skip(state, headers, ws, pre_auth),
    name = "system::live_voice_upgrade"
)]
pub async fn live_voice_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    pre_auth: Option<Extension<crate::middleware::auth::PreAuthenticated>>,
) -> Result<impl IntoResponse, AppError> {
    if pre_auth.is_none() {
        return Err(AppError::Unauthorized("Unauthorized".to_string()));
    }

    let permit = WS_CONNECTION_SEMAPHORE
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            AppError::RateLimit("Concurrent WebSocket connection limit reached".to_string())
        })?;

    if let Err(status) = verify_origin(&headers) {
        return Ok(status.into_response());
    }

    let protocol = headers
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or_default();

    let ws = if !protocol.is_empty() {
        ws.protocols([protocol])
    } else {
        ws
    };

    let ws = ws
        .max_message_size(16 * 1024 * 1024)
        .max_frame_size(16 * 1024 * 1024);

    Ok(ws
        .on_upgrade(move |socket| handle_live_socket(socket, state, permit))
        .into_response())
}

async fn handle_live_socket(
    mut client_ws: WebSocket,
    state: Arc<AppState>,
    _permit: OwnedSemaphorePermit,
) {
    let api_key = match std::env::var("GOOGLE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            let _ = client_ws
                .send(Message::Text(
                    "Error: GOOGLE_API_KEY not found on server".into(),
                ))
                .await;
            return;
        }
    };

    // Budget Guard pre-flight check (entity: system:live_voice)
    let live_entity_id = "system:live_voice";
    match state
        .security
        .budget_guard
        .check_budget(live_entity_id, 0.05)
        .await
    {
        Ok(true) => {}
        Ok(false) => {
            tracing::warn!("🚫 [LiveWS] Insufficient budget for Live Voice streaming");
            let _ = client_ws
                .send(Message::Text(
                    "Error: Insufficient budget quota for Live Voice session.".into(),
                ))
                .await;
            return;
        }
        Err(e) => {
            tracing::error!("❌ [LiveWS] Budget check failed (failing closed): {}", e);
            let _ = client_ws
                .send(Message::Text("Error: budget_verification_failed".into()))
                .await;
            return;
        }
    }

    let gemini_url = format!(
        "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.MultimodalLive?key={}",
        api_key
    );

    use tokio_tungstenite::connect_async;
    let connect_future = connect_async::<String>(gemini_url);
    let (gemini_ws, _) = match tokio::time::timeout(WS_LIVE_CONNECT_TIMEOUT, connect_future).await {
        Ok(Ok(conn)) => conn,
        Ok(Err(e)) => {
            let sanitized_err = state.security.secret_redactor.redact(&e.to_string());
            tracing::error!("❌ [LiveWS] Failed to connect to Gemini: {}", sanitized_err);
            let _ = client_ws
                .send(Message::Text("Error: upstream_connect_failed".into()))
                .await;
            return;
        }
        Err(_) => {
            tracing::error!("❌ [LiveWS] Gemini connection attempt timed out after 10 seconds");
            let _ = client_ws
                .send(Message::Text("Error: upstream_connect_timeout".into()))
                .await;
            return;
        }
    };

    relay_live_session(client_ws, state, gemini_ws).await;
}

/// Relay ownership stays within the session; the connector can be a local fixture.
async fn relay_live_session<S>(
    client_ws: WebSocket,
    state: Arc<AppState>,
    gemini_ws: tokio_tungstenite::WebSocketStream<S>,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let live_entity_id = "system:live_voice";
    let (gemini_sender, mut gemini_receiver) = gemini_ws.split();
    let gemini_sender = Arc::new(tokio::sync::Mutex::new(gemini_sender));
    let (client_sender, mut client_receiver) = client_ws.split();
    let client_sender = Arc::new(tokio::sync::Mutex::new(client_sender));

    tracing::info!("🎙️ [LiveWS] Gemini Live Proxy Established with Budget Guard");

    let client_sender_c2g = client_sender.clone();
    let gemini_sender_c2g = gemini_sender.clone();

    let last_meter_instant = Arc::new(parking_lot::Mutex::new(std::time::Instant::now()));
    let meter_state = state.clone();
    let meter_client_sender = client_sender.clone();
    let last_meter_for_timer = Arc::clone(&last_meter_instant);

    let meter_loop = async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        interval.tick().await;

        loop {
            interval.tick().await;
            let elapsed_secs = {
                let mut last = last_meter_for_timer.lock();
                let elapsed = last.elapsed().as_secs_f64();
                *last = std::time::Instant::now();
                elapsed
            };

            let cost_usd = (elapsed_secs * LIVE_ESTIMATED_COST_PER_SEC).max(0.0001);
            if let Err(e) = meter_state
                .security
                .budget_guard
                .record_usage(live_entity_id, cost_usd)
                .await
            {
                tracing::error!("🚫 [LiveWS] Failed to record usage (failing closed): {}", e);
                break;
            }

            match meter_state
                .security
                .budget_guard
                .check_budget(live_entity_id, 0.01)
                .await
            {
                Ok(true) => {}
                Ok(false) => {
                    tracing::warn!(
                        "🚫 [LiveWS] Quota exhausted during active stream. Terminating session."
                    );
                    let mut sender = meter_client_sender.lock().await;
                    let _ = sender
                        .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                            code: WS_CLOSE_QUOTA_EXHAUSTED,
                            reason: "Quota exhausted".into(),
                        })))
                        .await;
                    break;
                }
                Err(e) => {
                    tracing::error!(
                        "🚫 [LiveWS] Mid-stream budget check failed (failing closed): {}",
                        e
                    );
                    break;
                }
            }
        }
    };

    let c2g = async move {
        let mut bytes_sent: u64 = 0;

        while let Some(msg) = client_receiver.next().await {
            match msg {
                Ok(axum::extract::ws::Message::Text(t)) => {
                    let chunk_len = t.len() as u64;
                    bytes_sent += chunk_len;
                    if bytes_sent > LIVE_MAX_SESSION_BYTES {
                        tracing::warn!(
                            "🚫 [LiveWS] Session byte cap exceeded ({} bytes). Closing c2g pipe.",
                            bytes_sent
                        );
                        let mut sender = client_sender_c2g.lock().await;
                        let _ = sender
                            .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                                code: WS_CLOSE_SESSION_CAP_EXCEEDED,
                                reason: "Session cap exceeded".into(),
                            })))
                            .await;
                        break;
                    }
                    let mut g_sender = gemini_sender_c2g.lock().await;
                    let _ = g_sender
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            t.as_str().into(),
                        ))
                        .await;
                }
                Ok(axum::extract::ws::Message::Binary(b)) => {
                    let chunk_len = b.len() as u64;
                    bytes_sent += chunk_len;
                    if bytes_sent > LIVE_MAX_SESSION_BYTES {
                        tracing::warn!(
                            "🚫 [LiveWS] Session byte cap exceeded ({} bytes). Closing c2g pipe.",
                            bytes_sent
                        );
                        let mut sender = client_sender_c2g.lock().await;
                        let _ = sender
                            .send(Message::Close(Some(axum::extract::ws::CloseFrame {
                                code: WS_CLOSE_SESSION_CAP_EXCEEDED,
                                reason: "Session cap exceeded".into(),
                            })))
                            .await;
                        break;
                    }
                    let mut g_sender = gemini_sender_c2g.lock().await;
                    let _ = g_sender
                        .send(tokio_tungstenite::tungstenite::Message::Binary(b))
                        .await;
                }
                _ => {}
            }
        }
    };

    let client_sender_g2c = client_sender.clone();
    let g2c = async move {
        let mut downstream_bytes: u64 = 0;
        while let Some(msg) = gemini_receiver.next().await {
            match msg {
                Ok(tokio_tungstenite::tungstenite::Message::Text(t)) => {
                    downstream_bytes += t.len() as u64;
                    if downstream_bytes > LIVE_MAX_SESSION_BYTES {
                        tracing::warn!(
                            "🚫 [LiveWS] Downstream byte cap exceeded ({} bytes).",
                            downstream_bytes
                        );
                        break;
                    }
                    let mut sender = client_sender_g2c.lock().await;
                    let _ = sender
                        .send(axum::extract::ws::Message::Text(t.to_string().into()))
                        .await;
                }
                Ok(tokio_tungstenite::tungstenite::Message::Binary(b)) => {
                    downstream_bytes += b.len() as u64;
                    if downstream_bytes > LIVE_MAX_SESSION_BYTES {
                        tracing::warn!(
                            "🚫 [LiveWS] Downstream byte cap exceeded ({} bytes).",
                            downstream_bytes
                        );
                        break;
                    }
                    let mut sender = client_sender_g2c.lock().await;
                    let _ = sender.send(axum::extract::ws::Message::Binary(b)).await;
                }
                _ => {}
            }
        }
    };

    tokio::select! {
        _ = c2g => { tracing::info!("🎙️ [LiveWS] Client closed connection"); }
        _ = g2c => { tracing::info!("🎙️ [LiveWS] Gemini closed connection"); }
        _ = meter_loop => { tracing::info!("🎙️ [LiveWS] Metering terminated session"); }
    }

    // Settle unmetered remainder on disconnect
    let remaining_secs = {
        let last = last_meter_instant.lock();
        last.elapsed().as_secs_f64()
    };
    if remaining_secs >= 0.5 {
        let final_cost = (remaining_secs * LIVE_ESTIMATED_COST_PER_SEC).max(0.0001);
        if let Err(e) = state
            .security
            .budget_guard
            .record_usage(live_entity_id, final_cost)
            .await
        {
            tracing::warn!("⚠️ [LiveWS] Failed to record final session usage: {}", e);
        } else {
            tracing::info!(
                "🎙️ [LiveWS] Final session duration settled: {:.2}s (${:.4})",
                remaining_secs,
                final_cost
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tokio::net::TcpListener;
    use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message as UpstreamMessage};

    #[tokio::test]
    async fn relay_forwards_both_directions_and_ends_when_client_closes() {
        let upstream_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream_addr = upstream_listener.local_addr().unwrap();
        let upstream = tokio::spawn(async move {
            let (stream, _) = upstream_listener.accept().await.unwrap();
            let mut socket = accept_async(stream).await.unwrap();
            assert_eq!(
                socket.next().await.unwrap().unwrap(),
                UpstreamMessage::Text("client-frame".into())
            );
            socket
                .send(UpstreamMessage::Binary(vec![1, 2, 3].into()))
                .await
                .unwrap();
            // Relay shutdown drops the upstream connection even without a second data frame.
            let _ = socket.next().await;
        });
        let state = Arc::new(AppState::new_minimal_mock().await);
        let (finished_tx, finished_rx) = tokio::sync::mpsc::channel(1);
        let app = Router::new().route(
            "/",
            get(move |ws: WebSocketUpgrade| {
                let state = state.clone();
                let finished_tx = finished_tx.clone();
                async move {
                    ws.on_upgrade(move |client| async move {
                        let (remote, _) = connect_async(format!("ws://{upstream_addr}"))
                            .await
                            .unwrap();
                        relay_live_session(client, state, remote).await;
                        let _ = finished_tx.send(()).await;
                    })
                }
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let scenario = async {
            let (mut client, _) = connect_async(format!("ws://{address}")).await.unwrap();
            client
                .send(UpstreamMessage::Text("client-frame".into()))
                .await
                .unwrap();
            assert_eq!(
                client.next().await.unwrap().unwrap(),
                UpstreamMessage::Binary(vec![1, 2, 3].into())
            );
            client.close(None).await.unwrap();
            let mut finished_rx = finished_rx;
            assert_eq!(finished_rx.recv().await, Some(()));
            upstream.await.unwrap();
        };
        let result = tokio::time::timeout(std::time::Duration::from_secs(10), scenario).await;
        server.abort();
        result.unwrap();
    }
}
