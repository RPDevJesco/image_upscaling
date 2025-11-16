use crate::algorithms::image::{Image, Pixel};
use crate::algorithms::upscaler::{Upscaler, UpscaleTier};

/// Edge-Directed Interpolation (EDI)
///
/// Analyzes local gradients to interpolate along edges rather than across them.
/// This preserves sharp edges while smoothing flat regions.
/// Time complexity: O(n log n) due to gradient analysis
/// Space complexity: O(n) for gradient maps
pub struct EdgeDirected;

impl EdgeDirected {
    /// Calculate gradient magnitude at a pixel
    fn gradient_magnitude(image: &Image, x: i32, y: i32) -> f32 {
        let _center = image.get_pixel_clamped(x, y);

        let left = image.get_pixel_clamped(x - 1, y);
        let right = image.get_pixel_clamped(x + 1, y);
        let top = image.get_pixel_clamped(x, y - 1);
        let bottom = image.get_pixel_clamped(x, y + 1);

        let dx = (right.r as f32 - left.r as f32).abs()
            + (right.g as f32 - left.g as f32).abs()
            + (right.b as f32 - left.b as f32).abs();

        let dy = (bottom.r as f32 - top.r as f32).abs()
            + (bottom.g as f32 - top.g as f32).abs()
            + (bottom.b as f32 - top.b as f32).abs();

        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate edge direction at a pixel (in radians)
    fn edge_direction(image: &Image, x: i32, y: i32) -> f32 {
        let left = image.get_pixel_clamped(x - 1, y);
        let right = image.get_pixel_clamped(x + 1, y);
        let top = image.get_pixel_clamped(x, y - 1);
        let bottom = image.get_pixel_clamped(x, y + 1);

        let dx = (right.r as f32 - left.r as f32)
            + (right.g as f32 - left.g as f32)
            + (right.b as f32 - left.b as f32);

        let dy = (bottom.r as f32 - top.r as f32)
            + (bottom.g as f32 - top.g as f32)
            + (bottom.b as f32 - top.b as f32);

        dy.atan2(dx)
    }

    /// Sample with edge-aware interpolation
    fn sample_edge_directed(image: &Image, x: f32, y: f32) -> Pixel {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;

        // Calculate edge direction at the source location
        let edge_angle = Self::edge_direction(image, x0, y0);
        let gradient = Self::gradient_magnitude(image, x0, y0);

        // If gradient is low (flat region), use bilinear
        if gradient < 10.0 {
            return Self::bilinear_sample(image, x, y);
        }

        // Otherwise, interpolate along the edge direction
        let _fx = x - x0 as f32;
        let _fy = y - y0 as f32;

        // Get pixels along the edge direction
        let cos_angle = edge_angle.cos();
        let sin_angle = edge_angle.sin();

        let mut pixels = Vec::new();

        // Sample along edge direction
        for i in -1_i32..=1 {
            let offset = i as f32 * 0.5;
            let sample_x = x + cos_angle * offset;
            let sample_y = y + sin_angle * offset;

            let px = image.get_pixel_clamped(sample_x.round() as i32, sample_y.round() as i32);
            let weight = 1.0 - (i.abs() as f32 * 0.3);
            pixels.push((px, weight));
        }

        Pixel::weighted_average(&pixels)
    }

    /// Fallback bilinear sampling
    fn bilinear_sample(image: &Image, x: f32, y: f32) -> Pixel {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let p00 = image.get_pixel_clamped(x0, y0);
        let p10 = image.get_pixel_clamped(x0 + 1, y0);
        let p01 = image.get_pixel_clamped(x0, y0 + 1);
        let p11 = image.get_pixel_clamped(x0 + 1, y0 + 1);

        let top = Pixel::lerp(p00, p10, fx);
        let bottom = Pixel::lerp(p01, p11, fx);
        Pixel::lerp(top, bottom, fy)
    }
}

impl Upscaler for EdgeDirected {
    fn upscale(&self, image: &Image, scale_factor: f32) -> Image {
        let new_width = (image.width as f32 * scale_factor).round() as usize;
        let new_height = (image.height as f32 * scale_factor).round() as usize;

        let mut result = Image::new(new_width, new_height);

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 + 0.5) / scale_factor - 0.5;
                let src_y = (y as f32 + 0.5) / scale_factor - 0.5;

                let pixel = Self::sample_edge_directed(image, src_x, src_y);
                result.set_pixel(x, y, pixel);
            }
        }

        result
    }

    fn name(&self) -> &str {
        "Edge-Directed Interpolation"
    }

    fn tier(&self) -> UpscaleTier {
        UpscaleTier::Medium
    }
}

/// xBR-like upscaling (simplified version)
///
/// Inspired by the xBR (scale-by-rules) family of algorithms.
/// Analyzes pixel patterns to detect edges and corners.
/// Excellent for pixel art and sharp-edged content.
/// Time complexity: O(n) with high constant factor
/// Space complexity: O(1) working memory
pub struct ScaleByRules;

