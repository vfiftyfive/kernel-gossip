#[cfg(test)]
mod recommendation_engine_tests {
    use kernel_gossip_operator::recommendation::{RecommendationEngine, Recommendation};
    use kernel_gossip_types::KernelWhisper;

    #[test]
    fn test_cpu_throttle_high_recommendation() {
        let engine = RecommendationEngine::new();
        
        // Create KernelWhisper with high CPU throttling (85%)
        let kw = KernelWhisper::create("throttled-pod", "default", 85.0, 30.0);
        
        let recommendation = engine.analyze_kernel_whisper(&kw);
        
        assert!(recommendation.is_some());
        let rec = recommendation.unwrap();
        assert!(rec.insight.contains("high CPU throttling"));
        assert!(rec.suggested_action.contains("increase CPU limits"));
        assert_eq!(rec.priority, "high");
    }

    #[test]
    fn test_cpu_throttle_moderate_recommendation() {
        let engine = RecommendationEngine::new();
        
        // Create KernelWhisper with moderate CPU throttling (45%)
        let kw = KernelWhisper::create("moderate-pod", "default", 45.0, 60.0);
        
        let recommendation = engine.analyze_kernel_whisper(&kw);
        
        assert!(recommendation.is_some());
        let rec = recommendation.unwrap();
        assert!(rec.insight.contains("moderate CPU throttling"));
        assert!(rec.suggested_action.contains("monitor"));
        assert_eq!(rec.priority, "medium");
    }

    #[test]
    fn test_cpu_throttle_low_no_recommendation() {
        let engine = RecommendationEngine::new();
        
        // Create KernelWhisper with low CPU throttling (5%)
        let kw = KernelWhisper::create("healthy-pod", "default", 5.0, 80.0);
        
        let recommendation = engine.analyze_kernel_whisper(&kw);
        
        assert!(recommendation.is_none());
    }

    #[test]
    fn test_recommendation_includes_kernel_evidence() {
        let engine = RecommendationEngine::new();
        
        let kw = KernelWhisper::create("evidence-pod", "default", 75.0, 40.0);
        
        let recommendation = engine.analyze_kernel_whisper(&kw).unwrap();
        
        assert!(recommendation.kernel_evidence.contains("75.0%"));
        assert!(recommendation.kernel_evidence.contains("throttled"));
    }

    #[test]
    fn test_recommendation_struct_creation() {
        let rec = Recommendation {
            insight: "Test insight".to_string(),
            suggested_action: "Test action".to_string(),
            kernel_evidence: "Test evidence".to_string(),
            priority: "high".to_string(),
        };
        
        assert_eq!(rec.insight, "Test insight");
        assert_eq!(rec.suggested_action, "Test action");
        assert_eq!(rec.kernel_evidence, "Test evidence");
        assert_eq!(rec.priority, "high");
    }
}