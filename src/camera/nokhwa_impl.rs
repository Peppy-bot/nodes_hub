use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution as NokhwaResolution,
};
use nokhwa::Camera;

use crate::types::{CameraConfig, Error, Frame, Result};
use super::device::CameraDevice;

/// Nokhwa-based camera implementation
/// 
/// Note: Camera from nokhwa doesn't implement Send, but we use it in a single-threaded
/// context (spawn_blocking) where it's never actually sent between threads during execution.
/// The Send bound is required only for moving into the blocking task initially.
pub struct NokhwaCamera {
    camera: Option<SendableCamera>,
}

/// Wrapper to make Camera Send-safe
/// 
/// SAFETY: Camera is used only within a single thread (spawn_blocking).
/// It's never accessed from multiple threads concurrently.
struct SendableCamera(Camera);
unsafe impl Send for SendableCamera {}

impl NokhwaCamera {
    pub fn new() -> Self {
        Self { camera: None }
    }
}

impl Default for NokhwaCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraDevice for NokhwaCamera {
    fn open(&mut self, config: &CameraConfig) -> Result<()> {
        let index = parse_camera_index(&config.device_path)?;
        
        let frame_rate = config.frame_rate.as_u16();
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(
            CameraFormat::new(
                NokhwaResolution::new(config.resolution.width(), config.resolution.height()),
                FrameFormat::RAWRGB,
                u32::from(frame_rate),
            ),
        ));
        
        let camera = Camera::new(CameraIndex::Index(index), requested)
            .map_err(|e| Error::Camera(format!("Failed to open camera {}: {}", config.device_path, e)))?;
        
        self.camera = Some(SendableCamera(camera));
        Ok(())
    }
    
    fn capture_frame(&mut self) -> Result<Frame> {
        let camera = self.camera.as_mut()
            .ok_or_else(|| Error::Camera("Camera not open".to_string()))?;
        
        let frame = camera.0
            .frame()
            .map_err(|e| Error::Camera(format!("Failed to capture frame: {}", e)))?;
        
        let buffer = frame.buffer_bytes().to_vec();
        let resolution = frame.resolution();
        let timestamp = std::time::Instant::now();
        
        Ok(Frame::from_capture(
            buffer,
            resolution.width_x,
            resolution.height_y,
            timestamp,
        ))
    }
    
    fn is_open(&self) -> bool {
        self.camera.is_some()
    }
}

/// Parse camera device path into index
fn parse_camera_index(device_path: &str) -> Result<u32> {
    if let Some(stripped) = device_path.strip_prefix("/dev/video") {
        stripped
            .parse::<u32>()
            .map_err(|_| Error::InvalidDevicePath(device_path.to_string()))
    } else {
        Err(Error::InvalidDevicePath(device_path.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_camera_index_valid() {
        assert_eq!(parse_camera_index("/dev/video0").unwrap(), 0);
        assert_eq!(parse_camera_index("/dev/video1").unwrap(), 1);
        assert_eq!(parse_camera_index("/dev/video42").unwrap(), 42);
        assert_eq!(parse_camera_index("/dev/video1000").unwrap(), 1000);
    }
    
    #[test]
    fn test_parse_camera_index_invalid() {
        // Only /dev/videoN format is accepted
        assert!(parse_camera_index("/dev/video").is_err());
        assert!(parse_camera_index("/dev/camera0").is_err());
        assert!(parse_camera_index("video0").is_err());
        assert!(parse_camera_index("0").is_err());
        assert!(parse_camera_index("").is_err());
        assert!(parse_camera_index("invalid").is_err());
    }
}