impl ScaleByRules {
    /// Calculate color difference between two pixels
    fn color_diff(a: Pixel, b: Pixel) -> f32 {
        let dr = (a.r as f32 - b.r as f32).abs();
        let dg = (a.g as f32 - b.g as f32).abs();
        let db = (a.b as f32 - b.b as f32).abs();
        dr + dg + db
    }

    /// Upscale 2x using pattern matching
    fn upscale_2x(image: &Image) -> Image {
        let mut result = Image::new(image.width * 2, image.height * 2);

        for y in 0..image.height {
            for x in 0..image.width {
                let center = image.get_pixel(x, y).unwrap();

                // Get 3x3 neighborhood
                let neighbors = [
                    image.get_pixel_clamped(x as i32 - 1, y as i32 - 1), // top-left
                    image.get_pixel_clamped(x as i32, y as i32 - 1),     // top
                    image.get_pixel_clamped(x as i32 + 1, y as i32 - 1), // top-right
                    image.get_pixel_clamped(x as i32 - 1, y as i32),     // left
                    image.get_pixel_clamped(x as i32 + 1, y as i32),     // right
                    image.get_pixel_clamped(x as i32 - 1, y as i32 + 1), // bottom-left
                    image.get_pixel_clamped(x as i32, y as i32 + 1),     // bottom
                    image.get_pixel_clamped(x as i32 + 1, y as i32 + 1), // bottom-right
                ];

                // Output 2x2 block
                let out_x = x * 2;
                let out_y = y * 2;

                // Default: replicate center pixel
                let mut output = [center; 4];

                // Detect edges and blend accordingly
                let threshold = 30.0;

                // Check for horizontal edge
                if Self::color_diff(neighbors[3], neighbors[4]) > threshold {
                    // Horizontal edge detected
                    output[0] = Pixel::lerp(center, neighbors[3], 0.5);
                    output[1] = Pixel::lerp(center, neighbors[4], 0.5);
                }

                // Check for vertical edge
                if Self::color_diff(neighbors[1], neighbors[6]) > threshold {
                    // Vertical edge detected
                    output[0] = Pixel::lerp(center, neighbors[1], 0.5);
                    output[2] = Pixel::lerp(center, neighbors[6], 0.5);
                }

                // Check for diagonal edges
                if Self::color_diff(neighbors[0], neighbors[7]) > threshold {
                    output[0] = Pixel::lerp(center, neighbors[0], 0.3);
                }
                if Self::color_diff(neighbors[2], neighbors[5]) > threshold {
                    output[1] = Pixel::lerp(center, neighbors[2], 0.3);
                }

                result.set_pixel(out_x, out_y, output[0]);
                result.set_pixel(out_x + 1, out_y, output[1]);
                result.set_pixel(out_x, out_y + 1, output[2]);
                result.set_pixel(out_x + 1, out_y + 1, output[3]);
            }
        }

        result
    }
}

impl Upscaler for ScaleByRules {
    fn upscale(&self, image: &Image, scale_factor: f32) -> Image {
        // Only supports 2x for now
        if scale_factor == 2.0 {
            Self::upscale_2x(image)
        } else {
            // For other scales, do multiple 2x passes or fall back
            let target_width = (image.width as f32 * scale_factor).round() as usize;
            let target_height = (image.height as f32 * scale_factor).round() as usize;

            let mut current = image.clone();

            // Do 2x passes until we reach or exceed target
            while current.width < target_width || current.height < target_height {
                current = Self::upscale_2x(&current);
            }

            // If we overshot, downscale (simple for now)
            if current.width != target_width || current.height != target_height {
                // Use nearest neighbor to get exact size
                let mut result = Image::new(target_width, target_height);
                for y in 0..target_height {
                    for x in 0..target_width {
                        let src_x = (x * current.width) / target_width;
                        let src_y = (y * current.height) / target_height;
                        result.set_pixel(x, y, current.get_pixel(src_x, src_y).unwrap());
                    }
                }
                result
            } else {
                current
            }
        }
    }

    fn name(&self) -> &str {
        "Scale-by-Rules (xBR-like)"
    }

    fn tier(&self) -> UpscaleTier {
        UpscaleTier::Medium
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_edge_image() -> Image {
        let mut img = Image::new(8, 8);
        for y in 0..8 {
            for x in 0..8 {
                if x < 4 {
                    img.set_pixel(x, y, Pixel::new(0, 0, 0));
                } else {
                    img.set_pixel(x, y, Pixel::new(255, 255, 255));
                }
            }
        }
        img
    }

    #[test]
    fn test_edge_directed() {
        let img = create_edge_image();
        let upscaler = EdgeDirected;
        let result = upscaler.upscale(&img, 2.0);

        assert_eq!(result.width, 16);
        assert_eq!(result.height, 16);
    }

    #[test]
    fn test_scale_by_rules() {
        let img = create_edge_image();
        let upscaler = ScaleByRules;
        let result = upscaler.upscale(&img, 2.0);

        assert_eq!(result.width, 16);
        assert_eq!(result.height, 16);
    }
}
