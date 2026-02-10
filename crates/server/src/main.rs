use std::{env, sync::Arc};

use axum::{
    Router,
    routing::{get, post},
};
use fred::interfaces::ClientLike;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
mod state;
mod error;
mod extract;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "rusteze_server=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let bind = env::var("BIND").unwrap_or_else(|_| "0.0.0.0:14702".into());

    let pool = rusteze_db::connect(&database_url).await.expect("failed to connect to database");
    rusteze_db::migrate(&pool).await.expect("failed to run migrations");

    let redis_config = fred::types::config::Config::from_url(&redis_url).expect("invalid REDIS_URL");
    let redis = fred::clients::Client::new(redis_config, None, None, None);
    redis.init().await.expect("failed to connect to Redis");

    let state = Arc::new(AppState {
        db: pool,
        redis,
        jwt_secret,
    });

    let app = Router::new()
        // Health
        .route("/", get(routes::root))
        // Auth
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        // Servers
        .route("/servers", post(routes::servers::create_server))
        .route("/servers", get(routes::servers::list_servers))
        // Channels
        .route("/servers/{server_id}/channels", post(routes::channels::create_channel))
        .route("/servers/{server_id}/channels", get(routes::channels::list_channels))
        // Messages
        .route("/channels/{channel_id}/messages", get(routes::messages::list_messages))
        .route("/channels/{channel_id}/messages", post(routes::messages::send_message))
        // Invites
        .route("/servers/{server_id}/invites", post(routes::invites::create_invite))
        .route("/invites/{code}/join", post(routes::invites::join_invite))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    tracing::info!("API server listening on {bind}");
    axum::serve(listener, app).await.unwrap();
}
