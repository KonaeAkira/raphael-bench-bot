use std::fmt::Write;
use std::process::Command;
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
struct Repository {
    default_branch: String,
}

#[derive(Debug, Deserialize)]
struct Payload {
    action: String,
    issue: Issue,
    comment: Comment,
    repository: Repository,
}

fn checkout_and_build_raphael_cli_on_branch(branch_name: &str) {
    let raphael_dir = std::env::var("RAPHAEL_DIR").expect("missing RAPHAEL_DIR var");
    Command::new(format!("git checkout {branch_name}"))
        .current_dir(&raphael_dir)
        .status()
        .unwrap();
    Command::new(format!("cargo install --path raphael-cli"))
        .current_dir(&raphael_dir)
        .status()
        .unwrap();
}

fn checkout_and_build_raphael_cli_on_pr(pr_number: u32) {
    let raphael_dir = std::env::var("RAPHAEL_DIR").expect("missing RAPHAEL_DIR var");
    Command::new(format!("gh pr checkout {pr_number}"))
        .current_dir(&raphael_dir)
        .status()
        .unwrap();
    Command::new(format!("cargo install --path raphael-cli"))
        .current_dir(&raphael_dir)
        .status()
        .unwrap();
}

fn run_benchmark() -> String {
    let scripts_dir = "./scripts";
    let output = Command::new("bench-solver.sh")
        .current_dir(scripts_dir)
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

async fn webhook_handler(GithubEvent(payload): GithubEvent<Payload>) -> impl IntoResponse {
    dbg!(&payload);
    if payload.action == "created"
        && payload.comment.author_association == "OWNER"
        && payload.comment.body.trim() == "/bench"
    {
        let mut message = String::new();

        checkout_and_build_raphael_cli_on_branch(&payload.repository.default_branch);
        writeln!(
            message,
            "Benchmarking `{}`:\n{}\n\n",
            payload.repository.default_branch,
            run_benchmark()
        )
        .unwrap();

        checkout_and_build_raphael_cli_on_pr(payload.issue.number);
        writeln!(
            message,
            "Benchmarking `pr {}`:\n{}\n\n",
            payload.issue.number,
            run_benchmark()
        )
        .unwrap();

        dbg!(&message);
    }
    StatusCode::OK
}

#[tokio::main]
async fn main() {
    let secret = std::env::var("GITHUB_WEBHOOK_SECRET").expect("missing GITHUB_WEBHOOK_SECRET var");
    let app = Router::new()
        .route("/webhooks", post(webhook_handler))
        .with_state(GithubToken(Arc::new(secret)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
