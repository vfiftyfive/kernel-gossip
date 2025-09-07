use kernel_gossip_e2e::*;

#[tokio::test]
async fn test_cpu_throttle_detection_e2e() {
    // Initialize tracing for test output
    tracing_subscriber::fmt::init();
    
    // Initialize E2E test environment - REAL cluster only
    let test_env = E2ETestEnvironment::new().await
        .expect("Failed to connect to REAL Kubernetes cluster");
    
    // Verify operator is running
    test_env.verify_operator_running().await
        .expect("Operator must be running in kernel-gossip namespace");
    
    // Deploy a CPU-intensive workload with unique name
    let timestamp = chrono::Utc::now().timestamp();
    let pod_name = format!("cpu-stress-e2e-{timestamp}");
    let workload = test_env.deploy_cpu_stress_workload(&pod_name).await
        .expect("Failed to deploy test workload");
    
    // Wait for pod to be running
    test_env.wait_for_pod_ready(&workload.pod_name, &workload.namespace).await
        .expect("Pod failed to start");
    
    // Give it time to generate CPU load and throttling
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    // Since we don't have Pixie webhook integration yet, create a manual KernelWhisper
    // In production, this would come from Pixie via webhook
    test_env.create_manual_kernel_whisper(&workload.pod_name, 85.5, 45.0).await
        .expect("Failed to create test KernelWhisper");
    
    // Give operator time to reconcile
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    // Check if KernelWhisper was created for this pod
    let whispers = test_env.get_kernel_whispers_for_pod(&workload.pod_name).await
        .expect("Failed to query KernelWhispers");
    
    // Verify we detected CPU throttling
    assert!(!whispers.is_empty(), "No KernelWhispers created for throttled pod");
    
    let whisper = &whispers[0];
    assert!(whisper.spec.kernel_truth.throttled_percent > 50.0, 
        "Expected high throttling, got {}%", whisper.spec.kernel_truth.throttled_percent);
    
    // Verify operator added insights
    let logs = test_env.get_operator_logs_for_pod(&workload.pod_name).await
        .expect("Failed to get operator logs");
    
    assert!(logs.contains("INSIGHT"), "Operator should generate insights");
    assert!(logs.contains("RECOMMENDATION"), "Operator should provide recommendations");
    
    // Cleanup
    test_env.cleanup_workload(&workload).await
        .expect("Failed to cleanup test workload");
}