mod bpftrace;
mod parser;
mod webhook;
mod config;
mod pod_resolver;
// Removed unused modules: pod_uid_extractor, syscall_tracker, cgroup_tracker

use anyhow::Result;
use tracing::{info, error};

use crate::bpftrace::BpftraceProcess;
use crate::parser::EbpfParser;
use crate::webhook::WebhookClient;
use crate::config::Config;
use std::fs;

fn load_bpftrace_script() -> Result<String> {
    // Load script from mounted ConfigMap - fully configurable via env var
    let script_name = std::env::var("BPFTRACE_SCRIPT").unwrap_or_else(|_| "monitoring.bt".to_string());
    let script_path = format!("/etc/bpftrace-scripts/{}", script_name);
    
    let script_content = fs::read_to_string(&script_path)
        .map_err(|e| anyhow::anyhow!("Failed to load bpftrace script from {}: {}", script_path, e))?;
    
    info!("📁 Loaded bpftrace script from: {} ({} bytes)", script_path, script_content.len());
    Ok(script_content)
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    info!("🚀 Starting Kernel Observer with real eBPF monitoring");
    info!("Webhook URL: {}", config.webhook_url);

    let parser = EbpfParser::new(config.webhook_url.clone()).await?;
    let webhook_client = WebhookClient::new(config.webhook_url);

    // Load and spawn bpftrace process
    let script = load_bpftrace_script()?;
    let mut bpftrace = BpftraceProcess::spawn(&script).await?;
    info!("✅ bpftrace process spawned successfully");

    // Start cleanup task for old syscall sessions
    let parser_cleanup = parser.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            parser_cleanup.cleanup_old_sessions().await;
        }
    });

    // Process eBPF output in real-time
    while let Some(line) = bpftrace.next_line().await? {
        info!("eBPF: {}", line);
        
        // Check for stderr messages
        bpftrace.check_stderr().await?;
        
        if let Some(event) = parser.parse_line(&line).await? {
            if let Err(e) = webhook_client.send_event(event).await {
                error!("Failed to send webhook: {}", e);
            }
        }
    }

    // Wait for bpftrace to complete
    bpftrace.wait().await?;
    Ok(())
}
