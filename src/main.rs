use std::fmt::Write;
use std::process::Command;
use std::sync::Arc;

use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Router, http::StatusCode};
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct User {
    login: String,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct Issue {
    number: u64,
}

#[derive(Debug, Deserialize)]
struct Comment {
    user: User,
    body: String,
}

#[derive(Debug, Deserialize)]
struct Repository {
    name: String,
    owner: User,
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
    Command::new("git")
        .args(["checkout", branch_name])
        .current_dir(&raphael_dir)
        .status()
        .unwrap();
    Command::new("git")
        .arg("pull")
        .current_dir(&raphael_dir)
        .status()
        .unwrap();
    Command::new("cargo")
        .args(["install", "--path", "raphael-cli"])
        .current_dir(&raphael_dir)
        .status()
        .unwrap();
}

fn checkout_and_build_raphael_cli_on_pr(pr_number: u64) {
    let raphael_dir = std::env::var("RAPHAEL_DIR").expect("missing RAPHAEL_DIR var");
    Command::new("gh")
        .args(["pr", "checkout", &pr_number.to_string()])
        .current_dir(&raphael_dir)
        .status()
        .unwrap();
    Command::new("cargo")
        .args(["install", "--path", "raphael-cli"])
        .current_dir(&raphael_dir)
        .status()
        .unwrap();
}

fn run_benchmark_script() -> String {
    let output = Command::new("./scripts/bench-solver.sh").output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

fn create_comment_on_issue(payload: &Payload, message: String) {
    let owner = &payload.repository.owner.login;
    let repo = &payload.repository.name;
    let number = &payload.issue.number;
    Command::new("gh")
        .args(["api", "--method", "POST"])
        .args(["-H", "Accept: application/vnd.github+json"])
        .args(["-H", "X-GitHub-Api-Version: 2022-11-28"])
        .arg(format!("/repos/{owner}/{repo}/issues/{number}/comments"))
        .arg("-f")
        .arg(format!("body={message}"))
        .status()
        .unwrap();
}

fn run_benchmark_job(payload: Payload) {
    let mut message = String::new();
    checkout_and_build_raphael_cli_on_branch(&payload.repository.default_branch);
    writeln!(
        message,
        "Benchmarking `{}`:\n{}\n\n",
        payload.repository.default_branch,
        run_benchmark_script()
    )
    .unwrap();
    checkout_and_build_raphael_cli_on_pr(payload.issue.number);
    writeln!(
        message,
        "Benchmarking `pr {}`:\n{}\n\n",
        payload.issue.number,
        run_benchmark_script()
    )
    .unwrap();
    dbg!(&message);
    create_comment_on_issue(&payload, message);
}

async fn webhook_handler(GithubEvent(payload): GithubEvent<Payload>) -> impl IntoResponse {
    dbg!(&payload);
    if payload.action == "created"
        && payload.comment.user.login == "KonaeAkira"
        && payload.comment.user.id == 31180380
        && payload.comment.body.contains("@RaphaelBencher")
    {
        std::thread::spawn(move || run_benchmark_job(payload));
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
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
