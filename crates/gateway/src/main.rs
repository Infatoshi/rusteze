use std::{env, sync::Arc};

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
};
use fred::{
    interfaces::{ClientLike, EventInterface, PubsubInterface},
    types::{Builder, config::Config as RedisConfig},
};
use futures::{SinkExt, StreamExt};
use rusteze_models::{ClientEvent, ServerEvent};
use sqlx::PgPool;
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct GatewayState {
    jwt_secret: String,
    redis_url: String,
    db: PgPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rusteze_gateway=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let bind = env::var("GATEWAY_BIND").unwrap_or_else(|_| "0.0.0.0:14703".into());

    let db = rusteze_db::connect(&database_url)
        .await
        .expect("failed to connect to database");

    let state = Arc::new(GatewayState {
        jwt_secret,
        redis_url,
        db,
    });

    let app = Router::new()
        .route("/", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    tracing::info!("gateway listening on {bind}");
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<GatewayState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<GatewayState>) {
    let (mut sink, mut stream) = socket.split();

    // Wait for Authenticate message
    let user_id = loop {
        match stream.next().await {
            Some(Ok(Message::Text(text))) => {
                if let Ok(event) = serde_json::from_str::<ClientEvent>(&text) {
                    match event {
                        ClientEvent::Authenticate { token } => {
                            match rusteze_auth::token::validate_token(&token, &state.jwt_secret) {
                                Ok(claims) => break claims.sub,
                                Err(_) => {
                                    let _ = sink.close().await;
                                    return;
                                }
                            }
                        }
                        ClientEvent::Ping { ts } => {
                            let pong = serde_json::to_string(&ServerEvent::Pong { ts }).unwrap();
                            let _ = sink.send(Message::Text(pong.into())).await;
                        }
                        _ => {}
                    }
                }
            }
            Some(Ok(Message::Close(_))) | None => return,
            _ => {}
        }
    };

    tracing::info!("user {user_id} authenticated on gateway");

    // Load user's data for Ready event
    let servers = rusteze_db::servers::fetch_user_servers(&state.db, user_id)
        .await
        .unwrap_or_default();

    let channel_ids = rusteze_db::members::user_channel_ids(&state.db, user_id)
        .await
        .unwrap_or_default();

    // Build and send Ready event
    let ready = ServerEvent::Ready {
        user: rusteze_models::PartialUser {
            id: user_id,
            username: String::new(),
            discriminator: String::new(),
            display_name: None,
            avatar_url: None,
            status: rusteze_models::UserStatus::Online,
        },
        servers: servers
            .iter()
            .map(|s| rusteze_models::Server {
                id: s.id,
                name: s.name.clone(),
                owner_id: s.owner_id,
                icon_url: s.icon_url.clone(),
                banner_url: s.banner_url.clone(),
                description: s.description.clone(),
                created_at: s.created_at,
            })
            .collect(),
        channels: vec![], // channels loaded per-server by client
        members: vec![],
    };

    let ready_json = serde_json::to_string(&ready).unwrap();
    if sink.send(Message::Text(ready_json.into())).await.is_err() {
        return;
    }

    // Create a Redis subscriber for this connection
    let redis_config = RedisConfig::from_url(&state.redis_url).unwrap();
    let subscriber = match Builder::from_config(redis_config).build_subscriber_client() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to build redis subscriber: {e}");
            return;
        }
    };

    if subscriber.init().await.is_err() {
        return;
    }

    // Subscribe to user's personal channel
    let _ = subscriber.subscribe(format!("user:{user_id}")).await;

    // Subscribe to all channels the user has access to
    for ch_id in &channel_ids {
        let _ = subscriber.subscribe(format!("channel:{ch_id}")).await;
    }

    tracing::info!(
        "user {user_id} subscribed to {} channels",
        channel_ids.len()
    );

    // Bridge Redis -> WebSocket via broadcast channel
    let (tx, mut rx) = broadcast::channel::<String>(256);

    let mut message_rx = subscriber.message_rx();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        while let Ok(msg) = message_rx.recv().await {
            if let Ok(payload) = msg.value.convert::<String>() {
                let _ = tx_clone.send(payload);
            }
        }
    });

    // Main event loop
    loop {
        tokio::select! {
            // Outbound: Redis -> Client
            Ok(payload) = rx.recv() => {
                if sink.send(Message::Text(payload.into())).await.is_err() {
                    break;
                }
            }
            // Inbound: Client -> Server
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(event) = serde_json::from_str::<ClientEvent>(&text) {
                            match event {
                                ClientEvent::Ping { ts } => {
                                    let pong = serde_json::to_string(&ServerEvent::Pong { ts }).unwrap();
                                    let _ = sink.send(Message::Text(pong.into())).await;
                                }
                                ClientEvent::TypingStart { channel_id } => {
                                    let event = ServerEvent::TypingStart {
                                        channel_id,
                                        user_id,
                                    };
                                    if let Ok(payload) = serde_json::to_string(&event) {
                                        let _: Result<(), _> = PubsubInterface::publish(
                                            &subscriber,
                                            format!("channel:{channel_id}"),
                                            payload.as_str(),
                                        ).await;
                                    }
                                }
                                ClientEvent::Subscribe { channel_id } => {
                                    let _ = subscriber.subscribe(format!("channel:{channel_id}")).await;
                                    tracing::debug!("user {user_id} subscribed to channel:{channel_id}");
                                }
                                _ => {}
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }

    tracing::info!("user {user_id} disconnected from gateway");
    let _ = subscriber.quit().await;
}
