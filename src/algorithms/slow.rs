use crate::algorithms::image::{Image, Pixel};
use crate::algorithms::upscaler::{Upscaler, UpscaleTier};

/// Iterative Back-Projection (IBP)
///
/// Iteratively refines the upscaled image by minimizing reconstruction error.
/// Simulates the downscaling process and adjusts the upscaled image to minimize
/// the difference between the simulated downscale and the original.
///
/// Time complexity: O(n² * iterations) due to iterative refinement
/// Space complexity: O(n) for temporary buffers
pub struct IterativeBackProjection {
    iterations: usize,
    learning_rate: f32,
}

impl IterativeBackProjection {
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
                // Average pixels in the source region
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

    /// Calculate error between two images
    fn calculate_error(a: &Pixel, b: &Pixel) -> (f32, f32, f32) {
        (
            a.r as f32 - b.r as f32,
            a.g as f32 - b.g as f32,
            a.b as f32 - b.b as f32,
        )
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

impl Upscaler for IterativeBackProjection {
    fn upscale(&self, image: &Image, scale_factor: f32) -> Image {
        let _new_width = (image.width as f32 * scale_factor).round() as usize;
        let _new_height = (image.height as f32 * scale_factor).round() as usize;

        // Start with bilinear upscale as initial estimate
        let mut result = crate::instant::Bilinear.upscale(image, scale_factor);

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

                    let (er, eg, eb) = Self::calculate_error(&original, &simulated);

                    error_image.set_pixel(
                        x,
                        y,
                        Pixel::new(
                            (er + 128.0).clamp(0.0, 255.0) as u8,
                            (eg + 128.0).clamp(0.0, 255.0) as u8,
                            (eb + 128.0).clamp(0.0, 255.0) as u8,
                        ),
                    );
                }
            }

            // Back-project error to high-resolution image
            Self::back_project(&mut result, &error_image, scale_factor, self.learning_rate);
        }

        result
    }

    fn name(&self) -> &str {
        "Iterative Back-Projection"
    }

    fn tier(&self) -> UpscaleTier {
        UpscaleTier::Slow
    }
}

impl Default for IterativeBackProjection {
    fn default() -> Self {
        Self::new()
    }
}

/// Total Variation (TV) Regularization Upscaling
///
/// Uses edge-preserving smoothing to reduce artifacts while maintaining sharpness.
/// Minimizes total variation (sum of gradient magnitudes) while fitting the data.
///
/// Time complexity: O(n * iterations)
/// Space complexity: O(n) for gradient buffers
pub struct TotalVariation {
    iterations: usize,
    lambda: f32, // Regularization strength
}

impl TotalVariation {
    pub fn new() -> Self {
        Self {
            iterations: 15,
            lambda: 0.1,
        }
    }
    
    /// Calculate total variation at a pixel
    fn calculate_tv_gradient(image: &Image, x: usize, y: usize) -> (f32, f32, f32) {
        let center = image.get_pixel(x, y).unwrap();

        let right = image.get_pixel_clamped(x as i32 + 1, y as i32);
        let bottom = image.get_pixel_clamped(x as i32, y as i32 + 1);

        let grad_x_r = right.r as f32 - center.r as f32;
        let grad_x_g = right.g as f32 - center.g as f32;
        let grad_x_b = right.b as f32 - center.b as f32;

        let grad_y_r = bottom.r as f32 - center.r as f32;
        let grad_y_g = bottom.g as f32 - center.g as f32;
        let grad_y_b = bottom.b as f32 - center.b as f32;

        let tv_r = (grad_x_r.powi(2) + grad_y_r.powi(2)).sqrt();
        let tv_g = (grad_x_g.powi(2) + grad_y_g.powi(2)).sqrt();
        let tv_b = (grad_x_b.powi(2) + grad_y_b.powi(2)).sqrt();

        (tv_r, tv_g, tv_b)
    }

    /// Apply one iteration of TV regularization
    fn tv_iteration(image: &mut Image, lambda: f32) {
        let mut updates = Vec::new();

        for y in 0..image.height {
            for x in 0..image.width {
                let (tv_r, tv_g, tv_b) = Self::calculate_tv_gradient(image, x, y);

                // Get neighboring pixels for smoothing
                let neighbors = [
                    image.get_pixel_clamped(x as i32 - 1, y as i32),
                    image.get_pixel_clamped(x as i32 + 1, y as i32),
                    image.get_pixel_clamped(x as i32, y as i32 - 1),
                    image.get_pixel_clamped(x as i32, y as i32 + 1),
                ];

                let center = image.get_pixel(x, y).unwrap();

                // Weighted average with TV-based weights
                let total_tv = tv_r + tv_g + tv_b + 1e-6;
                let weight = lambda / total_tv;

                let mut new_r = center.r as f32;
                let mut new_g = center.g as f32;
                let mut new_b = center.b as f32;

                for neighbor in &neighbors {
                    new_r += weight * (neighbor.r as f32 - center.r as f32);
                    new_g += weight * (neighbor.g as f32 - center.g as f32);
                    new_b += weight * (neighbor.b as f32 - center.b as f32);
                }

                updates.push((
                    x,
                    y,
                    Pixel::new(
                        new_r.clamp(0.0, 255.0) as u8,
                        new_g.clamp(0.0, 255.0) as u8,
                        new_b.clamp(0.0, 255.0) as u8,
                    ),
                ));
            }
        }

        // Apply updates
        for (x, y, pixel) in updates {
            image.set_pixel(x, y, pixel);
        }
    }
}

impl Upscaler for TotalVariation {
    fn upscale(&self, image: &Image, scale_factor: f32) -> Image {
        // Start with bicubic as initial estimate
        let mut result = crate::fast::Bicubic.upscale(image, scale_factor);

        // Apply TV regularization
        for _ in 0..self.iterations {
            Self::tv_iteration(&mut result, self.lambda);
        }

        result
    }

    fn name(&self) -> &str {
        "Total Variation Regularization"
    }

    fn tier(&self) -> UpscaleTier {
        UpscaleTier::Slow
    }
}

impl Default for TotalVariation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image() -> Image {
        let mut img = Image::new(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                let val = ((x + y) * 40) as u8;
                img.set_pixel(x, y, Pixel::new(val, val, val));
            }
        }
        img
    }

    #[test]
    fn test_ibp() {
        let img = create_test_image();
        let upscaler = IterativeBackProjection::fast();
        let result = upscaler.upscale(&img, 2.0);

        assert_eq!(result.width, 8);
        assert_eq!(result.height, 8);
    }

    #[test]
    fn test_tv() {
        let img = create_test_image();
        let upscaler = TotalVariation::new();
        let result = upscaler.upscale(&img, 2.0);

        assert_eq!(result.width, 8);
        assert_eq!(result.height, 8);
    }
}
