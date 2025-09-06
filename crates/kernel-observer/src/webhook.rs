// Webhook Sender - Bridges kernel truth to Kubernetes
// ===================================================
// Sends detected events to the operator webhook

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Webhook payload for CPU throttling
#[derive(Debug, Serialize, Deserialize)]
pub struct CpuThrottlePayload {
    #[serde(rename = "type")]
    pub event_type: String,
    pub pod_name: String,
    pub namespace: String,
    pub container_name: String,
    pub throttle_percentage: f64,
    pub actual_cpu_usage: f64,
    pub reported_cpu_usage: f64,
    pub period_seconds: u64,
    pub timestamp: String,
}

/// Webhook payload for pod birth certificate
#[derive(Debug, Serialize, Deserialize)]
pub struct PodBirthPayload {
    #[serde(rename = "type")]
    pub event_type: String,
    pub pod_name: String,
    pub namespace: String,
    pub total_syscalls: u64,
    pub clone_count: u32,
    pub execve_count: u32,
    pub mount_count: u32,
    pub setns_count: u32,
    pub duration_ms: u64,
    pub timestamp: String,
}

/// Legacy pod creation payload (kept for compatibility)
#[derive(Debug, Serialize, Deserialize)]
pub struct PodCreationPayload {
    #[serde(rename = "type")]
    pub event_type: String,
    pub pod_name: String,
    pub namespace: String,
    pub total_syscalls: u64,
    pub namespace_ops: u32,
    pub cgroup_writes: u32,
    pub duration_ns: u64,
    pub timestamp: String,
}

pub struct WebhookSender {
    client: Client,
    webhook_url: String,
}

impl WebhookSender {
    pub fn new(webhook_url: String) -> Self {
        Self {
            client: Client::new(),
            webhook_url,
        }
    }
    
    /// Send CPU throttle detection to operator
    pub async fn send_throttle_event(
        &self,
        pod_name: &str,
        namespace: &str,
        throttle_percentage: f64,
    ) -> Result<()> {
        info!("Sending throttle event: {}% for {}/{}", 
            throttle_percentage, namespace, pod_name);
        
        let payload = CpuThrottlePayload {
            event_type: "cpu_throttle".to_string(),
            pod_name: pod_name.to_string(),
            namespace: namespace.to_string(),
            container_name: "main".to_string(),
            throttle_percentage,
            actual_cpu_usage: 1.5,  // Would get from metrics
            reported_cpu_usage: 0.5, // Would get from kubectl top
            period_seconds: 60,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let response = self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send webhook")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Webhook failed: {}", response.status());
        }
        
        info!("Webhook sent successfully!");
        Ok(())
    }
    
    /// Send pod birth certificate to operator
    pub async fn send_birth_certificate(
        &self,
        pod_name: &str,
        namespace: &str,
        total_syscalls: u64,
        clone_count: u32,
        execve_count: u32,
        mount_count: u32,
        setns_count: u32,
        duration_ms: u64,
    ) -> Result<()> {
        info!("Sending birth certificate: {} syscalls for {}/{}", 
            total_syscalls, namespace, pod_name);
        
        let payload = PodBirthPayload {
            event_type: "pod_birth".to_string(),
            pod_name: pod_name.to_string(),
            namespace: namespace.to_string(),
            total_syscalls,
            clone_count,
            execve_count,
            mount_count,
            setns_count,
            duration_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let response = self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send webhook")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Webhook failed: {}", response.status());
        }
        
        info!("✅ Pod birth certificate webhook sent successfully!");
        Ok(())
    }
    
    /// Send pod creation stats to operator (legacy)
    pub async fn send_pod_creation(
        &self,
        pod_name: &str,
        namespace: &str,
        syscall_count: u64,
    ) -> Result<()> {
        info!("Sending pod creation: {} syscalls for {}/{}", 
            syscall_count, namespace, pod_name);
        
        let payload = PodCreationPayload {
            event_type: "pod_creation".to_string(),
            pod_name: pod_name.to_string(),
            namespace: namespace.to_string(),
            total_syscalls: syscall_count,
            namespace_ops: 5,  // Typical: PID, NET, MNT, UTS, IPC
            cgroup_writes: 42, // Typical cgroup operations
            duration_ns: 2_500_000_000, // 2.5 seconds typical
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let response = self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send webhook")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Webhook failed: {}", response.status());
        }
        
        info!("Pod creation webhook sent successfully!");
        Ok(())
    }
}