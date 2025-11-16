use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::algorithms::image::{Image, Pixel};
use crate::event_chain_pipeline::upscale_config::UpscaleConfig;

/// Bilinear interpolation upscaling event
pub struct BilinearEvent;

impl BilinearEvent {
    fn sample_bilinear(image: &Image, x: f32, y: f32) -> Pixel {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let p00 = image.get_pixel_clamped(x0, y0);
        let p10 = image.get_pixel_clamped(x1, y0);
        let p01 = image.get_pixel_clamped(x0, y1);
        let p11 = image.get_pixel_clamped(x1, y1);

        let top = Pixel::lerp(p00, p10, fx);
        let bottom = Pixel::lerp(p01, p11, fx);
        Pixel::lerp(top, bottom, fy)
    }
}

impl ChainableEvent for BilinearEvent {
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
                let src_x = (x as f32 + 0.5) / config.scale_factor - 0.5;
                let src_y = (y as f32 + 0.5) / config.scale_factor - 0.5;

                let pixel = Self::sample_bilinear(&image, src_x, src_y);
                result.set_pixel(x, y, pixel);
            }
        }

        context.set("output_image", result);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "Bilinear"
    }
}
