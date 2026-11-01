// The `AsBindGroup` derive exceeds the default recursion limit (128) on newer Rust versions.
#![recursion_limit = "256"]

pub mod water;
pub use water::*;

#[cfg(feature = "image_utils")]
mod image_utils;
#[cfg(feature = "image_utils")]
pub use image_utils::*;

mod wave;
pub use wave::*;

mod param;
pub use param::WaterParam;
