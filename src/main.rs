#![forbid(unsafe_code)]

mod bootstrap;
mod db;
mod domain;
mod http;
mod schema;
mod support;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::run().await
}
