use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::algorithms::image::Image;
use crate::event_chain_pipeline::upscale_config::UpscaleConfig;

/// Nearest neighbor upscaling event
pub struct NearestNeighborEvent;

impl ChainableEvent for NearestNeighborEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let image: Image = match context.get("input_image") {
            Some(img) => img,
            None => return EventResult::Failure("No input image in context".to_string()),
        };

        let config: UpscaleConfig = match context.get("config") {
            Some(cfg) => cfg,
            None => return EventResult::Failure("No upscale config in context".to_string()),
        };

        let new_width = (image.width as f32 * config.scale_factor).round() as usize;
        let new_height = (image.height as f32 * config.scale_factor).round() as usize;

        let mut result = Image::new(new_width, new_height);

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 / config.scale_factor).floor() as usize;
                let src_y = (y as f32 / config.scale_factor).floor() as usize;

                let src_x = src_x.min(image.width - 1);
                let src_y = src_y.min(image.height - 1);

                result.set_pixel(x, y, image.get_pixel(src_x, src_y).unwrap());
            }
        }

        context.set("output_image", result);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "NearestNeighbor"
    }
}
