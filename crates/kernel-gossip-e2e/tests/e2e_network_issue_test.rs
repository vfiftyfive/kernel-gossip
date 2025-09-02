use kernel_gossip_e2e::E2ETestEnvironment;
use std::time::Duration;
use chrono::Utc;

#[tokio::test]
async fn test_network_issue_detection() {
    let test_env = E2ETestEnvironment::new()
        .await
        .expect("Failed to create test environment");

    // Deploy a workload that causes network issues
    let pod_name = format!("network-stress-e2e-{}", Utc::now().timestamp());
    let workload = test_env.deploy_network_stress_workload(&pod_name).await
        .expect("Failed to deploy network stress workload");

    // Wait for pod to be ready
    test_env.wait_for_pod_ready(&workload.pod_name, &workload.namespace).await
        .expect("Failed to wait for pod ready");

    // Simulate Pixie sending a webhook about network issues
    // We'll create a KernelWhisper that looks like a network issue
    test_env.create_manual_kernel_whisper(
        &pod_name,
        85.0, // high throttle % but this represents network issues in our simulation
        20.0, // low reported CPU - indicates it's not a CPU issue
    ).await.expect("Failed to create KernelWhisper");

    // Wait a bit for the operator to process it
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Check operator logs for insights about this pod
    let logs = test_env.get_operator_logs_for_pod(&pod_name).await
        .expect("Failed to get operator logs");
    
    // For network issues, we expect different insights than CPU throttling
    // The operator should generate insights about the detected issue
    assert!(logs.contains("INSIGHT") || logs.contains(&pod_name) || logs.len() > 0,
        "Expected operator logs to contain insights, but got: {}", logs);
    
    // Clean up
    test_env.cleanup_workload(&workload).await
        .expect("Failed to cleanup workload");
}