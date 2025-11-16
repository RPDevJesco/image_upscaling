use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::algorithms::image::{Image, Pixel};
use crate::event_chain_pipeline::upscale_config::UpscaleConfig;

/// Iterative Back-Projection upscaling event
pub struct IterativeBackProjectionEvent {
    iterations: usize,
    learning_rate: f32,
}

impl IterativeBackProjectionEvent {
    /// Create with default parameters (10 iterations, 0.5 learning rate)
    pub fn new() -> Self {
        Self {
            iterations: 10,
            learning_rate: 0.5,
        }
    }

    /// Fast preset (5 iterations)
    pub fn fast() -> Self {
        Self {
            iterations: 5,
            learning_rate: 0.5,
        }
    }

    /// Quality preset (20 iterations)
    pub fn quality() -> Self {
        Self {
            iterations: 20,
            learning_rate: 0.3,
        }
    }

    /// Simulate downsampling (simple averaging)
    fn simulate_downsample(image: &Image, target_width: usize, target_height: usize) -> Image {
        let mut result = Image::new(target_width, target_height);
        let scale_x = image.width as f32 / target_width as f32;
        let scale_y = image.height as f32 / target_height as f32;

        for y in 0..target_height {
            for x in 0..target_width {
                let src_x_start = (x as f32 * scale_x) as usize;
                let src_y_start = (y as f32 * scale_y) as usize;
                let src_x_end = ((x + 1) as f32 * scale_x).min(image.width as f32) as usize;
                let src_y_end = ((y + 1) as f32 * scale_y).min(image.height as f32) as usize;

                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;
                let mut count = 0;

                for sy in src_y_start..src_y_end {
                    for sx in src_x_start..src_x_end {
                        if let Some(px) = image.get_pixel(sx, sy) {
                            r_sum += px.r as f32;
                            g_sum += px.g as f32;
                            b_sum += px.b as f32;
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    let avg_pixel = Pixel::new(
                        (r_sum / count as f32) as u8,
                        (g_sum / count as f32) as u8,
                        (b_sum / count as f32) as u8,
                    );
                    result.set_pixel(x, y, avg_pixel);
                }
            }
        }

        result
    }

    /// Back-project error to high-resolution image
    fn back_project(
        high_res: &mut Image,
        low_res_error: &Image,
        scale_factor: f32,
        learning_rate: f32,
    ) {
        for y in 0..high_res.height {
            for x in 0..high_res.width {
                let src_x = (x as f32 / scale_factor) as usize;
                let src_y = (y as f32 / scale_factor) as usize;

                if let Some(error_pixel) = low_res_error.get_pixel(src_x, src_y) {
                    if let Some(current) = high_res.get_pixel(x, y) {
                        let new_r = (current.r as f32
                            + error_pixel.r as f32 * learning_rate)
                            .clamp(0.0, 255.0) as u8;
                        let new_g = (current.g as f32
                            + error_pixel.g as f32 * learning_rate)
                            .clamp(0.0, 255.0) as u8;
                        let new_b = (current.b as f32
                            + error_pixel.b as f32 * learning_rate)
                            .clamp(0.0, 255.0) as u8;

                        high_res.set_pixel(x, y, Pixel::new(new_r, new_g, new_b));
                    }
                }
            }
        }
    }
}

impl ChainableEvent for IterativeBackProjectionEvent {
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

        // Start with bilinear upscale as initial estimate
        let mut result = Image::new(new_width, new_height);

        // Initial bilinear upscale
        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 + 0.5) / config.scale_factor - 0.5;
                let src_y = (y as f32 + 0.5) / config.scale_factor - 0.5;

                let x0 = src_x.floor() as i32;
                let y0 = src_y.floor() as i32;
                let x1 = x0 + 1;
                let y1 = y0 + 1;

                let fx = src_x - x0 as f32;
                let fy = src_y - y0 as f32;

                let p00 = image.get_pixel_clamped(x0, y0);
                let p10 = image.get_pixel_clamped(x1, y0);
                let p01 = image.get_pixel_clamped(x0, y1);
                let p11 = image.get_pixel_clamped(x1, y1);

                let top = Pixel::lerp(p00, p10, fx);
                let bottom = Pixel::lerp(p01, p11, fx);
                let pixel = Pixel::lerp(top, bottom, fy);

                result.set_pixel(x, y, pixel);
            }
        }

        // Iterative refinement
        for _iter in 0..self.iterations {
            // Simulate downsampling the current high-res image
            let simulated_low = Self::simulate_downsample(&result, image.width, image.height);

            // Calculate error between simulated and original
            let mut error_image = Image::new(image.width, image.height);
            for y in 0..image.height {
                for x in 0..image.width {
                    let original = image.get_pixel(x, y).unwrap();
                    let simulated = simulated_low.get_pixel(x, y).unwrap();

                    let er = (original.r as f32 - simulated.r as f32) + 128.0;
                    let eg = (original.g as f32 - simulated.g as f32) + 128.0;
                    let eb = (original.b as f32 - simulated.b as f32) + 128.0;

                    error_image.set_pixel(
                        x,
                        y,
                        Pixel::new(
                            er.clamp(0.0, 255.0) as u8,
                            eg.clamp(0.0, 255.0) as u8,
                            eb.clamp(0.0, 255.0) as u8,
                        ),
                    );
                }
            }

            // Back-project error to high-resolution image
            Self::back_project(&mut result, &error_image, config.scale_factor, self.learning_rate);
        }

        context.set("output_image", result);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        match self.iterations {
            5 => "IBP-Fast",
            10 => "IBP-Standard",
            20 => "IBP-Quality",
            _ => "IBP",
        }
    }
}

impl Default for IterativeBackProjectionEvent {
    fn default() -> Self {
        Self::new()
    }
}
