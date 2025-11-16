use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::algorithms::image::{Image, Pixel};
use crate::event_chain_pipeline::upscale_config::UpscaleConfig;

/// Bicubic interpolation upscaling event
pub struct BicubicEvent;

impl BicubicEvent {
    fn cubic_kernel(t: f32) -> f32 {
        let t = t.abs();
        if t < 1.0 {
            1.5 * t * t * t - 2.5 * t * t + 1.0
        } else if t < 2.0 {
            -0.5 * t * t * t + 2.5 * t * t - 4.0 * t + 2.0
        } else {
            0.0
        }
    }

    fn sample_bicubic(image: &Image, x: f32, y: f32) -> Pixel {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let mut pixels = Vec::new();

        for dy in -1..=2 {
            for dx in -1..=2 {
                let px = image.get_pixel_clamped(x0 + dx, y0 + dy);
                let weight_x = Self::cubic_kernel(dx as f32 - fx);
                let weight_y = Self::cubic_kernel(dy as f32 - fy);
                let weight = weight_x * weight_y;

                pixels.push((px, weight));
            }
        }

        Pixel::weighted_average(&pixels)
    }
}

impl ChainableEvent for BicubicEvent {
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

                let pixel = Self::sample_bicubic(&image, src_x, src_y);
                result.set_pixel(x, y, pixel);
            }
        }

        context.set("output_image", result);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "Bicubic"
    }
}