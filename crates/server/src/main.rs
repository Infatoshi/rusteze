use std::{env, sync::Arc};

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get, patch, post, put},
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

    let media_path = env::var("MEDIA_PATH").unwrap_or_else(|_| "./data/uploads".into());
    let media = rusteze_media::LocalStorage::new(&media_path);

    let state = Arc::new(AppState {
        db: pool,
        redis,
        jwt_secret,
        media,
    });

    let app = Router::new()
        // Health
        .route("/", get(routes::root))

        // Auth
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/mfa/setup", post(routes::auth::mfa_setup))
        .route("/auth/mfa/verify", post(routes::auth::mfa_verify))
        .route("/auth/mfa", delete(routes::auth::mfa_disable))
        .route("/auth/oauth/google", get(routes::auth::oauth_google_url))
        .route("/auth/oauth/github", get(routes::auth::oauth_github_url))
        .route("/auth/oauth/callback", post(routes::auth::oauth_callback))

        // Users
        .route("/users/@me", get(routes::users::get_me))
        .route("/users/@me", patch(routes::users::update_me))
        .route("/users/{user_id}", get(routes::users::get_user))

        // Servers
        .route("/servers", post(routes::servers::create_server))
        .route("/servers", get(routes::servers::list_servers))
        .route("/servers/{server_id}", patch(routes::servers::update_server))
        .route("/servers/{server_id}", delete(routes::servers::delete_server))

        // Server members
        .route("/servers/{server_id}/members", get(routes::members::list_members))
        .route("/servers/{server_id}/members/@me", delete(routes::members::leave_server))
        .route("/servers/{server_id}/members/{user_id}", delete(routes::members::kick_member))

        // Channels
        .route("/servers/{server_id}/channels", post(routes::channels::create_channel))
        .route("/servers/{server_id}/channels", get(routes::channels::list_channels))
        .route("/servers/{server_id}/channels/{channel_id}", patch(routes::channels::update_channel))
        .route("/servers/{server_id}/channels/{channel_id}", delete(routes::channels::delete_channel))

        // Messages
        .route("/channels/{channel_id}/messages", get(routes::messages::list_messages))
        .route("/channels/{channel_id}/messages", post(routes::messages::send_message))
        .route("/channels/{channel_id}/messages/{message_id}", patch(routes::messages::edit_message))
        .route("/channels/{channel_id}/messages/{message_id}", delete(routes::messages::delete_message))
        .route("/channels/{channel_id}/messages/{message_id}/pin", post(routes::messages::pin_message))
        .route("/channels/{channel_id}/pins", get(routes::messages::list_pins))
        .route("/channels/{channel_id}/search", get(routes::messages::search_messages))

        // Reactions
        .route("/channels/{channel_id}/messages/{message_id}/reactions", put(routes::reactions::add_reaction))
        .route("/channels/{channel_id}/messages/{message_id}/reactions", get(routes::reactions::list_reactions))
        .route("/channels/{channel_id}/messages/{message_id}/reactions/{emoji}", delete(routes::reactions::remove_reaction))

        // Roles
        .route("/servers/{server_id}/roles", post(routes::roles::create_role))
        .route("/servers/{server_id}/roles", get(routes::roles::list_roles))
        .route("/servers/{server_id}/roles/{role_id}", patch(routes::roles::update_role))
        .route("/servers/{server_id}/roles/{role_id}", delete(routes::roles::delete_role))
        .route("/servers/{server_id}/roles/{role_id}/members", put(routes::roles::assign_role))
        .route("/servers/{server_id}/roles/{role_id}/members/{user_id}", delete(routes::roles::revoke_role))

        // Invites
        .route("/servers/{server_id}/invites", post(routes::invites::create_invite))
        .route("/invites/{code}/join", post(routes::invites::join_invite))

        // File uploads
        .route("/channels/{channel_id}/files", post(routes::uploads::upload_file_base64))
        .route("/channels/{channel_id}/upload", post(routes::uploads::upload_file))
        .route("/attachments/{attachment_id}", get(routes::uploads::get_attachment))
        .route("/attachments/{attachment_id}/base64", get(routes::uploads::get_attachment_base64))

        // DMs
        .route("/dms", post(routes::dms::create_dm))
        .route("/dms", get(routes::dms::list_dms))
        .route("/dms/group", post(routes::dms::create_group_dm))

        .layer(DefaultBodyLimit::max(15 * 1024 * 1024)) // 15MB for base64 file uploads
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    tracing::info!("API server listening on {bind}");
    axum::serve(listener, app).await.unwrap();
}
