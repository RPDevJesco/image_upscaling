use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::algorithms::image::Image;

/// Validate image dimensions and format
pub struct ValidateImageEvent {
    min_size: usize,
    max_size: usize,
}

impl ValidateImageEvent {
    pub fn new() -> Self {
        Self {
            min_size: 1,
            max_size: 16384,
        }
    }

    pub fn with_limits(min_size: usize, max_size: usize) -> Self {
        Self { min_size, max_size }
    }
}

impl ChainableEvent for ValidateImageEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let image: Image = match context.get("input_image") {
            Some(img) => img,
            None => return EventResult::Failure("No input image in context".to_string()),
        };

        if image.width < self.min_size || image.height < self.min_size {
            return EventResult::Failure(format!(
                "Image too small: {}x{} (minimum: {}x{})",
                image.width, image.height, self.min_size, self.min_size
            ));
        }

        if image.width > self.max_size || image.height > self.max_size {
            return EventResult::Failure(format!(
                "Image too large: {}x{} (maximum: {}x{})",
                image.width, image.height, self.max_size, self.max_size
            ));
        }

        println!("   Validation passed");
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "ValidateImage"
    }
}

impl Default for ValidateImageEvent {
    fn default() -> Self {
        Self::new()
    }
}
