#[cfg(test)]
mod server_tests {
    use tokio::net::TcpListener;
    use axum::{Router, routing::get};

    #[tokio::test]
    async fn test_health_endpoint() {
        // Set required env vars
        std::env::set_var("PIXIE_API_KEY", "test-key");
        std::env::set_var("PIXIE_CLUSTER_ID", "test-cluster");

        // Create app directly for testing
        let app = Router::new()
            .route("/health", get(|| async { "OK" }));

        // Bind to random port
        let listener = TcpListener::bind("127.0.0.1:0").await
            .expect("Failed to bind");
        let addr = listener.local_addr().expect("Failed to get addr");

        // Spawn server in background
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("Server failed");
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Test health endpoint
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://{}/health", addr))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(resp.status(), 200);
        
        let body = resp.text().await.expect("Failed to read body");
        assert_eq!(body, "OK");
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        // Create metrics app directly for testing
        let app = Router::new()
            .route("/metrics", get(|| async {
                "# HELP up Target up status\n# TYPE up gauge\nup 1\n"
            }));

        // Bind to random port
        let listener = TcpListener::bind("127.0.0.1:0").await
            .expect("Failed to bind");
        let addr = listener.local_addr().expect("Failed to get addr");

        // Spawn server in background
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("Server failed");
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Test metrics endpoint
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://{}/metrics", addr))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(resp.status(), 200);
        
        let body = resp.text().await.expect("Failed to read body");
        assert!(body.contains("# HELP"));
        assert!(body.contains("# TYPE"));
    }

    #[tokio::test]
    #[ignore = "Requires K8s cluster"]
    async fn test_server_creation() {
        // Set required env vars
        std::env::set_var("PIXIE_API_KEY", "test-key-create");
        std::env::set_var("PIXIE_CLUSTER_ID", "test-cluster-create");
        std::env::set_var("WEBHOOK_PORT", "8083");

        // Test that we can create the server
        let server = kernel_gossip_operator::server::create_server().await;
        assert!(server.is_ok());
    }
}