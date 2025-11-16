/// Configuration for upscaling pipeline stored in context
#[derive(Clone)]
pub struct UpscaleConfig {
    pub scale_factor: f32,
}

impl UpscaleConfig {
    pub fn new(scale_factor: f32) -> Self {
        Self { scale_factor }
    }
}
