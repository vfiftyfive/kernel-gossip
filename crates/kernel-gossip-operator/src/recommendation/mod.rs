use kernel_gossip_types::KernelWhisper;

#[derive(Debug, Clone)]
pub struct Recommendation {
    pub insight: String,
    pub suggested_action: String,
    pub kernel_evidence: String,
    pub priority: String,
}

pub struct RecommendationEngine {}

impl RecommendationEngine {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn analyze_kernel_whisper(&self, kw: &KernelWhisper) -> Option<Recommendation> {
        let throttle_percentage = kw.spec.kernel_truth.throttled_percent;
        
        if throttle_percentage >= 80.0 {
            Some(Recommendation {
                insight: format!("Pod {} is experiencing high CPU throttling at {:.1}%", kw.spec.pod_name, throttle_percentage),
                suggested_action: "Consider increase CPU limits by 50% to prevent throttling".to_string(),
                kernel_evidence: format!("Kernel shows {throttle_percentage:.1}% throttled time in recent period"),
                priority: "high".to_string(),
            })
        } else if throttle_percentage >= 40.0 {
            Some(Recommendation {
                insight: format!("Pod {} is experiencing moderate CPU throttling at {:.1}%", kw.spec.pod_name, throttle_percentage),
                suggested_action: "monitor CPU usage patterns and consider optimization".to_string(),
                kernel_evidence: format!("Kernel shows {throttle_percentage:.1}% throttled time in recent period"),
                priority: "medium".to_string(),
            })
        } else {
            None
        }
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}