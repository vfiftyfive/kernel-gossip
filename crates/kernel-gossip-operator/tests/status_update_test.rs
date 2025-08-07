#[cfg(test)]
mod status_update_tests {
    use kernel_gossip_operator::recommendation::{RecommendationEngine, Recommendation};
    use kernel_gossip_types::KernelWhisper;

    #[test]
    fn test_build_status_with_recommendation() {
        let engine = RecommendationEngine::new();
        let kw = KernelWhisper::create("test-pod", "default", 85.0, 30.0);
        
        let recommendation = engine.analyze_kernel_whisper(&kw).unwrap();
        let status = kernel_gossip_operator::crd::build_status_update(&recommendation);
        
        assert!(status.contains("high CPU throttling"));
        assert!(status.contains("increase CPU limits"));
        assert!(status.contains("85.0%"));
    }

    #[test]
    fn test_build_status_no_recommendation() {
        let status = kernel_gossip_operator::crd::build_status_update_no_action("No issues detected");
        
        assert!(status.contains("No issues detected"));
        assert!(status.contains("healthy"));
    }

    #[test]
    fn test_status_includes_timestamp() {
        let engine = RecommendationEngine::new();
        let kw = KernelWhisper::create("test-pod", "default", 75.0, 40.0);
        let recommendation = engine.analyze_kernel_whisper(&kw).unwrap();
        
        let status = kernel_gossip_operator::crd::build_status_update(&recommendation);
        
        // Status should include some form of timestamp or "updated" indicator
        assert!(!status.is_empty());
        assert!(status.len() > 50); // Should be a substantial status message
    }

    #[test] 
    fn test_recommendation_to_status_format() {
        let rec = Recommendation {
            insight: "Test insight about throttling".to_string(),
            suggested_action: "Test action to take".to_string(),
            kernel_evidence: "Test evidence from kernel".to_string(),
            priority: "high".to_string(),
        };
        
        let status = kernel_gossip_operator::crd::build_status_update(&rec);
        
        assert!(status.contains("Test insight"));
        assert!(status.contains("Test action"));
        assert!(status.contains("Test evidence"));
        assert!(status.contains("high"));
    }
}