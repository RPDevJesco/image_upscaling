#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    PixelArt,      // Low color count, sharp edges
    Photography,   // Natural gradients, high detail
    Text,          // High contrast, sharp edges
    Screenshot,    // Mix of text and graphics
    Artwork,       // Hand-drawn, painterly
    Mixed,         // Multiple types
}

impl ContentType {
    pub fn description(&self) -> &str {
        match self {
            ContentType::PixelArt => "Pixel art with sharp edges and limited colors",
            ContentType::Photography => "Natural photography with smooth gradients",
            ContentType::Text => "Text or line art with high contrast",
            ContentType::Screenshot => "Screenshot with mixed content",
            ContentType::Artwork => "Digital artwork or paintings",
            ContentType::Mixed => "Mixed content types",
        }
    }

    /// Get recommended algorithm for this content type
    pub fn recommended_algorithm(&self) -> &str {
        match self {
            ContentType::PixelArt => "nearest",
            ContentType::Photography => "lanczos3",
            ContentType::Text => "nearest",
            ContentType::Screenshot => "bicubic",
            ContentType::Artwork => "lanczos3",
            ContentType::Mixed => "bicubic",
        }
    }
}
