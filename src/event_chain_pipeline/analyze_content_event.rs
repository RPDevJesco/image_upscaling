use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::algorithms::image::Image;
use crate::content_analysis::ContentAnalysis;

/// Analyze image content to determine optimal processing strategy
pub struct AnalyzeContentEvent;

impl AnalyzeContentEvent {
    pub fn new() -> Self {
        Self
    }
}

impl ChainableEvent for AnalyzeContentEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let image: Image = match context.get("input_image") {
            Some(img) => img,
            None => return EventResult::Failure("No input image in context".to_string()),
        };

        let analysis = ContentAnalysis::analyze(&image);

        println!("   Content Type: {:?}", analysis.content_type);
        println!("   Recommended Algorithm: {}", analysis.content_type.recommended_algorithm());

        context.set("content_analysis", analysis);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "AnalyzeContent"
    }
}

impl Default for AnalyzeContentEvent {
    fn default() -> Self {
        Self::new()
    }
}
