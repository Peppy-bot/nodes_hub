use nokhwa::Camera;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    CameraFormat, CameraIndex, ControlValueSetter, FrameFormat, KnownCameraControl,
    RequestedFormat, RequestedFormatType, Resolution as NokhwaResolution,
};

use super::device::CameraDevice;
use crate::camera::controls::{
    CameraControlRequest, ControlResult, ExposureMode, WhiteBalanceMode,
};
use crate::types::{CameraConfig, Encoding, Error, Frame, Result};

use v4l2_sys_mit::{
    V4L2_CID_AUTO_WHITE_BALANCE, V4L2_CID_EXPOSURE_ABSOLUTE, V4L2_CID_EXPOSURE_AUTO,
};

/// V4L2_EXPOSURE_AUTO = 0 (camera controls exposure automatically)
const V4L2_EXPOSURE_AUTO_VALUE: i64 = 0;
/// V4L2_EXPOSURE_MANUAL = 1 (manual exposure value via V4L2_CID_EXPOSURE_ABSOLUTE)
const V4L2_EXPOSURE_MANUAL_VALUE: i64 = 1;

/// Nokhwa-based camera implementation
///
/// Note: Camera from nokhwa doesn't implement Send, but we use it in a single-threaded
/// context (spawn_blocking) where it's never actually sent between threads during execution.
/// The Send bound is required only for moving into the blocking task initially.
pub struct NokhwaCamera {
    camera: Option<SendableCamera>,
    /// The actual camera encoding negotiated after `open_stream()`. The camera
    /// driver may return a format different from what was requested, so this is
    /// read back from `camera.camera_format()` rather than taken from the config.
    actual_camera_encoding: Option<Encoding>,
}

/// Wrapper to make Camera Send-safe
///
/// SAFETY: Camera is used only within a single thread (spawn_blocking).
/// It's never accessed from multiple threads concurrently.
struct SendableCamera(Camera);
unsafe impl Send for SendableCamera {}

impl NokhwaCamera {
    pub fn new() -> Self {
        Self {
            camera: None,
            actual_camera_encoding: None,
        }
    }
}

impl Default for NokhwaCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraDevice for NokhwaCamera {
    fn open(&mut self, config: &CameraConfig) -> Result<()> {
        println!(
            "[uvc_camera] Opening nokhwa camera {}...",
            config.device_path
        );
        let index = parse_camera_index(&config.device_path)?;
        println!("[uvc_camera] Parsed index {}.", index);
        let frame_rate = config.frame_rate.as_u16();
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(
                NokhwaResolution::new(config.resolution.width(), config.resolution.height()),
                encoding_to_frame_format(config.camera_encoding),
                u32::from(frame_rate),
            )));

        println!(
            "[uvc_camera] Requesting format: {}x{} @ {} fps.",
            config.resolution.width(),
            config.resolution.height(),
            frame_rate
        );

        let mut camera = Camera::new(CameraIndex::Index(index), requested).map_err(|e| {
            Error::Camera(format!(
                "Failed to open camera {}: {}",
                config.device_path, e
            ))
        })?;

        println!(
            "[uvc_camera] Camera {} opened successfully.",
            config.device_path
        );

        camera.open_stream().map_err(|e| {
            Error::Camera(format!(
                "Failed to start stream for {}: {}",
                config.device_path, e
            ))
        })?;

        // Read back the format actually negotiated by the driver — it may differ
        // from what was requested (e.g. hardware only supports MJPEG).
        let negotiated = camera.camera_format().format();
        let actual_encoding = frame_format_to_encoding(negotiated);
        println!(
            "[uvc_camera] Camera {} stream started. Requested format: {}, negotiated format: {} ({:?})",
            config.device_path, config.camera_encoding, actual_encoding, negotiated,
        );

        self.actual_camera_encoding = Some(actual_encoding);
        self.camera = Some(SendableCamera(camera));
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<Frame> {
        let encoding = self
            .actual_camera_encoding
            .ok_or_else(|| Error::Camera("Camera not open".to_string()))?;
        let camera = self
            .camera
            .as_mut()
            .ok_or_else(|| Error::Camera("Camera not open".to_string()))?;

        let frame = camera
            .0
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
            encoding,
        ))
    }

    fn is_open(&self) -> bool {
        self.camera.is_some()
    }

    fn apply_control(&mut self, request: &CameraControlRequest) -> ControlResult {
        let camera = match self.camera.as_mut() {
            Some(c) => &mut c.0,
            None => return ControlResult::err("Camera not open"),
        };

        match request {
            CameraControlRequest::SetBrightness { value } => {
                set_integer_control(camera, KnownCameraControl::Brightness, *value)
            }
            CameraControlRequest::SetContrast { value } => {
                set_integer_control(camera, KnownCameraControl::Contrast, *value)
            }
            CameraControlRequest::SetGain { value } => {
                set_integer_control(camera, KnownCameraControl::Gain, *value)
            }
            CameraControlRequest::SetExposure { mode, value } => set_exposure(camera, mode, *value),
            CameraControlRequest::SetWhiteBalance { mode, temperature } => {
                set_white_balance(camera, mode, *temperature)
            }
        }
    }
}

/// Set a simple integer camera control and read back the current value
fn set_integer_control(camera: &mut Camera, kind: KnownCameraControl, value: i32) -> ControlResult {
    match camera.set_camera_control(kind, ControlValueSetter::Integer(i64::from(value))) {
        Ok(()) => {
            let current = camera
                .camera_control(kind)
                .ok()
                .and_then(|c| c.value().as_integer().copied())
                .map(|v| v as i32)
                .unwrap_or(value);
            ControlResult::ok(format!("{:?} set to {}", kind, current), current)
        }
        Err(e) => ControlResult::err(format!("Failed to set {:?}: {}", kind, e)),
    }
}

