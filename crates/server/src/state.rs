use sqlx::PgPool;

pub struct AppState {
    pub db: PgPool,
    pub redis: fred::clients::Client,
    pub jwt_secret: String,
}
