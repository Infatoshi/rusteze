pub mod auth;
pub mod messages;
pub mod servers;

use axum::Json;
use serde_json::{json, Value};

pub async fn root() -> Json<Value> {
    Json(json!({
        "rusteze": env!("CARGO_PKG_VERSION"),
        "ws": "ws://100.119.229.90:14703",
    }))
}
