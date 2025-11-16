//! # Image Upscaler
//!
//! A comprehensive collection of image upscaling algorithms, from instant
//! nearest-neighbor to slow iterative optimization methods.
//!
//! ## Algorithm Tiers
//!
//! - **Instant**: Nearest neighbor, bilinear (O(n))
//! - **Fast**: Bicubic, Lanczos (O(n) with higher constants)
//! - **Medium**: Edge-directed, scale-by-rules (O(n log n))
//! - **Slow**: Iterative back-projection, TV regularization (O(nÂ²) or iterative)
//!
//! ## Quick Start
//!
//! ```ignore
//! use image_upscaler::prelude::*;
//!
//! let image = Image::new(100, 100);
//! let upscaler = Lanczos::new();
//! let result = upscaler.upscale(&image, 2.0);
//! ```

pub mod content_analysis;
pub mod algorithms;
mod content_type;
pub mod event_chain_pipeline;

/// Get an upscaler by name
pub fn get_upscaler(name: &str) -> Option<Box<dyn Upscaler>> {
    match name.to_lowercase().as_str() {
        "nearest" | "nearest_neighbor" => Some(Box::new(instant::NearestNeighbor)),
        "bilinear" => Some(Box::new(instant::Bilinear)),
        "bicubic" => Some(Box::new(fast::Bicubic)),
        "lanczos" | "lanczos3" => Some(Box::new(fast::Lanczos::new())),
        "lanczos2" => Some(Box::new(fast::Lanczos::fast())),
        "lanczos4" => Some(Box::new(fast::Lanczos::high_quality())),
        "edge_directed" | "edi" => Some(Box::new(medium::EdgeDirected)),
        "scale_by_rules" | "xbr" => Some(Box::new(medium::ScaleByRules)),
        "ibp" | "back_projection" => Some(Box::new(slow::IterativeBackProjection::new())),
        "tv" | "total_variation" => Some(Box::new(slow::TotalVariation::new())),
        _ => None,
    }
}

/// Get all available upscalers
pub fn all_upscalers() -> Vec<Box<dyn Upscaler>> {
    vec![
        // Instant
        Box::new(instant::NearestNeighbor),
        Box::new(instant::Bilinear),
        // Fast
        Box::new(fast::Bicubic),
        Box::new(fast::Lanczos::fast()),
        Box::new(fast::Lanczos::new()),
        Box::new(fast::Lanczos::high_quality()),
        // Medium
        Box::new(medium::EdgeDirected),
        Box::new(medium::ScaleByRules),
        // Slow
        Box::new(slow::IterativeBackProjection::fast()),
        Box::new(slow::IterativeBackProjection::new()),
        Box::new(slow::IterativeBackProjection::quality()),
        Box::new(slow::TotalVariation::new()),
    ]
}

/// Get upscalers for a specific tier
pub fn upscalers_by_tier(tier: UpscaleTier) -> Vec<Box<dyn Upscaler>> {
    all_upscalers()
        .into_iter()
        .filter(|u| u.tier() == tier)
        .collect()
}


use crate::algorithms::{fast, instant, medium, slow};
use crate::algorithms::upscaler::{UpscaleTier, Upscaler};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_upscaler() {
        assert!(get_upscaler("nearest").is_some());
        assert!(get_upscaler("bilinear").is_some());
        assert!(get_upscaler("bicubic").is_some());
        assert!(get_upscaler("lanczos").is_some());
        assert!(get_upscaler("invalid").is_none());
    }

    #[test]
    fn test_all_upscalers() {
        let upscalers = all_upscalers();
        assert!(upscalers.len() >= 10);
    }

    #[test]
    fn test_upscalers_by_tier() {
        let instant = upscalers_by_tier(UpscaleTier::Instant);
        let fast = upscalers_by_tier(UpscaleTier::Fast);
        let medium = upscalers_by_tier(UpscaleTier::Medium);
        let slow = upscalers_by_tier(UpscaleTier::Slow);

        assert!(!instant.is_empty());
        assert!(!fast.is_empty());
        assert!(!medium.is_empty());
        assert!(!slow.is_empty());
    }
}
