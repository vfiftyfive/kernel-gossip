use axum::{
    extract::{Json, State},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use kube::Client;
use std::sync::Arc;

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

pub fn create_webhook_router(client: Arc<Client>) -> Router {
    Router::new()
        .route("/webhook/pixie", post(handle_pixie_webhook))
        .with_state(client)
}

async fn handle_pixie_webhook(
    State(client): State<Arc<Client>>,
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
            
            // For pod creation events, we should create the certificate
            // The pod may not exist yet in K8s API when we receive the kernel event
            // We'll check annotations later if the pod exists, but for now
            // we should capture the kernel truth about pod creation
            
            // Skip system namespaces
            if data.namespace == "kube-system" || 
               data.namespace == "kube-public" || 
               data.namespace == "kube-node-lease" ||
               data.namespace == "gke-gmp-system" ||
               data.namespace == "gmp-system" ||
               data.namespace == "gke-managed-filestorecsi" {
                info!("Skipping system namespace pod {}/{}", 
                      data.namespace, data.pod_name);
                return Ok(Json(WebhookResponse {
                    status: "skipped".to_string(),
                    message: format!("System namespace pod {}", data.pod_name),
                }));
            }
            
            // Try to check annotations if pod exists, but don't fail if it doesn't
            use kube::api::{Api, ObjectMeta};
            use k8s_openapi::api::core::v1::Pod;
            
            let pods: Api<Pod> = Api::namespaced(client.as_ref().clone(), &data.namespace);
            
            // Check if we should monitor this pod (default to true for pod creation)
            let should_monitor = match pods.get(&data.pod_name).await {
                Ok(pod) => {
                    // Pod exists, check annotation
                    pod.metadata
                        .annotations
                        .as_ref()
                        .and_then(|ann| ann.get("kernel-gossip.io/monitor"))
                        .map(|v| v == "true")
                        .unwrap_or(false)
                }
                Err(_) => {
                    // Pod doesn't exist yet - this is expected for creation events
                    // Default to creating certificate for non-system namespaces
                    info!("Pod {}/{} not found yet (expected for creation event), creating certificate", 
                          data.namespace, data.pod_name);
                    true
                }
            };
            
            if !should_monitor {
                info!("Pod {}/{} does not have monitoring annotation, skipping", 
                      data.namespace, data.pod_name);
                return Ok(Json(WebhookResponse {
                    status: "skipped".to_string(),
                    message: format!("Pod {} not configured for monitoring", data.pod_name),
                }));
            }
            
            // Create PodBirthCertificate CRD
            match crate::actions::create_pod_birth_certificate(&client, &data).await {
                Ok(pbc) => {
                    info!("Successfully created PodBirthCertificate: {:?}", pbc.metadata.name);
                }
                Err(e) => {
                    error!("Failed to create PodBirthCertificate: {}", e);
                    return Err(WebhookError(format!("Failed to create CRD: {e}")));
                }
            }
        }
        PixieWebhookPayload::CpuThrottle(data) => {
            info!(
                "Received CPU throttle event for {}/{}: {}%",
                data.namespace, data.pod_name, data.throttle_percentage
            );
            
            // Check if pod has monitoring annotation
            use kube::api::{Api, ObjectMeta};
            use k8s_openapi::api::core::v1::Pod;
            
            let pods: Api<Pod> = Api::namespaced(client.as_ref().clone(), &data.namespace);
            
            // Try to get the pod and check its annotations
            match pods.get(&data.pod_name).await {
                Ok(pod) => {
                    let should_monitor = pod.metadata
                        .annotations
                        .as_ref()
                        .and_then(|ann| ann.get("kernel-gossip.io/monitor"))
                        .map(|v| v == "true")
                        .unwrap_or(false);
                    
                    if !should_monitor {
                        info!("Pod {}/{} does not have monitoring annotation, skipping", 
                              data.namespace, data.pod_name);
                        return Ok(Json(WebhookResponse {
                            status: "skipped".to_string(),
                            message: format!("Pod {} not configured for monitoring", data.pod_name),
                        }));
                    }
                }
                Err(e) => {
                    // Pod doesn't exist or is a system process - skip for non-pod processes
                    info!("Could not find pod {}/{}, likely a system process: {}", 
                          data.namespace, data.pod_name, e);
                    return Ok(Json(WebhookResponse {
                        status: "skipped".to_string(),
                        message: format!("Pod {} not found or is system process", data.pod_name),
                    }));
                }
            }
            
            // Create KernelWhisper CRD only for annotated pods
            match crate::actions::create_kernel_whisper(&client, &data).await {
                Ok(kw) => {
                    info!("Successfully created KernelWhisper: {:?}", kw.metadata.name);
                }
                Err(e) => {
                    error!("Failed to create KernelWhisper: {}", e);
                    return Err(WebhookError(format!("Failed to create CRD: {e}")));
                }
            }
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