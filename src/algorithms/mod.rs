// Algorithm implementations
pub mod instant;
pub mod fast;
pub mod medium;
pub mod slow;
pub mod image;
pub mod upscaler;
mod upscale_tier;

pub mod prelude {
    // Instant tier
    pub use crate::instant::{NearestNeighbor, Bilinear};

    // Fast tier
    pub use crate::fast::{Bicubic, Lanczos};
}
