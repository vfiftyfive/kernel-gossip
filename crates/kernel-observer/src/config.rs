use std::env;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub webhook_url: String,
    #[allow(dead_code)]
    pub namespace: String,
    #[allow(dead_code)]
    pub configmap_name: String,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            webhook_url: env::var("WEBHOOK_URL")
                .unwrap_or_else(|_| "http://kernel-gossip-operator.kernel-gossip.svc.cluster.local:8080/webhook/pixie".to_string()),
            namespace: env::var("NAMESPACE").unwrap_or_else(|_| "kernel-gossip".to_string()),
            configmap_name: env::var("CONFIGMAP_NAME").unwrap_or_else(|_| "ebpf-monitored-pods".to_string()),
            log_level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        })
    }
}