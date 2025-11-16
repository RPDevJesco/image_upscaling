/// Simple RGB pixel representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Pixel {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn black() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }

    pub fn white() -> Self {
        Self { r: 255, g: 255, b: 255 }
    }

    /// Linear interpolation between two pixels
    pub fn lerp(a: Pixel, b: Pixel, t: f32) -> Pixel {
        let t = t.clamp(0.0, 1.0);
        Pixel {
            r: (a.r as f32 + (b.r as f32 - a.r as f32) * t) as u8,
            g: (a.g as f32 + (b.g as f32 - a.g as f32) * t) as u8,
            b: (a.b as f32 + (b.b as f32 - a.b as f32) * t) as u8,
        }
    }

    /// Weighted average of multiple pixels
    pub fn weighted_average(pixels: &[(Pixel, f32)]) -> Pixel {
        let mut r_sum = 0.0;
        let mut g_sum = 0.0;
        let mut b_sum = 0.0;
        let mut weight_sum = 0.0;

        for (pixel, weight) in pixels {
            r_sum += pixel.r as f32 * weight;
            g_sum += pixel.g as f32 * weight;
            b_sum += pixel.b as f32 * weight;
            weight_sum += weight;
        }

        if weight_sum == 0.0 {
            return Pixel::black();
        }

        Pixel {
            r: (r_sum / weight_sum).clamp(0.0, 255.0) as u8,
            g: (g_sum / weight_sum).clamp(0.0, 255.0) as u8,
            b: (b_sum / weight_sum).clamp(0.0, 255.0) as u8,
        }
    }
}

/// Simple image representation compatible with event_chains
#[derive(Debug, Clone)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Pixel>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Pixel::black(); width * height],
        }
    }

    pub fn from_pixels(width: usize, height: usize, pixels: Vec<Pixel>) -> Option<Self> {
        if pixels.len() != width * height {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels,
        })
    }

    /// Load an image from a file
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        use image::GenericImageView;

        let img = image::open(path)
            .map_err(|e| format!("Failed to open image: {}", e))?;

        let (width, height) = img.dimensions();
        let mut pixels = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                pixels.push(Pixel::new(pixel[0], pixel[1], pixel[2]));
            }
        }

        Ok(Self {
            width: width as usize,
            height: height as usize,
            pixels,
        })
    }

    /// Save an image to a file
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        use image::{ImageBuffer, Rgb};

        let mut img_buffer = ImageBuffer::new(self.width as u32, self.height as u32);

        for (x, y, pixel) in img_buffer.enumerate_pixels_mut() {
            let our_pixel = self.get_pixel(x as usize, y as usize).unwrap();
            *pixel = Rgb([our_pixel.r, our_pixel.g, our_pixel.b]);
        }

        img_buffer.save(path)
            .map_err(|e| format!("Failed to save image: {}", e))
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Pixel> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.pixels[y * self.width + x])
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = pixel;
        }
    }

    /// Get pixel with clamped coordinates (safe for out-of-bounds access)
    pub fn get_pixel_clamped(&self, x: i32, y: i32) -> Pixel {
        let x = x.clamp(0, self.width as i32 - 1) as usize;
        let y = y.clamp(0, self.height as i32 - 1) as usize;
        self.pixels[y * self.width + x]
    }

    /// Sample pixel at floating-point coordinates using nearest neighbor
    pub fn sample_nearest(&self, x: f32, y: f32) -> Pixel {
        let x = x.round().clamp(0.0, self.width as f32 - 1.0) as usize;
        let y = y.round().clamp(0.0, self.height as f32 - 1.0) as usize;
        self.pixels[y * self.width + x]
    }
}
