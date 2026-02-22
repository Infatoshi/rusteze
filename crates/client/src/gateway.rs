use std::thread;
use std::time::Duration;

use async_channel::{Receiver, Sender};
use rusteze_models::{ClientEvent, ServerEvent};
use tungstenite::{connect, Message};

/// Spawn a background thread that maintains a WebSocket connection to the gateway.
/// Automatically reconnects with exponential backoff on disconnection.
/// Returns a `Sender<ClientEvent>` the UI can use to send commands.
pub fn spawn_gateway(
    gateway_url: String,
    token: String,
    event_tx: Sender<ServerEvent>,
) -> Sender<ClientEvent> {
    let (cmd_tx, cmd_rx) = async_channel::unbounded::<ClientEvent>();

    thread::spawn(move || {
        let mut backoff_ms = 1000u64;
        let max_backoff_ms = 30000u64;

        loop {
            match run_gateway(&gateway_url, &token, &event_tx, &cmd_rx) {
                Ok(()) => {
                    tracing::info!("gateway closed cleanly");
                    break; // Clean shutdown
                }
                Err(e) => {
                    tracing::error!("gateway error: {e}, reconnecting in {backoff_ms}ms...");
                    thread::sleep(Duration::from_millis(backoff_ms));
                    backoff_ms = (backoff_ms * 2).min(max_backoff_ms);

                    // Check if the event channel is still alive
                    if event_tx.is_closed() {
                        tracing::info!("event channel closed, stopping gateway reconnect loop");
                        break;
                    }
                }
            }
        }
    });

    cmd_tx
}

fn run_gateway(
    url: &str,
    token: &str,
    tx: &Sender<ServerEvent>,
    cmd_rx: &Receiver<ClientEvent>,
) -> anyhow::Result<()> {
    tracing::info!("connecting to gateway: {url}");
    let (mut ws, _resp) = connect(url)?;
    tracing::info!("gateway connected, authenticating...");

    // Send Authenticate
    let auth = ClientEvent::Authenticate {
        token: token.to_string(),
    };
    let auth_json = serde_json::to_string(&auth)?;
    ws.send(Message::Text(auth_json.into()))?;

    // Set the socket to non-blocking for interleaving reads and command drains
    match ws.get_mut() {
        tungstenite::stream::MaybeTlsStream::Plain(tcp) => {
            tcp.set_nonblocking(true).ok();
        }
        _ => {}
    }

    // Track seen message IDs to deduplicate
    let mut seen_msg_ids = std::collections::HashSet::new();
    // Only keep last 500 IDs to bound memory
    const MAX_SEEN: usize = 500;

    loop {
        // 1. Drain outbound commands from the UI
        while let Ok(cmd) = cmd_rx.try_recv() {
            if let Ok(json) = serde_json::to_string(&cmd) {
                ws.send(Message::Text(json.into()))?;
            }
        }

        // 2. Try to read from WebSocket (non-blocking)
        match ws.read() {
            Ok(msg) => match msg {
                Message::Text(text) => {
                    match serde_json::from_str::<ServerEvent>(&text) {
                        Ok(event) => {
                            // Deduplicate MessageCreate events
                            let should_send = match &event {
                                ServerEvent::MessageCreate(msg) => {
                                    if seen_msg_ids.contains(&msg.id) {
                                        false
                                    } else {
                                        seen_msg_ids.insert(msg.id);
                                        if seen_msg_ids.len() > MAX_SEEN {
                                            // Evict oldest (just clear and re-add recent)
                                            seen_msg_ids.clear();
                                            seen_msg_ids.insert(msg.id);
                                        }
                                        true
                                    }
                                }
                                _ => true,
                            };

                            if should_send {
                                if tx.send_blocking(event).is_err() {
                                    tracing::info!("event channel closed, stopping gateway");
                                    return Ok(()); // Clean shutdown
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("failed to parse gateway event: {e}");
                        }
                    }
                }
                Message::Close(_) => {
                    tracing::info!("gateway closed by server");
                    return Err(anyhow::anyhow!("server closed connection"));
                }
                Message::Ping(data) => {
                    let _ = ws.send(Message::Pong(data));
                }
                _ => {}
            },
            Err(tungstenite::Error::Io(ref e))
                if e.kind() == std::io::ErrorKind::WouldBlock =>
            {
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                return Err(anyhow::anyhow!("read error: {e}"));
            }
        }
    }
}