/// Set exposure mode and optionally the absolute exposure value
fn set_exposure(camera: &mut Camera, mode: &ExposureMode, value: i32) -> ControlResult {
    let auto_value = match mode {
        ExposureMode::Auto => V4L2_EXPOSURE_AUTO_VALUE,
        ExposureMode::Manual => V4L2_EXPOSURE_MANUAL_VALUE,
    };

    if let Err(e) = camera.set_camera_control(
        KnownCameraControl::Other(V4L2_CID_EXPOSURE_AUTO as u128),
        ControlValueSetter::Integer(auto_value),
    ) {
        return ControlResult::err(format!("Failed to set exposure mode: {}", e));
    }

    match mode {
        ExposureMode::Auto => ControlResult::ok("Exposure set to auto mode", -1),
        ExposureMode::Manual => {
            // Set absolute exposure value (in 100µs units for V4L2)
            if let Err(e) = camera.set_camera_control(
                KnownCameraControl::Other(V4L2_CID_EXPOSURE_ABSOLUTE as u128),
                ControlValueSetter::Integer(i64::from(value)),
            ) {
                return ControlResult::err(format!(
                    "Exposure mode set to manual but value failed: {}",
                    e
                ));
            }

            let current = camera
                .camera_control(KnownCameraControl::Other(
                    V4L2_CID_EXPOSURE_ABSOLUTE as u128,
                ))
                .ok()
                .and_then(|c| c.value().as_integer().copied())
                .map(|v| v as i32)
                .unwrap_or(value);

            ControlResult::ok(
                format!("Exposure set to manual, value {}", current),
                current,
            )
        }
    }
}

/// Set white balance mode and optionally the temperature
fn set_white_balance(
    camera: &mut Camera,
    mode: &WhiteBalanceMode,
    temperature: i32,
) -> ControlResult {
    let auto_bool = matches!(mode, WhiteBalanceMode::Auto);

    if let Err(e) = camera.set_camera_control(
        KnownCameraControl::Other(V4L2_CID_AUTO_WHITE_BALANCE as u128),
        ControlValueSetter::Boolean(auto_bool),
    ) {
        return ControlResult::err(format!("Failed to set white balance mode: {}", e));
    }

    match mode {
        WhiteBalanceMode::Auto => ControlResult::ok("White balance set to auto mode", -1),
        WhiteBalanceMode::Manual => {
            if let Err(e) = camera.set_camera_control(
                KnownCameraControl::WhiteBalance,
                ControlValueSetter::Integer(i64::from(temperature)),
            ) {
                return ControlResult::err(format!(
                    "White balance mode set to manual but temperature failed: {}",
                    e
                ));
            }

            let current = camera
                .camera_control(KnownCameraControl::WhiteBalance)
                .ok()
                .and_then(|c| c.value().as_integer().copied())
                .map(|v| v as i32)
                .unwrap_or(temperature);

            ControlResult::ok(
                format!("White balance set to manual, temperature {}K", current),
                current,
            )
        }
    }
}

/// Map an [`Encoding`] to the corresponding nokhwa [`FrameFormat`] to request
/// from the camera hardware.
fn encoding_to_frame_format(encoding: Encoding) -> FrameFormat {
    match encoding {
        Encoding::Mjpeg => FrameFormat::MJPEG,
        Encoding::Rgb8 => FrameFormat::RAWRGB,
        Encoding::Bgr8 => FrameFormat::RAWBGR,
    }
}

/// Map a negotiated nokhwa [`FrameFormat`] back to our [`Encoding`].
///
/// Used to record what the camera driver actually settled on after `open_stream`,
/// which may differ from what was requested.
fn frame_format_to_encoding(fmt: FrameFormat) -> Encoding {
    match fmt {
        FrameFormat::MJPEG => Encoding::Mjpeg,
        FrameFormat::RAWRGB => Encoding::Rgb8,
        FrameFormat::RAWBGR => Encoding::Bgr8,
        other => {
            tracing::warn!(
                "Unknown camera FrameFormat {:?}, falling back to Rgb8",
                other
            );
            Encoding::Rgb8
        }
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
    fn test_encoding_to_frame_format() {
        assert_eq!(
            encoding_to_frame_format(Encoding::Rgb8),
            FrameFormat::RAWRGB
        );
        assert_eq!(
            encoding_to_frame_format(Encoding::Bgr8),
            FrameFormat::RAWBGR
        );
        assert_eq!(
            encoding_to_frame_format(Encoding::Mjpeg),
            FrameFormat::MJPEG
        );
    }

    #[test]
    fn test_frame_format_to_encoding() {
        assert_eq!(
            frame_format_to_encoding(FrameFormat::RAWRGB),
            Encoding::Rgb8
        );
        assert_eq!(
            frame_format_to_encoding(FrameFormat::RAWBGR),
            Encoding::Bgr8
        );
        assert_eq!(
            frame_format_to_encoding(FrameFormat::MJPEG),
            Encoding::Mjpeg
        );
    }

    #[test]
    fn test_encoding_frame_format_roundtrip() {
        for enc in [Encoding::Rgb8, Encoding::Bgr8, Encoding::Mjpeg] {
            let fmt = encoding_to_frame_format(enc);
            let back = frame_format_to_encoding(fmt);
            assert_eq!(back, enc, "Roundtrip failed for {enc:?}");
        }
    }

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
