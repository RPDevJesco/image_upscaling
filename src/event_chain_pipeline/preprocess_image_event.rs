use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::algorithms::image::Image;
use crate::event_chain_pipeline::pipeline_config::PipelineConfig;

/// Apply preprocessing if needed (denoise, sharpen, etc.)
pub struct PreprocessImageEvent;

impl PreprocessImageEvent {
    pub fn new() -> Self {
        Self
    }
}

impl ChainableEvent for PreprocessImageEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let config: PipelineConfig = match context.get("config") {
            Some(cfg) => cfg,
            None => return EventResult::Failure("No config in context".to_string()),
        };

        if !config.enable_preprocessing {
            println!("   Preprocessing disabled, skipping");
            return EventResult::Success(());
        }

        let needs_denoising = context.get::<bool>("needs_denoising").unwrap_or(false);
        let needs_sharpening = context.get::<bool>("needs_sharpening").unwrap_or(false);

        if !needs_denoising && !needs_sharpening {
            println!("   No preprocessing needed");
            return EventResult::Success(());
        }

        let mut image: Image = match context.get("input_image") {
            Some(img) => img,
            None => return EventResult::Failure("No input image in context".to_string()),
        };

        if needs_denoising {
            println!("   Applying noise reduction...");
            image = apply_simple_denoise(&image);
        }

        if needs_sharpening {
            println!("   Applying sharpening...");
            image = apply_simple_sharpen(&image);
        }

        context.set("input_image", image);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "PreprocessImage"
    }
}

impl Default for PreprocessImageEvent {
    fn default() -> Self {
        Self::new()
    }
}

// Simple denoise using averaging
fn apply_simple_denoise(image: &Image) -> Image {
    let mut result = image.clone();

    for y in 1..(image.height - 1) {
        for x in 1..(image.width - 1) {
            let mut r_sum = 0u32;
            let mut g_sum = 0u32;
            let mut b_sum = 0u32;
            let mut count = 0;

            for dy in -1..=1 {
                for dx in -1..=1 {
                    if let Some(pixel) = image.get_pixel((x as i32 + dx) as usize, (y as i32 + dy) as usize) {
                        r_sum += pixel.r as u32;
                        g_sum += pixel.g as u32;
                        b_sum += pixel.b as u32;
                        count += 1;
                    }
                }
            }

            if count > 0 {
                use crate::algorithms::image::Pixel;
                result.set_pixel(
                    x, y,
                    Pixel::new(
                        (r_sum / count) as u8,
                        (g_sum / count) as u8,
                        (b_sum / count) as u8,
                    )
                );
            }
        }
    }

    result
}

// Simple sharpening using unsharp mask
fn apply_simple_sharpen(image: &Image) -> Image {
    let mut result = image.clone();

    for y in 1..(image.height - 1) {
        for x in 1..(image.width - 1) {
            let center = image.get_pixel(x, y).unwrap();

            // Calculate laplacian
            let neighbors = [
                image.get_pixel(x - 1, y).unwrap(),
                image.get_pixel(x + 1, y).unwrap(),
                image.get_pixel(x, y - 1).unwrap(),
                image.get_pixel(x, y + 1).unwrap(),
            ];

            let avg_r = neighbors.iter().map(|p| p.r as i32).sum::<i32>() / 4;
            let avg_g = neighbors.iter().map(|p| p.g as i32).sum::<i32>() / 4;
            let avg_b = neighbors.iter().map(|p| p.b as i32).sum::<i32>() / 4;

            let sharp_r = ((center.r as i32 - avg_r) / 2 + center.r as i32).clamp(0, 255) as u8;
            let sharp_g = ((center.g as i32 - avg_g) / 2 + center.g as i32).clamp(0, 255) as u8;
            let sharp_b = ((center.b as i32 - avg_b) / 2 + center.b as i32).clamp(0, 255) as u8;

            use crate::algorithms::image::Pixel;
            result.set_pixel(x, y, Pixel::new(sharp_r, sharp_g, sharp_b));
        }
    }

    result
}
