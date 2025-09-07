use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[kube(
    group = "kernel.gossip.io",
    version = "v1alpha1",
    kind = "PodBirthCertificate",
    plural = "podbirthcertificates",
    shortname = "pbc",
    namespaced
)]
pub struct PodBirthCertificateSpec {
    pub pod_name: String,
    pub namespace: String,
    pub timeline: Vec<TimelineEntry>,
    pub kernel_stats: KernelStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TimelineEntry {
    pub timestamp_ms: u64,
    pub actor: Actor,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Actor {
    Scheduler,
    Kubelet,
    Runtime,
    Kernel,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KernelStats {
    pub total_syscalls: u32,
    pub namespaces_created: u8,
    pub cgroup_writes: u32,
    pub iptables_rules: u32,
    pub total_duration_ms: u64,
}

// Implementation methods - ONLY what's needed for tests
impl PodBirthCertificate {
    pub fn create(pod_name: &str, namespace: &str) -> Self {
        Self {
            metadata: kube::api::ObjectMeta {
                name: Some(format!("{pod_name}-birth")),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            spec: PodBirthCertificateSpec {
                pod_name: pod_name.to_string(),
                namespace: namespace.to_string(),
                timeline: vec![],
                kernel_stats: KernelStats {
                    total_syscalls: 0,
                    namespaces_created: 0,
                    cgroup_writes: 0,
                    iptables_rules: 0,
                    total_duration_ms: 0,
                },
            },
        }
    }

    pub fn api_version(&self) -> &str {
        "kernel.gossip.io/v1alpha1"
    }

    pub fn kind(&self) -> &str {
        "PodBirthCertificate"
    }

    pub fn pod_name(&self) -> &str {
        &self.spec.pod_name
    }

    pub fn namespace(&self) -> &str {
        &self.spec.namespace
    }
}

impl TimelineEntry {
    pub fn new(timestamp_ms: u64, actor: Actor, action: &str) -> Self {
        Self {
            timestamp_ms,
            actor,
            action: action.to_string(),
        }
    }

    pub fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms
    }

    pub fn actor(&self) -> &Actor {
        &self.actor
    }

}
