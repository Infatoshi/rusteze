pub mod auth;
pub mod channels;
pub mod dms;
pub mod invites;
pub mod members;
pub mod messages;
pub mod reactions;
pub mod roles;
pub mod servers;
pub mod uploads;
pub mod users;

use axum::Json;
use serde_json::{json, Value};

pub async fn root() -> Json<Value> {
    Json(json!({
        "rusteze": env!("CARGO_PKG_VERSION"),
        "ws": "/gateway",
    }))
}
