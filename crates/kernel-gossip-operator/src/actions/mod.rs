use anyhow::Result;
use kube::{Api, Client};
use kernel_gossip_types::{
    PodBirthCertificate, TimelineEntry, KernelStats, Actor,
    KernelWhisper, KernelTruth, MetricsLie,
};
use crate::webhook::{PodCreationPayload, CpuThrottlePayload};
use tracing::info;

// Builder functions for unit testing
pub fn build_pod_birth_certificate(payload: &PodCreationPayload) -> PodBirthCertificate {
    let pbc_name = format!("{}-pbc", payload.pod_name);
    
    // Convert timestamp to milliseconds
    let timestamp_ms = chrono::DateTime::parse_from_rfc3339(&payload.timestamp)
        .unwrap_or_else(|_| chrono::Utc::now().into())
        .timestamp_millis() as u64;
    
    let timeline = vec![
        TimelineEntry {
            timestamp_ms,
            actor: Actor::Kernel,
            action: "Pod creation started".to_string(),
            details: Some(format!("Total syscalls: {}", payload.total_syscalls)),
        },
    ];

    let kernel_stats = KernelStats {
        total_syscalls: payload.total_syscalls as u32,
        namespaces_created: 1, // One namespace per pod
        cgroup_writes: payload.cgroup_writes as u32,
        iptables_rules: 0, // Not tracked in payload
        total_duration_ms: payload.duration_ns / 1_000_000, // Convert ns to ms
    };

    let mut pbc = PodBirthCertificate::create(&payload.pod_name, &payload.namespace);
    pbc.spec.timeline = timeline;
    pbc.spec.kernel_stats = kernel_stats;
    pbc.metadata.name = Some(pbc_name);
    
    pbc
}

pub fn build_kernel_whisper(payload: &CpuThrottlePayload) -> KernelWhisper {
    let kw_name = format!("{}-kw", payload.pod_name);
    
    let kernel_truth = KernelTruth {
        throttled_percent: payload.throttle_percentage,
        actual_cpu_cores: payload.actual_cpu_usage,
    };

    let metrics_lie = MetricsLie {
        cpu_percent: payload.reported_cpu_usage * 100.0, // Convert to percentage
        reported_status: "Healthy".to_string(), // Metrics always report healthy
    };

    // Use the create method properly
    let mut kw = KernelWhisper::create(
        &payload.pod_name,
        &payload.namespace,
        payload.throttle_percentage,
        payload.reported_cpu_usage * 100.0,
    );
    
    // Override the auto-generated name and update fields
    kw.metadata.name = Some(kw_name);
    kw.spec.detected_at = payload.timestamp.clone();
    kw.spec.kernel_truth = kernel_truth;
    kw.spec.metrics_lie = metrics_lie;
    
    kw
}

pub async fn create_pod_birth_certificate(
    client: &Client,
    payload: &PodCreationPayload,
) -> Result<PodBirthCertificate> {
    let api: Api<PodBirthCertificate> = Api::namespaced(
        client.clone(),
        &payload.namespace,
    );

    let pbc = build_pod_birth_certificate(payload);

    let result = api.create(&Default::default(), &pbc).await?;
    
    info!(
        "Created PodBirthCertificate {}/{}", 
        payload.namespace, pbc.metadata.name.as_ref().unwrap()
    );

    Ok(result)
}

pub async fn create_kernel_whisper(
    client: &Client,
    payload: &CpuThrottlePayload,
) -> Result<KernelWhisper> {
    let api: Api<KernelWhisper> = Api::namespaced(
        client.clone(),
        &payload.namespace,
    );

    let kw = build_kernel_whisper(payload);

    let result = api.create(&Default::default(), &kw).await?;
    
    info!(
        "Created KernelWhisper {}/{} with severity {:?}", 
        payload.namespace, kw.metadata.name.as_ref().unwrap(), kw.spec.severity
    );

    Ok(result)
}