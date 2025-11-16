/// Configuration for the upscaling pipeline
#[derive(Clone)]
pub struct PipelineConfig {
    pub scale_factor: f32,
    pub force_algorithm: Option<String>,
    pub enable_preprocessing: bool,
    pub enable_postprocessing: bool,
}

impl PipelineConfig {
    pub fn new(scale_factor: f32) -> Self {
        Self {
            scale_factor,
            force_algorithm: None,
            enable_preprocessing: true,
            enable_postprocessing: true,
        }
    }

    pub fn with_algorithm(mut self, algorithm: String) -> Self {
        self.force_algorithm = Some(algorithm);
        self
    }

    pub fn with_preprocessing(mut self, enabled: bool) -> Self {
        self.enable_preprocessing = enabled;
        self
    }

    pub fn with_postprocessing(mut self, enabled: bool) -> Self {
        self.enable_postprocessing = enabled;
        self
    }
}
