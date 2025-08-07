#[cfg(test)]
mod webhook_handler_tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::json;

    #[tokio::test]
    async fn test_webhook_handler_pod_creation() {
        // Create test app with webhook route
        let app = kernel_gossip_operator::webhook::create_webhook_router();

        let payload = json!({
            "type": "pod_creation",
            "timestamp": "2024-01-01T00:00:00Z",
            "pod_name": "test-pod",
            "namespace": "default",
            "total_syscalls": 1234,
            "namespace_ops": 56,
            "cgroup_writes": 78,
            "duration_ns": 1000000
        });

        let request = Request::builder()
            .method("POST")
            .uri("/webhook/pixie")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("accepted"));
    }

    #[tokio::test]
    async fn test_webhook_handler_cpu_throttle() {
        let app = kernel_gossip_operator::webhook::create_webhook_router();

        let payload = json!({
            "type": "cpu_throttle",
            "timestamp": "2024-01-01T00:00:00Z",
            "pod_name": "throttled-pod",
            "namespace": "production",
            "container_name": "app",
            "throttle_percentage": 45.5,
            "actual_cpu_usage": 0.8,
            "reported_cpu_usage": 0.5,
            "period_seconds": 300
        });

        let request = Request::builder()
            .method("POST")
            .uri("/webhook/pixie")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_webhook_handler_invalid_payload() {
        let app = kernel_gossip_operator::webhook::create_webhook_router();

        let request = Request::builder()
            .method("POST")
            .uri("/webhook/pixie")
            .header("content-type", "application/json")
            .body(Body::from("invalid json"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_webhook_handler_missing_content_type() {
        let app = kernel_gossip_operator::webhook::create_webhook_router();

        let payload = json!({
            "type": "pod_creation",
            "timestamp": "2024-01-01T00:00:00Z",
            "pod_name": "test-pod",
            "namespace": "default"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/webhook/pixie")
            // No content-type header
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
}