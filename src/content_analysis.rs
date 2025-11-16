/// Content type detection for smart algorithm selection
use crate::algorithms::image::{Image, Pixel};
pub(crate) use crate::content_type::ContentType;

#[derive(Debug, Clone)]
pub struct ContentAnalysis {
    pub content_type: ContentType,
    pub color_count: usize,
    pub edge_sharpness: f32,
    pub gradient_smoothness: f32,
    pub text_likelihood: f32,
    pub noise_level: f32,
}

impl ContentAnalysis {
    pub fn analyze(image: &Image) -> Self {
        let color_count = count_unique_colors(image);
        let edge_sharpness = calculate_edge_sharpness(image);
        let gradient_smoothness = calculate_gradient_smoothness(image);
        let text_likelihood = detect_text_regions(image);
        let noise_level = calculate_noise_level(image);

        let content_type = classify_content(
            color_count,
            edge_sharpness,
            gradient_smoothness,
            text_likelihood,
        );

        Self {
            content_type,
            color_count,
            edge_sharpness,
            gradient_smoothness,
            text_likelihood,
            noise_level,
        }
    }

    pub fn print_summary(&self) {
        println!("   Content Analysis:");
        println!("     Type:              {:?}", self.content_type);
        println!("     Unique colors:     {}", self.color_count);
        println!("     Edge sharpness:    {:.2}", self.edge_sharpness);
        println!("     Gradient smooth:   {:.2}", self.gradient_smoothness);
        println!("     Text likelihood:   {:.2}", self.text_likelihood);
        println!("     Noise level:       {:.2}", self.noise_level);
        println!("     Recommended algo:  {}", self.content_type.recommended_algorithm());
    }
}

/// Count unique colors in an image (sampled for performance)
fn count_unique_colors(image: &Image) -> usize {
    use std::collections::HashSet;

    let mut colors = HashSet::new();
    let sample_step = (image.width * image.height / 10000).max(1);

    for (i, pixel) in image.pixels.iter().enumerate() {
        if i % sample_step == 0 {
            let color = (pixel.r as u32) << 16 | (pixel.g as u32) << 8 | (pixel.b as u32);
            colors.insert(color);

            // Early exit if clearly not pixel art
            if colors.len() > 4096 {
                return colors.len();
            }
        }
    }

    colors.len()
}

/// Calculate average edge sharpness (0.0 = smooth, 1.0 = sharp)
fn calculate_edge_sharpness(image: &Image) -> f32 {
    let mut sharp_edges = 0;
    let mut total_edges = 0;

    for y in 1..(image.height - 1) {
        for x in 1..(image.width - 1) {
            let center = image.get_pixel(x, y).unwrap();
            let right = image.get_pixel(x + 1, y).unwrap();
            let bottom = image.get_pixel(x, y + 1).unwrap();

            let diff_h = pixel_diff(&center, &right);
            let diff_v = pixel_diff(&center, &bottom);

            if diff_h > 10.0 || diff_v > 10.0 {
                total_edges += 1;
                if diff_h > 50.0 || diff_v > 50.0 {
                    sharp_edges += 1;
                }
            }
        }
    }

    if total_edges == 0 {
        0.0
    } else {
        sharp_edges as f32 / total_edges as f32
    }
}

/// Calculate gradient smoothness (0.0 = noisy, 1.0 = smooth)
fn calculate_gradient_smoothness(image: &Image) -> f32 {
    let mut smooth_count = 0;
    let mut total_count = 0;

    for y in 2..(image.height - 2) {
        for x in 2..(image.width - 2) {
            let p1 = image.get_pixel(x, y).unwrap();
            let p2 = image.get_pixel(x + 1, y).unwrap();
            let p3 = image.get_pixel(x + 2, y).unwrap();

            let diff1 = pixel_diff(&p1, &p2);
            let diff2 = pixel_diff(&p2, &p3);

            total_count += 1;

            // Check if gradient is smooth (similar differences)
            if (diff1 - diff2).abs() < 10.0 {
                smooth_count += 1;
            }
        }
    }

    if total_count == 0 {
        0.0
    } else {
        smooth_count as f32 / total_count as f32
    }
}

