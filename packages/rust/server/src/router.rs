use crate::routes;
use crate::services::email::EmailService;
use crate::types::DbPool;
use axum::routing::get;
use axum::{Extension, Json, Router};
use serde::Serialize;

pub fn router(pool: DbPool, email_service: EmailService) -> Router {
    Router::new()
        .route("/", get(hello))
        .nest("/user", routes::user::router())
        .nest("/client", routes::client::router())
        .nest("/message", routes::message::router())
        .nest("/client", routes::client::router())
        .layer(Extension(pool))
        .layer(Extension(email_service))
}

#[derive(Serialize)]
struct HelloInfo {
    status: String,
}

async fn hello() -> Json<HelloInfo> {
    (HelloInfo {
        status: "Hello World!".to_string(),
    })
    .into()
}
