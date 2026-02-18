use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution as NokhwaResolution,
};
use nokhwa::Camera;

use crate::types::{Error, RawFrame, Resolution, Result};
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
    fn open(&mut self, device_path: &str, resolution: Resolution, frame_rate: u16) -> Result<()> {
        let index = parse_camera_index(device_path)?;
        
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(
            CameraFormat::new(
                NokhwaResolution::new(resolution.width_u32(), resolution.height_u32()),
                FrameFormat::RAWRGB,
                u32::from(frame_rate),
            ),
        ));
        
        let camera = Camera::new(CameraIndex::Index(index), requested)
            .map_err(|e| Error::Camera(format!("Failed to open camera {}: {}", device_path, e)))?;
        
        self.camera = Some(SendableCamera(camera));
        Ok(())
    }
    
    fn capture_frame(&mut self) -> Result<RawFrame> {
        let camera = self.camera.as_mut()
            .ok_or_else(|| Error::Camera("Camera not open".to_string()))?;
        
        let frame = camera.0
            .frame()
            .map_err(|e| Error::Camera(format!("Failed to capture frame: {}", e)))?;
        
        let buffer = frame.buffer_bytes().to_vec();
        let resolution = frame.resolution();
        let timestamp = std::time::Instant::now();
        
        Ok(RawFrame::new(
            buffer,
            resolution.width_x as u16,
            resolution.height_y as u16,
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
    fn test_parse_camera_index() {
        assert_eq!(parse_camera_index("/dev/video0").unwrap(), 0);
        assert_eq!(parse_camera_index("/dev/video1").unwrap(), 1);
        assert_eq!(parse_camera_index("/dev/video42").unwrap(), 42);
        
        assert!(parse_camera_index("/dev/video").is_err());
        assert!(parse_camera_index("/dev/camera0").is_err());
        assert!(parse_camera_index("video0").is_err());
    }
}
