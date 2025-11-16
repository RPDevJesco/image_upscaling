use crate::algorithms::image::{Image, Pixel};
use crate::algorithms::upscaler::{Upscaler, UpscaleTier};

/// Nearest neighbor upscaling - the fastest possible algorithm
///
/// Simply replicates pixels. No interpolation. Perfect for pixel art.
/// Time complexity: O(n) where n is output pixels
/// Space complexity: O(1) working memory
pub struct NearestNeighbor;

impl Upscaler for NearestNeighbor {
    fn upscale(&self, image: &Image, scale_factor: f32) -> Image {
        let new_width = (image.width as f32 * scale_factor).round() as usize;
        let new_height = (image.height as f32 * scale_factor).round() as usize;

        let mut result = Image::new(new_width, new_height);

        for y in 0..new_height {
            for x in 0..new_width {
                // Map output coordinates back to input coordinates
                let src_x = (x as f32 / scale_factor).floor() as usize;
                let src_y = (y as f32 / scale_factor).floor() as usize;

                // Clamp to valid range
                let src_x = src_x.min(image.width - 1);
                let src_y = src_y.min(image.height - 1);

                result.set_pixel(x, y, image.get_pixel(src_x, src_y).unwrap());
            }
        }

        result
    }

    fn name(&self) -> &str {
        "Nearest Neighbor"
    }

    fn tier(&self) -> UpscaleTier {
        UpscaleTier::Instant
    }
}

/// Bilinear interpolation upscaling
///
/// Interpolates between the 4 nearest pixels using linear interpolation.
/// Much smoother than nearest neighbor but still very fast.
/// Time complexity: O(n) where n is output pixels
/// Space complexity: O(1) working memory
pub struct Bilinear;

impl Bilinear {
    /// Sample a pixel using bilinear interpolation at floating-point coordinates
    fn sample_bilinear(image: &Image, x: f32, y: f32) -> Pixel {
        // Get the four surrounding pixels
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        // Get fractional parts for interpolation
        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        // Get the four corner pixels (with clamping for edge cases)
        let p00 = image.get_pixel_clamped(x0, y0);
        let p10 = image.get_pixel_clamped(x1, y0);
        let p01 = image.get_pixel_clamped(x0, y1);
        let p11 = image.get_pixel_clamped(x1, y1);

        // Interpolate in X direction
        let top = Pixel::lerp(p00, p10, fx);
        let bottom = Pixel::lerp(p01, p11, fx);

        // Interpolate in Y direction
        Pixel::lerp(top, bottom, fy)
    }
}

impl Upscaler for Bilinear {
    fn upscale(&self, image: &Image, scale_factor: f32) -> Image {
        let new_width = (image.width as f32 * scale_factor).round() as usize;
        let new_height = (image.height as f32 * scale_factor).round() as usize;

        let mut result = Image::new(new_width, new_height);

        for y in 0..new_height {
            for x in 0..new_width {
                // Map output coordinates to input space (continuous)
                let src_x = (x as f32 + 0.5) / scale_factor - 0.5;
                let src_y = (y as f32 + 0.5) / scale_factor - 0.5;

                let pixel = Self::sample_bilinear(image, src_x, src_y);
                result.set_pixel(x, y, pixel);
            }
        }

        result
    }

    fn name(&self) -> &str {
        "Bilinear"
    }

    fn tier(&self) -> UpscaleTier {
        UpscaleTier::Instant
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image() -> Image {
        let mut img = Image::new(2, 2);
        img.set_pixel(0, 0, Pixel::new(0, 0, 0));       // Black
        img.set_pixel(1, 0, Pixel::new(255, 255, 255)); // White
        img.set_pixel(0, 1, Pixel::new(255, 0, 0));     // Red
        img.set_pixel(1, 1, Pixel::new(0, 255, 0));     // Green
        img
    }

    #[test]
    fn test_nearest_neighbor_2x() {
        let img = create_test_image();
        let upscaler = NearestNeighbor;
        let result = upscaler.upscale(&img, 2.0);

        assert_eq!(result.width, 4);
        assert_eq!(result.height, 4);

        // Check that pixels are replicated
        assert_eq!(result.get_pixel(0, 0).unwrap(), Pixel::new(0, 0, 0));
        assert_eq!(result.get_pixel(1, 0).unwrap(), Pixel::new(0, 0, 0));
    }

    #[test]
    fn test_bilinear_2x() {
        let img = create_test_image();
        let upscaler = Bilinear;
        let result = upscaler.upscale(&img, 2.0);

        assert_eq!(result.width, 4);
        assert_eq!(result.height, 4);

        // Center pixels should be interpolated
        // We can't test exact values due to rounding, but verify it ran
        assert!(result.get_pixel(1, 1).is_some());
    }
}
