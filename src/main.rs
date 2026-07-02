use anyhow::Result;
use rmcp::ServiceExt;

mod ntb;
mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("env-ntb-mcp v0.1.0 — Environmental AI MCP Server for NTB Indonesia");

    let server = server::EnvNtbServer::new();
    let service = server.serve(rmcp::transport::io::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
