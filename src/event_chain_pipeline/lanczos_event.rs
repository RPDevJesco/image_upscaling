use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::algorithms::image::{Image, Pixel};
use crate::event_chain_pipeline::upscale_config::UpscaleConfig;

/// Lanczos interpolation upscaling event
pub struct LanczosEvent {
    lobes: i32,
}

impl LanczosEvent {
    pub fn new() -> Self {
        Self { lobes: 3 }
    }

    pub fn fast() -> Self {
        Self { lobes: 2 }
    }

    pub fn high_quality() -> Self {
        Self { lobes: 4 }
    }

    fn lanczos_kernel(&self, t: f32) -> f32 {
        let t = t.abs();
        if t < f32::EPSILON {
            return 1.0;
        }
        if t >= self.lobes as f32 {
            return 0.0;
        }

        let pi_t = std::f32::consts::PI * t;
        let sinc_t = pi_t.sin() / pi_t;
        let sinc_ta = (pi_t / self.lobes as f32).sin() / (pi_t / self.lobes as f32);

        sinc_t * sinc_ta
    }

    fn sample_lanczos(&self, image: &Image, x: f32, y: f32) -> Pixel {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let mut pixels = Vec::new();

        let range = self.lobes;
        for dy in (-range + 1)..=range {
            for dx in (-range + 1)..=range {
                let px = image.get_pixel_clamped(x0 + dx, y0 + dy);
                let weight_x = self.lanczos_kernel(dx as f32 - fx);
                let weight_y = self.lanczos_kernel(dy as f32 - fy);
                let weight = weight_x * weight_y;

                pixels.push((px, weight));
            }
        }

        Pixel::weighted_average(&pixels)
    }
}

impl ChainableEvent for LanczosEvent {
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

                let pixel = self.sample_lanczos(&image, src_x, src_y);
                result.set_pixel(x, y, pixel);
            }
        }

        context.set("output_image", result);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        match self.lobes {
            2 => "Lanczos2",
            3 => "Lanczos3",
            4 => "Lanczos4",
            _ => "Lanczos",
        }
    }
}

impl Default for LanczosEvent {
    fn default() -> Self {
        Self::new()
    }
}