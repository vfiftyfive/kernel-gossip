// Test CPU Throttle Detection - Simple Demo
// ==========================================
// This reads actual cgroup throttling data and sends webhooks
// Demonstrates the full pipeline without needing eBPF on macOS

use anyhow::Result;
use kernel_observer::webhook::WebhookSender;
use std::fs;
use std::path::Path;
use tracing::{info, warn};
use tracing_subscriber;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting CPU throttle detection test...");
    
    // Create webhook sender
    let webhook_url = std::env::var("WEBHOOK_URL")
        .unwrap_or_else(|_| "http://localhost:8080/webhook/pixie".to_string());
    
    info!("Using webhook URL: {}", webhook_url);
    
    // Create async runtime
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        test_cpu_throttle_detection(webhook_url).await
    })
}

async fn test_cpu_throttle_detection(webhook_url: String) -> Result<()> {
    let webhook_sender = WebhookSender::new(webhook_url);
    
    // Find cgroup paths for our stress pod
    let cgroup_base = "/sys/fs/cgroup";
    
    // Look for cpu-stress pods
    info!("Searching for CPU stress pods...");
    
    // Try to find kubepods cgroup
    let kubepods_path = format!("{}/kubepods.slice", cgroup_base);
    if !Path::new(&kubepods_path).exists() {
        warn!("kubepods.slice not found, trying alternative paths...");
        
        // On GKE, might be directly under /sys/fs/cgroup
        let alt_path = format!("{}/cpu.stat", cgroup_base);
        if Path::new(&alt_path).exists() {
            check_throttling(&alt_path, "system", &webhook_sender).await?;
        }
    }
    
    // Try to find specific pod cgroups
    find_and_check_pod_cgroups(cgroup_base, &webhook_sender).await?;
    
    Ok(())
}

async fn find_and_check_pod_cgroups(base: &str, webhook_sender: &WebhookSender) -> Result<()> {
    // Walk through cgroup filesystem looking for pods
    let paths = fs::read_dir(base)?;
    
    for entry in paths {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Look for kubepods or pod-specific cgroups
            if name.contains("kubepods") || name.contains("pod") {
                info!("Found potential pod cgroup: {}", path.display());
                
                // Check for cpu.stat file
                let cpu_stat_path = path.join("cpu.stat");
                if cpu_stat_path.exists() {
                    check_throttling(
                        cpu_stat_path.to_str().unwrap(),
                        name,
                        webhook_sender
                    ).await?;
                }
                
                // Recurse into subdirectories - use Box::pin for async recursion
                let path_str = path.to_str().unwrap().to_string();
                let _ = Box::pin(find_and_check_pod_cgroups(&path_str, webhook_sender)).await;
            }
        }
    }
    
    Ok(())
}

async fn check_throttling(path: &str, pod_name: &str, webhook_sender: &WebhookSender) -> Result<()> {
    info!("Checking throttling at: {}", path);
    
    let content = fs::read_to_string(path)?;
    
    let mut nr_periods = 0u64;
    let mut nr_throttled = 0u64;
    let mut throttled_time = 0u64;
    
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            match parts[0] {
                "nr_periods" => nr_periods = parts[1].parse().unwrap_or(0),
                "nr_throttled" => nr_throttled = parts[1].parse().unwrap_or(0),
                "throttled_time" => throttled_time = parts[1].parse().unwrap_or(0),
                _ => {}
            }
        }
    }
    
    if nr_periods > 0 {
        let throttle_percentage = (nr_throttled as f64 / nr_periods as f64) * 100.0;
        
        if throttle_percentage > 0.0 {
            info!("🚨 DETECTED CPU THROTTLING!");
            info!("  Pod/Container: {}", pod_name);
            info!("  Throttle percentage: {:.2}%", throttle_percentage);
            info!("  Throttled periods: {}/{}", nr_throttled, nr_periods);
            info!("  Throttled time: {} ns", throttled_time);
            
            // Send webhook
            info!("Sending webhook to operator...");
            
            // Extract pod name from cgroup path if possible
            let clean_pod_name = if pod_name.contains("cpu-stress") {
                "cpu-stress-demo"
            } else {
                pod_name
            };
            
            webhook_sender.send_throttle_event(
                clean_pod_name,
                "kernel-gossip",
                throttle_percentage
            ).await?;
            
            info!("✅ Webhook sent successfully!");
        } else {
            info!("No throttling detected for {}", pod_name);
        }
    } else {
        info!("No CPU stats available for {}", pod_name);
    }
    
    Ok(())
}