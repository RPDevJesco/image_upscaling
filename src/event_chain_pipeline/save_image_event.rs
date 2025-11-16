use std::path::PathBuf;
use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::algorithms::image::Image;

/// Save output image to file
pub struct SaveImageEvent {
    path: PathBuf,
}

impl SaveImageEvent {
    pub fn to_path<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }
}

impl ChainableEvent for SaveImageEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let image: Image = match context.get("output_image") {
            Some(img) => img,
            None => return EventResult::Failure("No output image in context".to_string()),
        };

        match image.save(&self.path) {
            Ok(_) => {
                println!("   Image saved successfully");
                EventResult::Success(())
            }
            Err(e) => EventResult::Failure(format!("Failed to save image: {}", e)),
        }
    }

    fn name(&self) -> &str {
        "SaveImage"
    }
}
