use kube::{
    Api, Client,
    runtime::watcher::{Config, Event, watcher},
};
use k8s_openapi::api::core::v1::{Pod, ConfigMap};
use futures::StreamExt;
use std::collections::BTreeMap;
use tracing::{info, warn};

/// Watches for pods with kernel-gossip.io/monitor annotation
/// and maintains a ConfigMap with their details
pub async fn run_pod_watcher(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    let pod_api: Api<Pod> = Api::namespaced(client.clone(), "kernel-gossip");
    let cm_api: Api<ConfigMap> = Api::namespaced(client.clone(), "kernel-gossip");
    
    // Create initial ConfigMap if it doesn't exist
    ensure_configmap_exists(&cm_api).await?;
    
    // Watch for pods with our annotation
    let config = Config::default()
        .labels("kernel-gossip.io/monitor=true");
    
    let mut stream = watcher(pod_api, config).boxed();
    
    while let Some(event) = stream.next().await {
        match event {
            Ok(Event::Applied(pod)) => {
                info!("Pod {} applied/modified, updating ConfigMap", pod.metadata.name.as_ref().unwrap_or(&"unknown".to_string()));
                update_monitored_pods(&cm_api, &client).await?;
            }
            Ok(Event::Deleted(pod)) => {
                info!("Pod {} deleted, updating ConfigMap", pod.metadata.name.as_ref().unwrap_or(&"unknown".to_string()));
                update_monitored_pods(&cm_api, &client).await?;
            }
            Ok(Event::Restarted(pods)) => {
                info!("Pod watcher restarted, {} pods in initial state", pods.len());
                update_monitored_pods(&cm_api, &client).await?;
            }
            Err(e) => {
                warn!("Pod watcher error: {}", e);
            }
        }
    }
    
    Ok(())
}

async fn ensure_configmap_exists(cm_api: &Api<ConfigMap>) -> Result<(), kube::Error> {
    let cm_name = "ebpf-monitored-pods";
    
    // Check if ConfigMap exists
    if cm_api.get_opt(cm_name).await?.is_some() {
        return Ok(());
    }
    
    // Create ConfigMap
    let cm = ConfigMap {
        metadata: kube::api::ObjectMeta {
            name: Some(cm_name.to_string()),
            namespace: Some("kernel-gossip".to_string()),
            ..Default::default()
        },
        data: Some(BTreeMap::from([
            ("pods".to_string(), "".to_string()),
        ])),
        ..Default::default()
    };
    
    cm_api.create(&Default::default(), &cm).await?;
    info!("Created ConfigMap for monitored pods");
    Ok(())
}

async fn update_monitored_pods(
    cm_api: &Api<ConfigMap>, 
    client: &Client
) -> Result<(), Box<dyn std::error::Error>> {
    let pod_api: Api<Pod> = Api::namespaced(client.clone(), "kernel-gossip");
    
    // Get all pods with annotation
    let pods = pod_api.list(&Default::default()).await?;
    
    let mut monitored_pods = Vec::new();
    
    for pod in pods.items {
        // Check for annotation
        if let Some(annotations) = &pod.metadata.annotations {
            if annotations.get("kernel-gossip.io/monitor") == Some(&"true".to_string()) {
                let pod_name = pod.metadata.name.unwrap_or_default();
                let pod_uid = pod.metadata.uid.unwrap_or_default();
                
                // Format: pod_name:pod_uid
                monitored_pods.push(format!("{pod_name}:{pod_uid}"));
                info!("Monitoring pod: {} (UID: {})", pod_name, pod_uid);
            }
        }
    }
    
    // Update ConfigMap
    let mut cm = cm_api.get("ebpf-monitored-pods").await?;
    cm.data = Some(BTreeMap::from([
        ("pods".to_string(), monitored_pods.join(",")),
        ("count".to_string(), monitored_pods.len().to_string()),
        ("updated".to_string(), chrono::Utc::now().to_rfc3339()),
    ]));
    
    cm_api.replace("ebpf-monitored-pods", &Default::default(), &cm).await?;
    info!("Updated ConfigMap with {} monitored pods", monitored_pods.len());
    
    Ok(())
}