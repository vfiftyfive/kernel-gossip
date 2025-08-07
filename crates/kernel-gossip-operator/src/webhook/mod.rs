use axum::{
    extract::Json,
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PixieWebhookPayload {
    PodCreation(PodCreationPayload),
    CpuThrottle(CpuThrottlePayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodCreationPayload {
    pub timestamp: String,
    pub pod_name: String,
    pub namespace: String,
    pub total_syscalls: u64,
    pub namespace_ops: u64,
    pub cgroup_writes: u64,
    pub duration_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuThrottlePayload {
    pub timestamp: String,
    pub pod_name: String,
    pub namespace: String,
    pub container_name: String,
    pub throttle_percentage: f64,
    pub actual_cpu_usage: f64,
    pub reported_cpu_usage: f64,
    pub period_seconds: u64,
}

#[derive(Debug, Serialize)]
struct WebhookResponse {
    status: String,
    message: String,
}

pub fn create_webhook_router() -> Router {
    Router::new()
        .route("/webhook/pixie", post(handle_pixie_webhook))
}

async fn handle_pixie_webhook(
    headers: HeaderMap,
    Json(payload): Json<PixieWebhookPayload>,
) -> Result<Json<WebhookResponse>, WebhookError> {
    // Validate content-type
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| WebhookError("Missing content-type header".to_string()))?;
    
    if !content_type.starts_with("application/json") {
        return Err(WebhookError("Invalid content-type".to_string()));
    }
    match payload {
        PixieWebhookPayload::PodCreation(data) => {
            info!(
                "Received pod creation event for {}/{}",
                data.namespace, data.pod_name
            );
            // TODO: Create PodBirthCertificate CRD
        }
        PixieWebhookPayload::CpuThrottle(data) => {
            info!(
                "Received CPU throttle event for {}/{}: {}%",
                data.namespace, data.pod_name, data.throttle_percentage
            );
            // TODO: Create KernelWhisper CRD
        }
    }

    Ok(Json(WebhookResponse {
        status: "accepted".to_string(),
        message: "Webhook payload processed".to_string(),
    }))
}

#[derive(Debug)]
struct WebhookError(String);

impl IntoResponse for WebhookError {
    fn into_response(self) -> Response {
        error!("Webhook error: {}", self.0);
        (
            StatusCode::BAD_REQUEST,
            Json(WebhookResponse {
                status: "error".to_string(),
                message: self.0,
            }),
        )
            .into_response()
    }
}