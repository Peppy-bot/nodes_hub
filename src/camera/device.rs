use crate::types::{CameraConfig, Frame, Result};
use super::controls::{CameraControlRequest, ControlResult};

/// Abstraction for camera devices
/// 
/// This trait enables testing by allowing mock implementations
pub trait CameraDevice: Send {
    /// Open and configure the camera
    fn open(&mut self, config: &CameraConfig) -> Result<()>;
    
    /// Capture a single frame (always RGB8 from camera)
    fn capture_frame(&mut self) -> Result<Frame>;
    
    /// Check if the camera is open
    fn is_open(&self) -> bool;

    /// Apply a camera control request.
    ///
    /// Returns a `ControlResult` indicating success/failure and the current
    /// value after applying the control.  The default implementation reports
    /// that controls are unsupported, which is appropriate for mock devices
    /// that don't need hardware control.
    fn apply_control(&mut self, _request: &CameraControlRequest) -> ControlResult {
        ControlResult::err("Camera controls not supported for this device")
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use crate::camera::controls::{CameraControlRequest, ControlResult};
    use crate::types::Error;
    
    /// Mock camera for testing
    pub struct MockCamera {
        is_open: bool,
        frame_counter: u32,
        width: u32,
        height: u32,
        /// Actual camera encoding (set during open, used when creating frames)
        camera_encoding: Option<crate::types::Encoding>,
        /// Simulated control values (brightness, contrast, gain, exposure, white_balance)
        pub brightness: i32,
        pub contrast: i32,
        pub gain: i32,
        pub exposure: i32,
        pub white_balance_temperature: i32,
    }
    
    impl MockCamera {
        pub fn new() -> Self {
            Self {
                is_open: false,
                frame_counter: 0,
                width: 640,
                height: 480,
                camera_encoding: None,
                brightness: 128,
                contrast: 128,
                gain: 50,
                exposure: 100,
                white_balance_temperature: 4500,
            }
        }
        
        /// Generate a test pattern (RGB gradient)
        fn generate_test_frame(&self) -> Vec<u8> {
            let width = self.width as usize;
            let height = self.height as usize;
            let mut data = Vec::with_capacity(width * height * 3);
            
            for y in 0..height {
                for x in 0..width {
                    let r = ((x * 255) / width) as u8;
                    let g = ((y * 255) / height) as u8;
                    let b = 128;
                    data.extend_from_slice(&[r, g, b]);
                }
            }
            
            data
        }
    }
    
    impl Default for MockCamera {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl CameraDevice for MockCamera {
        fn open(&mut self, config: &CameraConfig) -> Result<()> {
            self.width = config.resolution.width();
            self.height = config.resolution.height();
            self.camera_encoding = Some(config.camera_encoding);
            self.is_open = true;
            print!("[uvc_camera] Mock camera opened with resolution {}x{} at {} fps, camera_encoding: {}, topic_encoding: {}. ",
                self.width, self.height, config.frame_rate.as_u16(), config.camera_encoding, config.topic_encoding);
            Ok(())
        }
        
        fn capture_frame(&mut self) -> Result<Frame> {
            if !self.is_open {
                return Err(Error::Camera("Camera not open".to_string()));
            }

            let encoding = self.camera_encoding
                .ok_or_else(|| Error::Camera("Camera not open".to_string()))?;

            self.frame_counter += 1;

            Ok(Frame::from_capture(
                self.generate_test_frame(),
                self.width,
                self.height,
                std::time::Instant::now(),
                encoding,
            ))
        }
        
        fn is_open(&self) -> bool {
            self.is_open
        }

        fn apply_control(&mut self, request: &CameraControlRequest) -> ControlResult {
            use crate::camera::controls::ExposureMode;
            use crate::camera::controls::WhiteBalanceMode;

            match request {
                CameraControlRequest::SetBrightness { value } => {
                    self.brightness = *value;
                    ControlResult::ok(format!("Brightness set to {}", value), *value)
                }
                CameraControlRequest::SetContrast { value } => {
                    self.contrast = *value;
                    ControlResult::ok(format!("Contrast set to {}", value), *value)
                }
                CameraControlRequest::SetGain { value } => {
                    self.gain = *value;
                    ControlResult::ok(format!("Gain set to {}", value), *value)
                }
                CameraControlRequest::SetExposure { mode, value } => match mode {
                    ExposureMode::Auto => ControlResult::ok("Exposure set to auto mode", -1),
                    ExposureMode::Manual => {
                        self.exposure = *value;
                        ControlResult::ok(format!("Exposure set to manual, value {}", value), *value)
                    }
                },
                CameraControlRequest::SetWhiteBalance { mode, temperature } => match mode {
                    WhiteBalanceMode::Auto => {
                        ControlResult::ok("White balance set to auto mode", -1)
                    }
                    WhiteBalanceMode::Manual => {
                        self.white_balance_temperature = *temperature;
                        ControlResult::ok(
                            format!("White balance set to manual, temperature {}K", temperature),
                            *temperature,
                        )
                    }
                },
            }
        }
    }
}
