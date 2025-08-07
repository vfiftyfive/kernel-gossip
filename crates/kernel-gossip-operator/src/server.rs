use axum::{routing::get, Router};
use std::net::SocketAddr;
use crate::config::Config;
use tokio::net::TcpListener;
use axum::serve::Serve;
use kube::Client;
use std::sync::Arc;

pub async fn create_server() -> anyhow::Result<Serve<Router, Router>> {
    let config = Config::from_env()?;
    
    // Create K8s client
    let client = Client::try_default().await?;
    let client = Arc::new(client);
    
    let webhook_routes = crate::webhook::create_webhook_router(client);
    
    let app = Router::new()
        .route("/health", get(health_handler))
        .merge(webhook_routes);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.webhook_port));
    let listener = TcpListener::bind(addr).await?;
    let server = axum::serve(listener, app);
    
    Ok(server)
}

pub async fn create_metrics_server() -> anyhow::Result<Serve<Router, Router>> {
    let config = Config::from_env()?;
    
    let app = Router::new()
        .route("/metrics", get(metrics_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.metrics_port));
    let listener = TcpListener::bind(addr).await?;
    let server = axum::serve(listener, app);
    
    Ok(server)
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn metrics_handler() -> String {
    // Minimal Prometheus format
    "# HELP up Target up status\n# TYPE up gauge\nup 1\n".to_string()
}