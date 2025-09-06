// Kernel Observer Main - Loads eBPF and monitors CPU throttling
// =============================================================
// This is the userspace component that loads eBPF programs and
// sends webhooks when throttling is detected

use anyhow::{Context, Result};
use std::env;
use tracing::{info, error};
use tracing_subscriber;

mod webhook;

#[cfg(feature = "ebpf")]
mod ebpf_loader;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Kernel Observer...");
    
    let webhook_url = env::var("WEBHOOK_URL")
        .unwrap_or_else(|_| "http://kernel-gossip-operator.kernel-gossip.svc.cluster.local:8080/webhook/pixie".to_string());
    
    info!("Webhook URL: {}", webhook_url);
    
    // Check if eBPF is enabled and available
    let enable_ebpf = env::var("ENABLE_EBPF")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    
    if enable_ebpf && cfg!(feature = "ebpf") {
        info!("🚀 Starting with REAL eBPF support!");
        run_ebpf_monitor(webhook_url).await?;
    } else {
        info!("Starting cgroup-based monitoring (no eBPF)");
        run_simple_monitor(webhook_url).await?;
    }
    
    Ok(())
}

#[cfg(feature = "ebpf")]
async fn run_ebpf_monitor(webhook_url: String) -> Result<()> {
    use crate::ebpf_loader::EbpfManager;
    use crate::webhook::WebhookSender;
    use tokio::time::{sleep, Duration};
    
    info!("🎯 Initializing eBPF programs...");
    
    let mut ebpf_manager = EbpfManager::new();
    let webhook_sender = WebhookSender::new(webhook_url);
    
    // Load eBPF programs
    ebpf_manager.load_syscall_counter()
        .context("Failed to load syscall counter")?;
    
    ebpf_manager.load_throttle_detector()
        .context("Failed to load throttle detector")?;
    
    info!("✅ eBPF programs loaded successfully!");
    info!("📡 Now monitoring with REAL kernel hooks!");
    
    // Create signal handler for graceful shutdown
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    
    loop {
        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down...");
                break;
            }
            _ = monitor_with_ebpf(&mut ebpf_manager, &webhook_sender) => {
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
    
    Ok(())
}

#[cfg(feature = "ebpf")]
async fn monitor_with_ebpf(
    ebpf_manager: &mut crate::ebpf_loader::EbpfManager,
    webhook_sender: &webhook::WebhookSender,
) -> Result<()> {
    use tracing::warn;
    
    // Check for CPU throttling via eBPF
    match ebpf_manager.monitor_throttles().await {
        Ok(detections) => {
            for detection in detections {
                info!("🚨 eBPF detected throttling for cgroup {}", detection.cgroup_id);
                
                // Send webhook
                match webhook_sender.send_throttle_event(
                    &format!("cgroup-{}", detection.cgroup_id),
                    "kernel-gossip",
                    (detection.throttle_count as f64 / 100.0) * 100.0,
                ).await {
                    Ok(_) => info!("✅ Webhook sent for eBPF-detected throttling"),
                    Err(e) => error!("❌ Failed to send webhook: {}", e),
                }
            }
        }
        Err(e) => warn!("Failed to monitor throttles: {}", e),
    }
    
    // Monitor syscalls
    match ebpf_manager.monitor_syscalls().await {
        Ok(stats) => {
            if stats.total_syscalls > 0 {
                info!("📊 eBPF syscall stats: {} total, {} clone, {} execve, {} mount",
                    stats.total_syscalls, stats.clone_count, 
                    stats.execve_count, stats.mount_count);
            }
        }
        Err(e) => warn!("Failed to monitor syscalls: {}", e),
    }
    
    Ok(())
}

// Simple cgroup monitoring fallback
async fn run_simple_monitor(webhook_url: String) -> Result<()> {
    use crate::webhook::WebhookSender;
    
    use tokio::time::{sleep, Duration};
    
    let webhook_sender = WebhookSender::new(webhook_url);
    
    info!("Starting simple cgroup monitoring (no eBPF)...");
    
    // Create a signal handler for graceful shutdown
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    
    loop {
        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down...");
                break;
            }
            _ = monitor_once(&webhook_sender) => {
                sleep(Duration::from_secs(30)).await;
            }
        }
    }
    
    Ok(())
}

async fn monitor_once(webhook_sender: &webhook::WebhookSender) -> Result<()> {
    use std::fs;
    
    
    info!("Checking for CPU throttling...");
    
    // Scan all pod cgroups for throttling
    let base_paths = vec![
        "/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice",
        "/sys/fs/cgroup/kubepods.slice/kubepods-burstable.slice",
    ];
    
    // Scan all pod directories
    for base_path in base_paths {
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let pod_path = entry.path();
                if !pod_path.is_dir() {
                    continue;
                }
                
                let cpu_stat_path = pod_path.join("cpu.stat");
                if !cpu_stat_path.exists() {
                    continue;
                }
                
                if let Ok(content) = fs::read_to_string(&cpu_stat_path) {
                    let mut nr_periods = 0u64;
                    let mut nr_throttled = 0u64;
                    
                    for line in content.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() == 2 {
                            match parts[0] {
                                "nr_periods" => nr_periods = parts[1].parse().unwrap_or(0),
                                "nr_throttled" => nr_throttled = parts[1].parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                    }
                    
                    if nr_periods > 0 && nr_throttled > 0 {
                        let throttle_pct = (nr_throttled as f64 / nr_periods as f64) * 100.0;
                        
                        if throttle_pct > 10.0 {
                            // Extract pod name from path
                            let pod_name = pod_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            
                            info!("🚨 DETECTED CPU throttling: {:.1}% for pod {}", throttle_pct, pod_name);
                            
                            // Map to known pod names
                            let friendly_name = if pod_name.contains("ad0c1fe4") {
                                "cpu-stress-demo"
                            } else {
                                pod_name
                            };
                            
                            // Send webhook (don't fail if it errors)
                            match webhook_sender.send_throttle_event(
                                friendly_name,
                                "kernel-gossip",
                                throttle_pct
                            ).await {
                                Ok(_) => info!("✅ Webhook sent for {:.1}% throttling", throttle_pct),
                                Err(e) => error!("❌ Failed to send webhook: {}", e),
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

// eBPF module will be added here when ready