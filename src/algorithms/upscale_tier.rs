#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpscaleTier {
    /// O(n) - Nearest neighbor
    Instant,
    /// O(n) - Bilinear, bicubic, Lanczos
    Fast,
    /// O(n log n) - Edge-directed, fractal methods
    Medium,
    /// O(n²) or iterative - Back-projection, sparse coding, TV regularization
    Slow,
}

impl UpscaleTier {
    pub fn description(&self) -> &str {
        match self {
            UpscaleTier::Instant => "Instant (nearest neighbor)",
            UpscaleTier::Fast => "Fast (bilinear, bicubic, Lanczos)",
            UpscaleTier::Medium => "Medium (edge-directed, fractal)",
            UpscaleTier::Slow => "Slow (iterative optimization)",
        }
    }
}
