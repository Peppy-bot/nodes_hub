use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution};
use nokhwa::Camera;
use peppygen::exposed_topics::video_stream::{self, MessageHeader};
use peppygen::parameters::video::Resolution as VideoResolution;
use peppygen::Result;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio_util::sync::CancellationToken;

use crate::conversion::{convert_rgb_to_bgr, encode_jpeg};
use crate::encoding::Encoding;

// Constants
const FRAME_RETRY_DELAY_MS: u64 = 10;
const STATUS_PRINT_INTERVAL_SECS: u64 = 3;

/// Camera configuration parameters
#[derive(Debug, Clone)]
pub struct CameraParameters {
    pub resolution: VideoResolution,
    pub frame_rate: u16,
    pub encoding: Encoding,
    pub device_path: String,
}

/// Parse camera index from device path
/// Supports formats like "/dev/video0", "/dev/video1", or just "0", "1"
pub fn parse_camera_index(device_path: &str) -> CameraIndex {
    // Try to extract number from paths like "/dev/video0" or "/dev/video1"
    if let Some(video_prefix_pos) = device_path.rfind("video") {
        let number_str = &device_path[video_prefix_pos + 5..];
        if let Ok(index) = number_str.parse::<u32>() {
            return CameraIndex::Index(index);
        }
    }
    
    // Try parsing the entire string as a number
    if let Ok(index) = device_path.parse::<u32>() {
        return CameraIndex::Index(index);
    }
    
    // Default to index 0 if parsing fails
    tracing::warn!(
        "Could not parse camera index from '{}', defaulting to index 0",
        device_path
    );
    CameraIndex::Index(0)
}

/// Run the camera capture loop
pub async fn run_camera_capture_loop(
    node_runner: Arc<peppygen::NodeRunner>,
    params: CameraParameters,
    cancel_token: CancellationToken,
) -> Result<()> {
    println!("[uvc_camera] Starting camera capture loop...");

    let width = params.resolution.width as u32;
    let height = params.resolution.height as u32;
    let frame_duration = Duration::from_millis(1000 / params.frame_rate as u64);
    
    // Run the entire camera loop in a blocking task
    tokio::task::spawn_blocking(move || {
        // Parse camera index from device path (e.g., "/dev/video0" -> 0)
        let camera_index = parse_camera_index(&params.device_path);
        let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);

        println!("[uvc_camera] Opening camera {}...", params.device_path);
        let mut camera = Camera::new(camera_index, requested_format)
            .unwrap_or_else(|_| panic!("Failed to open camera {}", params.device_path));

        // Set camera resolution
        let resolution = Resolution::new(width, height);
        camera
            .set_resolution(resolution)
            .expect("Failed to set resolution");

        // Set frame rate
        camera
            .set_frame_rate(params.frame_rate as u32)
            .expect("Failed to set frame rate");

        println!(
            "[uvc_camera] Camera configured: {}x{} @ {} fps",
            width, height, params.frame_rate
        );

        // Open camera stream
        camera
            .open_stream()
            .expect("Failed to open camera stream");

        println!("[uvc_camera] Camera stream opened successfully");

        let mut frame_id: u32 = 0;
        let mut last_print_time = Instant::now();

        loop {
            if cancel_token.is_cancelled() {
                println!("[uvc_camera] Shutdown requested, stopping camera capture loop");
                break;
            }

            // Capture frame from camera
            let frame = match camera.frame() {
                Ok(frame) => frame,
                Err(e) => {
                    tracing::warn!("Failed to capture frame: {}", e);
                    std::thread::sleep(Duration::from_millis(FRAME_RETRY_DELAY_MS));
                    continue;
                }
            };

            // Convert frame to RGB bytes
            let rgb_frame = match frame.decode_image::<RgbFormat>() {
                Ok(img) => img,
                Err(e) => {
                    tracing::warn!("Failed to decode frame: {}", e);
                    std::thread::sleep(Duration::from_millis(FRAME_RETRY_DELAY_MS));
                    continue;
                }
            };

            let rgb_data = rgb_frame.into_raw();

            // Convert data based on requested encoding
            let data = match params.encoding {
                Encoding::Rgb8 => rgb_data,
                Encoding::Bgr8 => convert_rgb_to_bgr(rgb_data),
                Encoding::Mjpeg => match encode_jpeg(&rgb_data, width, height) {
                    Ok(jpeg_data) => jpeg_data,
                    Err(e) => {
                        tracing::warn!("Failed to encode JPEG: {}", e);
                        std::thread::sleep(Duration::from_millis(FRAME_RETRY_DELAY_MS));
                        continue;
                    }
                },
            };

            let header = MessageHeader {
                stamp: SystemTime::now(),
                frame_id,
            };

            // Emit frame using block_in_place to call async code from blocking context
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    if let Err(e) = video_stream::emit(
                        &node_runner,
                        header,
                        params.encoding.to_string(),
                        width,
                        height,
                        data,
                    )
                    .await {
                        tracing::warn!("Failed to emit frame: {}", e);
                    }
                })
            });

            if last_print_time.elapsed().as_secs() >= STATUS_PRINT_INTERVAL_SECS {
                println!("[uvc_camera] Emitted frame {}", frame_id);
                last_print_time = Instant::now();
            }

            frame_id = frame_id.wrapping_add(1);

            // Rate limiting to target FPS
            std::thread::sleep(frame_duration);
        }
    })
    .await
    .expect("Camera thread panicked");

    Ok(())
}
