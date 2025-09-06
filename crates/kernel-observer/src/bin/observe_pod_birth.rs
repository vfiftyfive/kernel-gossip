// Observe Pod Birth - Watch the kernel cascade in real-time!
// ===========================================================

use kernel_observer::pod_lifecycle_observer::{PodLifecycleObserver, PodLifecycleEvents};
use std::env;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let pod_uid = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: observe_pod_birth <pod_uid>");
            eprintln!("Example: observe_pod_birth abc123def456");
            eprintln!("\nTo find a pod UID:");
            eprintln!("  kubectl get pod <pod-name> -o jsonpath='{{.metadata.uid}}'");
            std::process::exit(1);
        });
    
    info!("🔬 Starting kernel observation for pod {}", pod_uid);
    info!("This will capture the REAL cascade of kernel operations!");
    
    match PodLifecycleObserver::observe_pod_creation(&pod_uid).await {
        Ok(events) => {
            let certificate = PodLifecycleObserver::generate_birth_certificate(&events);
            println!("\n{}", certificate);
            
            // Summary stats for the talk
            println!("\n📊 TALK HIGHLIGHTS:");
            println!("-------------------");
            println!("✨ {} total kernel operations observed", events.operations.len());
            println!("⏱️  {} ms from cgroup creation to running", events.total_duration_ms);
            
            let namespace_count = [
                events.namespace_changes.pid_namespace_created,
                events.namespace_changes.net_namespace_created,
                events.namespace_changes.mnt_namespace_created,
                events.namespace_changes.uts_namespace_created,
                events.namespace_changes.ipc_namespace_created,
                events.namespace_changes.cgroup_namespace_created,
            ].iter().filter(|&&x| x).count();
            
            println!("🔒 {} namespace isolations created", namespace_count);
            
            let resource_controls = [
                events.cgroup_operations.cpu_limit_set,
                events.cgroup_operations.memory_limit_set,
                events.cgroup_operations.pids_limit_set,
                events.cgroup_operations.io_limit_set,
            ].iter().filter(|&&x| x).count();
            
            println!("📦 {} resource controls applied", resource_controls);
            
            println!("\n💡 This is what Kubernetes REALLY does in the kernel!");
            println!("   No mocking, no simulation - pure kernel truth! 🎯");
        }
        Err(e) => {
            error!("Failed to observe pod: {}", e);
            error!("Make sure:");
            error!("  1. You're running this on a Kubernetes node");
            error!("  2. The pod UID is correct");
            error!("  3. You have permission to read /proc and /sys");
        }
    }
    
    Ok(())
}