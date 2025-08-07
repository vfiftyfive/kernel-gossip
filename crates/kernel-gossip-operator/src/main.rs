use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting kernel-gossip-operator");

    // Create servers
    let webhook_server = kernel_gossip_operator::server::create_server().await?;
    let metrics_server = kernel_gossip_operator::server::create_metrics_server().await?;

    info!("Webhook server listening on port {}", std::env::var("WEBHOOK_PORT").unwrap_or_else(|_| "8080".to_string()));
    info!("Metrics server listening on port {}", std::env::var("METRICS_PORT").unwrap_or_else(|_| "9090".to_string()));

    // Run both servers concurrently
    tokio::select! {
        result = webhook_server => {
            error!("Webhook server stopped: {:?}", result);
        }
        result = metrics_server => {
            error!("Metrics server stopped: {:?}", result);
        }
    }

    Ok(())
}