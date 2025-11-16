pub mod pipeline_config;
pub mod load_image_event;
pub mod validate_image_event;
pub mod analyze_content_event;
pub mod detect_quality_issues_event;
pub mod preprocess_image_event;
pub mod upscale_with_strategy_event;
pub mod postprocess_image_event;
pub mod save_image_event;
pub mod upscale_config;
pub mod nearest_neighbor_event;
pub mod bilinear_event;
pub mod bicubic_event;
pub mod lanczos_event;
pub mod iterative_back_projection_event;

pub mod prelude {
    pub use crate::event_chain_pipeline::pipeline_config;
    pub use crate::event_chain_pipeline::load_image_event;
    pub use crate::event_chain_pipeline::validate_image_event;
    pub use crate::event_chain_pipeline::analyze_content_event;
    pub use crate::event_chain_pipeline::detect_quality_issues_event;
    pub use crate::event_chain_pipeline::preprocess_image_event;
    pub use crate::event_chain_pipeline::upscale_with_strategy_event;
    pub use crate::event_chain_pipeline::postprocess_image_event;
    pub use crate::event_chain_pipeline::save_image_event;
    pub use crate::event_chain_pipeline::upscale_config;
    pub use crate::event_chain_pipeline::nearest_neighbor_event;
    pub use crate::event_chain_pipeline::bilinear_event;
    pub use crate::event_chain_pipeline::bicubic_event;
    pub use crate::event_chain_pipeline::lanczos_event;
    pub use crate::event_chain_pipeline::iterative_back_projection_event;
}