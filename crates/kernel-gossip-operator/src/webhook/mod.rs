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
#[serde(rename_all = "snake_case")]
pub enum EbpfWebhookPayload {
    #[serde(rename = "cpu_throttle")]
    CpuThrottle {
        pod_name: String,
        namespace: String,
        container_name: String,
        throttle_percentage: f64,
        actual_cpu_usage: f64,
        reported_cpu_usage: f64,
        period_seconds: u64,
        ebpf_detection: bool,
        throttle_ns: u64,
        timestamp: String,
    },
    #[serde(rename = "pod_creation")]
    PodCreation {
        pod_name: String,
        namespace: String,
        total_syscalls: u64,
        namespace_ops: u64,
        cgroup_writes: u64,
        duration_ns: u64,
        timeline: Vec<TimelineEvent>,
        ebpf_detection: bool,
        timestamp: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub timestamp_ms: u64,
    pub action: String,
}

// Payload structs for actions module compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodCreationPayload {
    pub pod_name: String,
    pub namespace: String,
    pub total_syscalls: u64,
    pub namespace_ops: u64,
    pub cgroup_writes: u64,
    pub duration_ns: u64,
    pub timeline: Vec<TimelineEvent>,
    pub ebpf_detection: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuThrottlePayload {
    pub pod_name: String,
    pub namespace: String,
    pub container_name: String,
    pub throttle_percentage: f64,
    pub actual_cpu_usage: f64,
    pub reported_cpu_usage: f64,
    pub period_seconds: u64,
    pub ebpf_detection: bool,
    pub throttle_ns: u64,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
struct WebhookResponse {
    status: String,
    message: String,
}

pub fn create_webhook_router(client: Arc<Client>) -> Router {
    Router::new()
        .route("/webhook/ebpf", post(handle_ebpf_webhook))
        .with_state(client)
}

async fn handle_ebpf_webhook(
    State(client): State<Arc<Client>>,
    headers: HeaderMap,
    Json(payload): Json<EbpfWebhookPayload>,
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
        EbpfWebhookPayload::PodCreation { pod_name, namespace, total_syscalls, namespace_ops, cgroup_writes, duration_ns, timeline, ebpf_detection, timestamp } => {
            info!(
                "Received pod creation event for {}/{}",
                namespace, pod_name
            );
            
            // For pod creation events, we should create the certificate
            // The pod may not exist yet in K8s API when we receive the kernel event
            // We'll check annotations later if the pod exists, but for now
            // we should capture the kernel truth about pod creation
            
            // Skip system namespaces
            if namespace == "kube-system" || 
               namespace == "kube-public" || 
               namespace == "kube-node-lease" ||
               namespace == "gke-gmp-system" ||
               namespace == "gmp-system" ||
               namespace == "gke-managed-filestorecsi" {
                info!("Skipping system namespace pod {}/{}", 
                      namespace, pod_name);
                return Ok(Json(WebhookResponse {
                    status: "skipped".to_string(),
                    message: format!("System namespace pod {pod_name}"),
                }));
            }
            
            // Try to check annotations if pod exists, but don't fail if it doesn't
            use kube::api::Api;
            use k8s_openapi::api::core::v1::Pod;
            
            let pods: Api<Pod> = Api::namespaced(client.as_ref().clone(), &namespace);
            
            // Check if we should monitor this pod (default to true for pod creation)
            let should_monitor = match pods.get(&pod_name).await {
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
                          namespace, pod_name);
                    true
                }
            };
            
            if !should_monitor {
                info!("Pod {}/{} does not have monitoring annotation, skipping", 
                      namespace, pod_name);
                return Ok(Json(WebhookResponse {
                    status: "skipped".to_string(),
                    message: format!("Pod {pod_name} not configured for monitoring"),
                }));
            }
            
            // Create temporary payload structure for the function
            let payload = PodCreationPayload {
                pod_name,
                namespace,
                total_syscalls,
                namespace_ops,
                cgroup_writes,
                duration_ns,
                timeline,
                ebpf_detection,
                timestamp,
            };
            
            // Create PodBirthCertificate CRD
            match crate::actions::create_pod_birth_certificate(&client, &payload).await {
                Ok(pbc) => {
                    info!("Successfully created PodBirthCertificate: {:?}", pbc.metadata.name);
                }
                Err(e) => {
                    error!("Failed to create PodBirthCertificate: {}", e);
                    return Err(WebhookError(format!("Failed to create CRD: {e}")));
                }
            }
        }
        EbpfWebhookPayload::CpuThrottle { pod_name, namespace, container_name, throttle_percentage, actual_cpu_usage, reported_cpu_usage, period_seconds, ebpf_detection, throttle_ns, timestamp } => {
            info!(
                "Received CPU throttle event for {}/{}: {}%",
                namespace, pod_name, throttle_percentage
            );
            
            // Check if pod has monitoring annotation
            use kube::api::Api;
            use k8s_openapi::api::core::v1::Pod;
            
            let pods: Api<Pod> = Api::namespaced(client.as_ref().clone(), &namespace);
            
            // Try to get the pod and check its annotations
            match pods.get(&pod_name).await {
                Ok(pod) => {
                    let should_monitor = pod.metadata
                        .annotations
                        .as_ref()
                        .and_then(|ann| ann.get("kernel-gossip.io/monitor"))
                        .map(|v| v == "true")
                        .unwrap_or(false);
                    
                    if !should_monitor {
                        info!("Pod {}/{} does not have monitoring annotation, skipping", 
                              namespace, pod_name);
                        return Ok(Json(WebhookResponse {
                            status: "skipped".to_string(),
                            message: format!("Pod {pod_name} not configured for monitoring"),
                        }));
                    }
                }
                Err(e) => {
                    // Pod doesn't exist or is a system process - skip for non-pod processes
                    info!("Could not find pod {}/{}, likely a system process: {}", 
                          namespace, pod_name, e);
                    return Ok(Json(WebhookResponse {
                        status: "skipped".to_string(),
                        message: format!("Pod {pod_name} not found or is system process"),
                    }));
                }
            }
            
            // Create KernelWhisper CRD only for annotated pods
            match crate::actions::create_kernel_whisper(&client, &CpuThrottlePayload { pod_name, namespace, container_name, throttle_percentage, actual_cpu_usage, reported_cpu_usage, period_seconds, ebpf_detection, throttle_ns, timestamp }).await {
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