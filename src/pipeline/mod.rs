//! Frame processing pipeline

pub mod converter;
pub mod processor;

pub use converter::{bgr_to_rgb, convert_frame, encode_jpeg, rgb_to_bgr};
pub use processor::process_frame;
