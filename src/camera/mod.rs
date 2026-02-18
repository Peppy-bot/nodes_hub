//! Camera management and capture

pub mod capture;
pub mod device;
pub mod nokhwa_impl;

pub use capture::{run_camera_capture_loop, run_nokhwa_capture_loop};
pub use device::CameraDevice;
pub use nokhwa_impl::NokhwaCamera;

#[cfg(test)]
pub use device::mock::MockCamera;
