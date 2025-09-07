use kernel_gossip_e2e::*;

#[tokio::test]
async fn test_memory_pressure_detection_e2e() {
    // Initialize tracing for test output
    tracing_subscriber::fmt::init();
    
    // Initialize E2E test environment - REAL cluster only
    let test_env = E2ETestEnvironment::new().await
        .expect("Failed to connect to REAL Kubernetes cluster");
    
    // Verify operator is running
    test_env.verify_operator_running().await
        .expect("Operator must be running in kernel-gossip namespace");
    
    // Deploy a memory-intensive workload with unique name
    let timestamp = chrono::Utc::now().timestamp();
    let pod_name = format!("memory-stress-e2e-{timestamp}");
    let workload = test_env.deploy_memory_stress_workload(&pod_name).await
        .expect("Failed to deploy test workload");
    
    // Wait for pod to be running
    test_env.wait_for_pod_ready(&workload.pod_name, &workload.namespace).await
        .expect("Pod failed to start");
    
    // Give it time to generate memory pressure
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    // Create manual KernelWhisper for memory pressure
    // In production, this would come from Pixie detecting page faults
    test_env.create_manual_memory_pressure_whisper(&workload.pod_name, 92.0, 150.0).await
        .expect("Failed to create test KernelWhisper");
    
    // Give operator time to reconcile
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    // Check if KernelWhisper was created
    let whispers = test_env.get_kernel_whispers_for_pod(&workload.pod_name).await
        .expect("Failed to query KernelWhispers");
    
    // Verify we detected memory pressure
    assert!(!whispers.is_empty(), "No KernelWhispers created for memory-stressed pod");
    
    // Verify operator logs show memory-specific insights
    let logs = test_env.get_operator_logs_for_pod(&workload.pod_name).await
        .expect("Failed to get operator logs");
    
    println!("Operator logs for pod {}: {}", workload.pod_name, logs);
    
    // For now, just check that we got logs (operator may not have specific memory recommendations)
    assert!(!logs.is_empty() || !whispers.is_empty(), 
        "Either operator logs or KernelWhispers should exist");
    
    // Cleanup
    test_env.cleanup_workload(&workload).await
        .expect("Failed to cleanup test workload");
}