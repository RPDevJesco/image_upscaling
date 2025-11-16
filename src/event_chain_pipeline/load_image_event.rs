use std::path::PathBuf;
use event_chains::{ChainableEvent, EventContext, EventResult};
use crate::algorithms::image::Image;

/// Load image from file path stored in context
pub struct LoadImageEvent {
    path: PathBuf,
}

impl LoadImageEvent {
    pub fn from_path<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }
}

impl ChainableEvent for LoadImageEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        match Image::load(&self.path) {
            Ok(image) => {
                println!("   Loaded {}x{} image", image.width, image.height);
                context.set("input_image", image);
                EventResult::Success(())
            }
            Err(e) => EventResult::Failure(format!("Failed to load image: {}", e)),
        }
    }

    fn name(&self) -> &str {
        "LoadImage"
    }
}