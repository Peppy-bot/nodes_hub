use crate::types::{CameraConfig, Frame, Result};

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
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use crate::types::Error;
    
    /// Mock camera for testing
    pub struct MockCamera {
        is_open: bool,
        frame_counter: u32,
        width: u32,
        height: u32,
    }
    
    impl MockCamera {
        pub fn new() -> Self {
            Self {
                is_open: false,
                frame_counter: 0,
                width: 640,
                height: 480,
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
            self.is_open = true;
            Ok(())
        }
        
        fn capture_frame(&mut self) -> Result<Frame> {
            if !self.is_open {
                return Err(Error::Camera("Camera not open".to_string()));
            }
            
            self.frame_counter += 1;
            
            Ok(Frame::from_capture(
                self.generate_test_frame(),
                self.width,
                self.height,
                std::time::Instant::now(),
            ))
        }
        
        fn is_open(&self) -> bool {
            self.is_open
        }
    }
}
