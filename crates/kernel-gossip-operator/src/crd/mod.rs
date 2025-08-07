use std::sync::Arc;
use kube::{
    runtime::controller::{Action, Controller},
    Api, Client, ResourceExt,
};
use futures::StreamExt;
use tokio::time::Duration;
use tracing::{error, info, warn};
use chrono;
use kernel_gossip_types::{PodBirthCertificate, KernelWhisper, Severity};
use crate::recommendation::{RecommendationEngine, Recommendation};

// Helper functions for unit testing
pub fn reconcile_logic_pod_birth(pbc: &PodBirthCertificate) -> Result<(), String> {
    let name = pbc.name_any();
    if name.is_empty() {
        return Err("PodBirthCertificate has no name".to_string());
    }
    Ok(())
}

pub struct ReconcileAction {
    severity: String,
    needs_attention: bool,
}

impl ReconcileAction {
    pub fn severity_level(&self) -> &str {
        &self.severity
    }
    
    pub fn requires_attention(&self) -> bool {
        self.needs_attention
    }
}

pub fn reconcile_logic_kernel_whisper(kw: &KernelWhisper) -> ReconcileAction {
    let (severity, needs_attention) = match kw.spec.severity {
        Severity::Critical => ("critical", true),
        Severity::Warning => ("warning", false),
        Severity::Info => ("info", false),
    };
    
    ReconcileAction {
        severity: severity.to_string(),
        needs_attention,
    }
}

pub fn calculate_requeue_duration(severity: &Severity) -> u64 {
    match severity {
        Severity::Critical => 60,    // 1 minute
        Severity::Warning => 180,    // 3 minutes
        Severity::Info => 600,       // 10 minutes
    }
}

pub struct PodBirthCertificateController;
pub struct KernelWhisperController;

// Context passed to reconcile functions
#[derive(Clone)]
pub struct Context {
    pub client: Client,
}

// Reconcile function for PodBirthCertificate
pub async fn reconcile_pod_birth_certificate(
    pbc: Arc<PodBirthCertificate>,
    _ctx: Arc<Context>,
) -> Result<Action, Error> {
    let name = pbc.name_any();
    
    // Validate the resource
    if name.is_empty() {
        warn!("PodBirthCertificate has no name, requeueing with backoff");
        return Ok(Action::requeue(Duration::from_secs(30)));
    }
    
    info!("Reconciling PodBirthCertificate: {}", name);
    
    // For now, just log the timeline
    for entry in &pbc.spec.timeline {
        info!(
            "Timeline entry at {}: {} by {:?}",
            entry.timestamp_ms, entry.action, entry.actor
        );
    }
    
    info!(
        "Kernel stats - syscalls: {}, namespaces: {}, cgroups: {}, duration: {}ms",
        pbc.spec.kernel_stats.total_syscalls,
        pbc.spec.kernel_stats.namespaces_created,
        pbc.spec.kernel_stats.cgroup_writes,
        pbc.spec.kernel_stats.total_duration_ms
    );
    
    // Requeue after 5 minutes to check for updates
    Ok(Action::requeue(Duration::from_secs(300)))
}

