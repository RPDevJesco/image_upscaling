use crate::algorithms::image::Image;
pub(crate) use crate::algorithms::upscale_tier::UpscaleTier;

/// Trait for all upscaling algorithms
pub trait Upscaler: Send + Sync {
    /// Upscale an image by the given factor
    fn upscale(&self, image: &Image, scale_factor: f32) -> Image;

    /// Get the name of this upscaler
    fn name(&self) -> &str;

    /// Get the complexity tier: Instant, Fast, Medium, or Slow
    fn tier(&self) -> UpscaleTier;
}
