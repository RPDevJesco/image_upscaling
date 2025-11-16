use crate::algorithms::image::{Image, Pixel};
use crate::algorithms::upscaler::{Upscaler, UpscaleTier};
use std::f32::consts::PI;

/// Bicubic interpolation upscaling
///
/// Uses cubic interpolation on a 4x4 pixel neighborhood.
/// Smoother than bilinear with minimal ringing artifacts.
/// Time complexity: O(n) where n is output pixels (16 samples per pixel)
/// Space complexity: O(1) working memory
pub struct Bicubic;

impl Bicubic {
    /// Cubic interpolation kernel (Catmull-Rom spline)
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

    /// Sample using bicubic interpolation
    fn sample_bicubic(image: &Image, x: f32, y: f32) -> Pixel {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let mut pixels = Vec::new();

        // Sample 4x4 neighborhood
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

impl Upscaler for Bicubic {
    fn upscale(&self, image: &Image, scale_factor: f32) -> Image {
        let new_width = (image.width as f32 * scale_factor).round() as usize;
        let new_height = (image.height as f32 * scale_factor).round() as usize;

        let mut result = Image::new(new_width, new_height);

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 + 0.5) / scale_factor - 0.5;
                let src_y = (y as f32 + 0.5) / scale_factor - 0.5;

                let pixel = Self::sample_bicubic(image, src_x, src_y);
                result.set_pixel(x, y, pixel);
            }
        }

        result
    }

    fn name(&self) -> &str {
        "Bicubic"
    }

    fn tier(&self) -> UpscaleTier {
        UpscaleTier::Fast
    }
}

/// Lanczos interpolation upscaling
///
/// Uses sinc-based Lanczos kernel for high-quality resampling.
/// Sharpest of the fast algorithms but may introduce slight ringing.
/// Time complexity: O(n) where n is output pixels (typically 36-64 samples per pixel)
/// Space complexity: O(1) working memory
pub struct Lanczos {
    /// Lanczos kernel size (a=2 or a=3 typically)
    lobes: i32,
}

impl Lanczos {
    /// Create Lanczos upscaler with 3 lobes (good quality/performance balance)
    pub fn new() -> Self {
        Self { lobes: 3 }
    }

    /// Create Lanczos upscaler with 2 lobes (faster, slightly less quality)
    pub fn fast() -> Self {
        Self { lobes: 2 }
    }

    /// Create Lanczos upscaler with 4 lobes (highest quality, slower)
    pub fn high_quality() -> Self {
        Self { lobes: 4 }
    }

    /// Lanczos kernel function
    fn lanczos_kernel(&self, t: f32) -> f32 {
        let t = t.abs();
        if t < f32::EPSILON {
            return 1.0;
        }
        if t >= self.lobes as f32 {
            return 0.0;
        }

        let pi_t = PI * t;
        let sinc_t = pi_t.sin() / pi_t;
        let sinc_ta = (pi_t / self.lobes as f32).sin() / (pi_t / self.lobes as f32);

        sinc_t * sinc_ta
    }

    /// Sample using Lanczos interpolation
    fn sample_lanczos(&self, image: &Image, x: f32, y: f32) -> Pixel {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let mut pixels = Vec::new();

        // Sample neighborhood based on lobe count
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

impl Upscaler for Lanczos {
    fn upscale(&self, image: &Image, scale_factor: f32) -> Image {
        let new_width = (image.width as f32 * scale_factor).round() as usize;
        let new_height = (image.height as f32 * scale_factor).round() as usize;

        let mut result = Image::new(new_width, new_height);

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 + 0.5) / scale_factor - 0.5;
                let src_y = (y as f32 + 0.5) / scale_factor - 0.5;

                let pixel = self.sample_lanczos(image, src_x, src_y);
                result.set_pixel(x, y, pixel);
            }
        }

        result
    }

    fn name(&self) -> &str {
        match self.lobes {
            2 => "Lanczos2",
            3 => "Lanczos3",
            4 => "Lanczos4",
            _ => "Lanczos",
        }
    }

    fn tier(&self) -> UpscaleTier {
        UpscaleTier::Fast
    }
}

impl Default for Lanczos {
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
                let val = ((x + y) * 30) as u8;
                img.set_pixel(x, y, Pixel::new(val, val, val));
            }
        }
        img
    }

    #[test]
    fn test_bicubic_upscale() {
        let img = create_test_image();
        let upscaler = Bicubic;
        let result = upscaler.upscale(&img, 2.0);

        assert_eq!(result.width, 8);
        assert_eq!(result.height, 8);
    }

    #[test]
    fn test_lanczos_upscale() {
        let img = create_test_image();
        let upscaler = Lanczos::new();
        let result = upscaler.upscale(&img, 2.0);

        assert_eq!(result.width, 8);
        assert_eq!(result.height, 8);
    }

    #[test]
    fn test_lanczos_variants() {
        let img = create_test_image();

        let fast = Lanczos::fast();
        let normal = Lanczos::new();
        let hq = Lanczos::high_quality();

        assert_eq!(fast.lobes, 2);
        assert_eq!(normal.lobes, 3);
        assert_eq!(hq.lobes, 4);

        let _ = fast.upscale(&img, 2.0);
        let _ = normal.upscale(&img, 2.0);
        let _ = hq.upscale(&img, 2.0);
    }
}
