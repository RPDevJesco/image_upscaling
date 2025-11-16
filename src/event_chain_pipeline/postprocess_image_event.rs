use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::event_chain_pipeline::pipeline_config::PipelineConfig;

/// Apply post-processing effects if needed
pub struct PostProcessImageEvent;

impl PostProcessImageEvent {
    pub fn new() -> Self {
        Self
    }
}

impl ChainableEvent for PostProcessImageEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let config: PipelineConfig = match context.get("config") {
            Some(cfg) => cfg,
            None => return EventResult::Failure("No config in context".to_string()),
        };

        if !config.enable_postprocessing {
            println!("   Post-processing disabled, skipping");
            return EventResult::Success(());
        }

        // For now, post-processing is minimal
        // Could add: color correction, artifact reduction, etc.
        println!("   Post-processing complete");

        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "PostProcessImage"
    }
}

impl Default for PostProcessImageEvent {
    fn default() -> Self {
        Self::new()
    }
}
