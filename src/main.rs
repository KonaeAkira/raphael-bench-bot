use axum::extract::FromRef;
use axum::routing::post;
use axum::{Json, Router};
use axum_standardwebhooks::{SharedWebhook, StandardWebhook, Webhook};
use serde_json::Value;

async fn webhook_handler(StandardWebhook(Json(payload)): StandardWebhook<Json<Value>>) -> String {
    // The webhook signature has been verified, and we can safely use the payload
    dbg!(&payload);
    format!("Received webhook: {}", payload)
}

#[derive(Clone)]
struct AppState {
    webhook: SharedWebhook,
}

impl FromRef<AppState> for SharedWebhook {
    fn from_ref(state: &AppState) -> Self {
        state.webhook.clone()
    }
}

#[tokio::main]
async fn main() {
    let secret = std::env::var("GITHUB_WEBHOOK_SECRET").unwrap();
    let app = Router::new()
        .route("/webhooks", post(webhook_handler))
        .with_state(AppState {
            webhook: SharedWebhook::new(Webhook::new(&secret).unwrap()),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
