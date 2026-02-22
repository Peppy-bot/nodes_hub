//! Camera management and capture

pub mod capture;
pub mod controls;
pub mod device;
pub mod nokhwa_impl;

pub use capture::run_nokhwa_capture_loop;
pub use controls::{create_control_channel, ControlSender};
pub use device::CameraDevice;
pub use nokhwa_impl::NokhwaCamera;

#[cfg(test)]
pub use device::mock::MockCamera;
