use tracing::{info, error};
use kube::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting kernel-gossip-operator");

    // Create K8s client
    let client = Client::try_default().await?;
    let client_for_controller = client.clone();
    let client_for_watcher = client.clone();

    // Create servers
    let webhook_server = kernel_gossip_operator::server::create_server().await?;
    let metrics_server = kernel_gossip_operator::server::create_metrics_server().await?;

    info!("Webhook server listening on port {}", std::env::var("WEBHOOK_PORT").unwrap_or_else(|_| "8080".to_string()));
    info!("Metrics server listening on port {}", std::env::var("METRICS_PORT").unwrap_or_else(|_| "9090".to_string()));

    // Start CRD controllers
    let controller_handle = tokio::spawn(async move {
        if let Err(e) = kernel_gossip_operator::crd::run_controllers(client_for_controller).await {
            error!("Controller error: {}", e);
        }
    });

    // Start pod watcher
    let pod_watcher_handle = tokio::spawn(async move {
        if let Err(e) = kernel_gossip_operator::pod_watcher::run_pod_watcher(client_for_watcher).await {
            error!("Pod watcher error: {}", e);
        }
    });

    // Run all components concurrently
    tokio::select! {
        result = webhook_server => {
            error!("Webhook server stopped: {:?}", result);
        }
        result = metrics_server => {
            error!("Metrics server stopped: {:?}", result);
        }
        result = controller_handle => {
            error!("Controllers stopped: {:?}", result);
        }
        result = pod_watcher_handle => {
            error!("Pod watcher stopped: {:?}", result);
        }
    }

    Ok(())
}