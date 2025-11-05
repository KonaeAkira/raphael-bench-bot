use std::sync::Arc;

use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Router, http::StatusCode};
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Issue {
    number: u32,
}

#[derive(Debug, Deserialize)]
struct Comment {
    author_association: String,
    body: String,
}

#[derive(Debug, Deserialize)]
struct Payload {
    action: String,
    issue: Issue,
    comment: Comment,
}

async fn webhook_handler(GithubEvent(payload): GithubEvent<Payload>) -> impl IntoResponse {
    dbg!(&payload);
    StatusCode::OK
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
