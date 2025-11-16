use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::content_analysis::{ContentAnalysis, ContentType};

/// Detect quality issues that may need preprocessing
pub struct DetectQualityIssuesEvent;

impl DetectQualityIssuesEvent {
    pub fn new() -> Self {
        Self
    }
}

impl ChainableEvent for DetectQualityIssuesEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let analysis: ContentAnalysis = match context.get("content_analysis") {
            Some(a) => a,
            None => return EventResult::Failure("No content analysis in context".to_string()),
        };

        let mut issues = Vec::new();

        if analysis.noise_level > 0.15 {
            issues.push("High noise level detected");
            context.set("needs_denoising", true);
        }

        if analysis.edge_sharpness < 0.3 && analysis.content_type != ContentType::Photography {
            issues.push("Low edge sharpness detected");
            context.set("needs_sharpening", true);
        }

        if !issues.is_empty() {
            println!("   Quality Issues:");
            for issue in &issues {
                println!("     - {}", issue);
            }
        } else {
            println!("   No quality issues detected");
        }

        context.set("quality_issues", issues);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "DetectQualityIssues"
    }
}

impl Default for DetectQualityIssuesEvent {
    fn default() -> Self {
        Self::new()
    }
}