/// Detect text regions (high contrast, regular patterns)
fn detect_text_regions(image: &Image) -> f32 {
    let mut high_contrast_regions = 0;
    let mut total_regions = 0;

    let block_size = 8;

    for y in (0..image.height).step_by(block_size) {
        for x in (0..image.width).step_by(block_size) {
            let mut min_brightness = 255u32;
            let mut max_brightness = 0u32;

            for dy in 0..block_size.min(image.height - y) {
                for dx in 0..block_size.min(image.width - x) {
                    if let Some(pixel) = image.get_pixel(x + dx, y + dy) {
                        let brightness = (pixel.r as u32 + pixel.g as u32 + pixel.b as u32) / 3;
                        min_brightness = min_brightness.min(brightness);
                        max_brightness = max_brightness.max(brightness);
                    }
                }
            }

            total_regions += 1;

            // Text typically has high contrast
            if max_brightness - min_brightness > 100 {
                high_contrast_regions += 1;
            }
        }
    }

    if total_regions == 0 {
        0.0
    } else {
        high_contrast_regions as f32 / total_regions as f32
    }
}

/// Calculate noise level
fn calculate_noise_level(image: &Image) -> f32 {
    let mut noise_sum = 0.0;
    let mut count = 0;

    for y in 1..(image.height - 1) {
        for x in 1..(image.width - 1) {
            let center = image.get_pixel(x, y).unwrap();

            // Get 4-connected neighbors
            let neighbors = [
                image.get_pixel(x - 1, y).unwrap(),
                image.get_pixel(x + 1, y).unwrap(),
                image.get_pixel(x, y - 1).unwrap(),
                image.get_pixel(x, y + 1).unwrap(),
            ];

            // Calculate average neighbor
            let avg_r = neighbors.iter().map(|p| p.r as f32).sum::<f32>() / 4.0;
            let avg_g = neighbors.iter().map(|p| p.g as f32).sum::<f32>() / 4.0;
            let avg_b = neighbors.iter().map(|p| p.b as f32).sum::<f32>() / 4.0;

            // Noise is deviation from local average
            let noise = ((center.r as f32 - avg_r).abs() +
                (center.g as f32 - avg_g).abs() +
                (center.b as f32 - avg_b).abs()) / 3.0;

            noise_sum += noise;
            count += 1;
        }
    }

    if count == 0 {
        0.0
    } else {
        (noise_sum / count as f32) / 255.0
    }
}

/// Classify content based on metrics
fn classify_content(
    color_count: usize,
    edge_sharpness: f32,
    gradient_smoothness: f32,
    text_likelihood: f32,
) -> ContentType {
    // Pixel art: few colors, sharp edges
    if color_count < 256 && edge_sharpness > 0.7 {
        return ContentType::PixelArt;
    }

    // Text: high contrast, sharp edges
    if text_likelihood > 0.5 && edge_sharpness > 0.6 {
        return ContentType::Text;
    }

    // Screenshot: mix of text and graphics
    if text_likelihood > 0.3 && text_likelihood < 0.6 {
        return ContentType::Screenshot;
    }

    // Photography: smooth gradients, many colors
    if gradient_smoothness > 0.6 && color_count > 4096 {
        return ContentType::Photography;
    }

    // Artwork: moderate complexity
    if color_count > 256 && color_count < 8192 {
        return ContentType::Artwork;
    }

    ContentType::Mixed
}

/// Calculate color difference between two pixels
fn pixel_diff(a: &Pixel, b: &Pixel) -> f32 {
    let dr = (a.r as f32 - b.r as f32).abs();
    let dg = (a.g as f32 - b.g as f32).abs();
    let db = (a.b as f32 - b.b as f32).abs();
    (dr + dg + db) / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_art_detection() {
        // Create a simple pixel art pattern
        let mut img = Image::new(16, 16);
        for y in 0..16 {
            for x in 0..16 {
                let pixel = if (x / 4 + y / 4) % 2 == 0 {
                    Pixel::new(255, 0, 0)
                } else {
                    Pixel::new(0, 0, 255)
                };
                img.set_pixel(x, y, pixel);
            }
        }

        let analysis = ContentAnalysis::analyze(&img);
        assert!(analysis.color_count < 10);
        assert!(analysis.edge_sharpness > 0.5);
    }

    #[test]
    fn test_gradient_detection() {
        // Create a smooth gradient
        let mut img = Image::new(100, 100);
        for y in 0..100 {
            for x in 0..100 {
                let val = ((x + y) * 255 / 200) as u8;
                img.set_pixel(x, y, Pixel::new(val, val, val));
            }
        }

        let analysis = ContentAnalysis::analyze(&img);
        assert!(analysis.gradient_smoothness > 0.5);
    }
}
