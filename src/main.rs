use std::sync::Arc;

use axum::Router;
use axum::response::IntoResponse;
use axum::routing::post;
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Event {
    action: String,
}

async fn webhook_handler(GithubEvent(event): GithubEvent<Event>) -> impl IntoResponse {
    dbg!(&event);
    event.action
}

#[tokio::main]
async fn main() {
    let secret = std::env::var("GITHUB_WEBHOOK_SECRET").unwrap();
    let app = Router::new()
        .route("/webhooks", post(webhook_handler))
        .with_state(GithubToken(Arc::new(secret)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
