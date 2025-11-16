use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::algorithms::fast::{Bicubic, Lanczos};
use crate::algorithms::image::Image;
use crate::algorithms::instant::{Bilinear, NearestNeighbor};
use crate::algorithms::slow::IterativeBackProjection;
use crate::content_analysis::ContentAnalysis;
use crate::event_chain_pipeline::pipeline_config::PipelineConfig;

/// Select and apply the optimal upscaling algorithm
pub struct UpscaleWithStrategyEvent;

impl UpscaleWithStrategyEvent {
    pub fn new() -> Self {
        Self
    }
}

impl ChainableEvent for UpscaleWithStrategyEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let image: Image = match context.get("input_image") {
            Some(img) => img,
            None => return EventResult::Failure("No input image in context".to_string()),
        };

        let config: PipelineConfig = match context.get("config") {
            Some(cfg) => cfg,
            None => return EventResult::Failure("No config in context".to_string()),
        };

        // Get content analysis for recommendation
        let analysis: ContentAnalysis = match context.get("content_analysis") {
            Some(a) => a,
            None => return EventResult::Failure("No content analysis in context".to_string()),
        };

        let recommended = analysis.content_type.recommended_algorithm();

        // Determine which algorithm to use
        let algorithm_name = if let Some(ref forced) = config.force_algorithm {
            // User specified algorithm - respect their choice
            if forced != recommended {
                println!("   Using {} (user choice, recommended: {})", forced, recommended);
            } else {
                println!("   Using {} (user choice, matches recommendation)", forced);
            }
            forced.clone()
        } else {
            // Auto-select based on analysis
            println!("   Auto-selected: {} (based on {:?})", recommended, analysis.content_type);
            recommended.to_string()
        };

        // Get the upscaler
        let upscaler: Box<dyn crate::algorithms::upscaler::Upscaler> = match algorithm_name.as_str() {
            "nearest" => Box::new(NearestNeighbor),
            "bilinear" => Box::new(Bilinear),
            "bicubic" => Box::new(Bicubic),
            "lanczos2" => Box::new(Lanczos::fast()),
            "lanczos3" => Box::new(Lanczos::new()),
            "lanczos4" => Box::new(Lanczos::high_quality()),
            "ibp-fast" => Box::new(IterativeBackProjection::fast()),
            "ibp" | "ibp-standard" => Box::new(IterativeBackProjection::new()),
            "ibp-quality" => Box::new(IterativeBackProjection::quality()),
            _ => return EventResult::Failure(format!("Unknown algorithm: {}", algorithm_name)),
        };

        println!("   Upscaling with {} ({}x)...", upscaler.name(), config.scale_factor);
        let result = upscaler.upscale(&image, config.scale_factor);

        println!("   Output size: {}x{}", result.width, result.height);
        context.set("output_image", result);
        context.set("algorithm_used", algorithm_name);

        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "UpscaleWithStrategy"
    }
}

impl Default for UpscaleWithStrategyEvent {
    fn default() -> Self {
        Self::new()
    }
}
