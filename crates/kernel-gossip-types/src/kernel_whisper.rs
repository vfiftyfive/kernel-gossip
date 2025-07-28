use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[kube(
    group = "kernel.gossip.io",
    version = "v1alpha1",
    kind = "KernelWhisper",
    plural = "kernelwhispers",
    shortname = "kw",
    namespaced
)]
pub struct KernelWhisperSpec {
    pub pod_name: String,
    pub namespace: String,
    pub detected_at: String,
    pub kernel_truth: KernelTruth,
    pub metrics_lie: MetricsLie,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KernelTruth {
    pub throttled_percent: f64,
    pub actual_cpu_cores: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetricsLie {
    pub cpu_percent: f64,
    pub reported_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

// Implementation methods - ONLY what's needed for tests
impl KernelWhisper {
    pub fn create(
        pod_name: &str,
        namespace: &str,
        throttled_percent: f64,
        cpu_percent: f64,
    ) -> Self {
        // Determine severity based on throttling
        let severity = if throttled_percent > 80.0 {
            Severity::Critical
        } else if throttled_percent > 50.0 {
            Severity::Warning
        } else {
            Severity::Info
        };

        Self {
            metadata: kube::api::ObjectMeta {
                name: Some(format!("{pod_name}-cpu-throttle")),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            spec: KernelWhisperSpec {
                pod_name: pod_name.to_string(),
                namespace: namespace.to_string(),
                detected_at: chrono::Utc::now().to_rfc3339(),
                kernel_truth: KernelTruth {
                    throttled_percent,
                    actual_cpu_cores: (100.0 - throttled_percent) / 100.0,
                },
                metrics_lie: MetricsLie {
                    cpu_percent,
                    reported_status: "healthy".to_string(),
                },
                severity,
            },
        }
    }

    pub fn api_version(&self) -> &str {
        "kernel.gossip.io/v1alpha1"
    }

    pub fn kind(&self) -> &str {
        "KernelWhisper"
    }

    pub fn pod_name(&self) -> &str {
        &self.spec.pod_name
    }

    pub fn namespace(&self) -> &str {
        &self.spec.namespace
    }

    pub fn kernel_truth(&self) -> &KernelTruth {
        &self.spec.kernel_truth
    }

    pub fn metrics_lie(&self) -> &MetricsLie {
        &self.spec.metrics_lie
    }
}
