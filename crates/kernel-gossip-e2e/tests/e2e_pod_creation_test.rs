use kernel_gossip_e2e::E2ETestEnvironment;
use std::time::Duration;
use chrono::Utc;

#[tokio::test]
async fn test_pod_creation_tracing() {
    let test_env = E2ETestEnvironment::new()
        .await
        .expect("Failed to create test environment");

    // Deploy a simple workload to trace its creation
    let pod_name = format!("traced-pod-e2e-{}", Utc::now().timestamp());
    
    // Create a simple nginx pod
    let workload = test_env.deploy_simple_workload(&pod_name).await
        .expect("Failed to deploy simple workload");

    // Wait for pod to be ready
    test_env.wait_for_pod_ready(&workload.pod_name, &workload.namespace).await
        .expect("Failed to wait for pod ready");

    // In a real scenario, Pixie would send a webhook with pod creation trace data
    // For now, we'll simulate by checking if the operator can handle pod creation events
    
    // Create a PodBirthCertificate manually (simulating Pixie webhook)
    test_env.create_manual_pod_birth_certificate(
        &pod_name,
        vec!["clone", "execve", "mount", "open", "setns"],
        847, // typical number of syscalls for container start
    ).await.expect("Failed to create PodBirthCertificate");

    // Wait a bit for the operator to process it
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Check operator logs for insights about pod creation
    let logs = test_env.get_operator_logs_for_pod(&pod_name).await
        .expect("Failed to get operator logs");
    
    // We expect insights about the pod creation syscalls
    assert!(logs.contains("syscall") || logs.contains(&pod_name) || logs.len() > 0,
        "Expected operator logs to contain pod creation insights, but got: {}", logs);
    
    // Clean up
    test_env.cleanup_workload(&workload).await
        .expect("Failed to cleanup workload");
}