// Reconcile function for KernelWhisper
pub async fn reconcile_kernel_whisper(
    kw: Arc<KernelWhisper>,
    _ctx: Arc<Context>,
) -> Result<Action, Error> {
    let name = kw.name_any();
    info!("Reconciling KernelWhisper: {} with severity {:?}", name, kw.spec.severity);
    
    // Generate recommendations using the recommendation engine
    let recommendation_engine = RecommendationEngine::new();
    if let Some(recommendation) = recommendation_engine.analyze_kernel_whisper(&kw) {
        info!(
            "📊 INSIGHT: {} - Priority: {}",
            recommendation.insight, recommendation.priority
        );
        info!("💡 RECOMMENDATION: {}", recommendation.suggested_action);
        info!("🔍 KERNEL EVIDENCE: {}", recommendation.kernel_evidence);
        
        // Update CRD status with recommendation
        let status_message = build_status_update(&recommendation);
        if let Err(e) = update_kernel_whisper_status(&_ctx.client, &kw, &status_message).await {
            warn!("Failed to update KernelWhisper status: {}", e);
        }
    } else {
        // No recommendation needed - update status with healthy state
        let status_message = build_status_update_no_action("Pod operating within normal parameters");
        if let Err(e) = update_kernel_whisper_status(&_ctx.client, &kw, &status_message).await {
            warn!("Failed to update KernelWhisper status: {}", e);
        }
    }
    
    // Log based on severity for immediate visibility
    match kw.spec.severity {
        Severity::Critical => {
            warn!(
                "CRITICAL: Pod {} is experiencing {}% CPU throttling!",
                kw.spec.pod_name, kw.spec.kernel_truth.throttled_percent
            );
        }
        Severity::Warning => {
            warn!(
                "WARNING: Pod {} is experiencing {}% CPU throttling",
                kw.spec.pod_name, kw.spec.kernel_truth.throttled_percent
            );
            // Monitor but don't take immediate action
        }
        Severity::Info => {
            info!(
                "INFO: Pod {} has minor CPU throttling ({}%)",
                kw.spec.pod_name, kw.spec.kernel_truth.throttled_percent
            );
            // Just log for visibility
        }
    }
    
    // Requeue based on severity
    let requeue_duration = match kw.spec.severity {
        Severity::Critical => Duration::from_secs(60),    // Check every minute
        Severity::Warning => Duration::from_secs(180),    // Check every 3 minutes
        Severity::Info => Duration::from_secs(600),       // Check every 10 minutes
    };
    
    Ok(Action::requeue(requeue_duration))
}

// Error handler
fn error_policy(_pbc: Arc<PodBirthCertificate>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("Reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(60))
}

// Error handler for KernelWhisper
fn error_policy_kw(_kw: Arc<KernelWhisper>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("KernelWhisper reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(60))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),
    
    #[error("Invalid resource: {0}")]
    InvalidResource(String),
}

// Start the controllers
pub async fn run_controllers(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Arc::new(Context { client: client.clone() });
    
    // PodBirthCertificate controller
    let pbc_api: Api<PodBirthCertificate> = Api::all(client.clone());
    let pbc_controller = Controller::new(pbc_api, Default::default())
        .run(reconcile_pod_birth_certificate, error_policy, ctx.clone())
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled PodBirthCertificate: {:?}", o),
                Err(e) => error!("PodBirthCertificate reconciliation failed: {:?}", e),
            }
        });
    
    // KernelWhisper controller
    let kw_api: Api<KernelWhisper> = Api::all(client.clone());
    let kw_controller = Controller::new(kw_api, Default::default())
        .run(reconcile_kernel_whisper, error_policy_kw, ctx)
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled KernelWhisper: {:?}", o),
                Err(e) => error!("KernelWhisper reconciliation failed: {:?}", e),
            }
        });
    
    // Run both controllers concurrently
    tokio::select! {
        _ = pbc_controller => {},
        _ = kw_controller => {},
    }
    
    Ok(())
}

// Status update functions for CRDs
pub fn build_status_update(recommendation: &Recommendation) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    format!(
        "🚨 INSIGHT: {} | 💡 ACTION: {} | 🔍 EVIDENCE: {} | ⚡ PRIORITY: {} | 🕐 UPDATED: {}",
        recommendation.insight,
        recommendation.suggested_action,
        recommendation.kernel_evidence,
        recommendation.priority,
        timestamp
    )
}

pub fn build_status_update_no_action(message: &str) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    format!(
        "✅ STATUS: {} - System is healthy | 🕐 UPDATED: {}",
        message,
        timestamp
    )
}

// Placeholder for actual K8s status update (will implement when we have K8s access)
pub async fn update_kernel_whisper_status(
    _client: &Client,
    _kw: &KernelWhisper,
    _status_message: &str,
) -> Result<(), Error> {
    // TODO: Implement actual K8s status update
    // This would update the KernelWhisper CRD status field with the status_message
    info!("Would update CRD status with: {}", _status_message);
    Ok(())
}

