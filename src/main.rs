#![allow(dead_code)]

use anyhow::Result;
use rmcp::ServiceExt;

mod indonesia;
mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("env-indonesia-mcp v1.0.0 — Environmental AI MCP Server for Indonesia");

    let server = server::EnvIndonesiaServer::new();
    let service = server.serve(rmcp::transport::io::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
