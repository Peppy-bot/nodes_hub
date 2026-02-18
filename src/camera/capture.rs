use peppygen::exposed_topics::video_stream::{self, MessageHeader};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio_util::sync::CancellationToken;

use crate::types::{CameraConfig, Error, FrameId, Result};
use super::device::CameraDevice;
use crate::pipeline;

/// Camera capture loop configuration
const FRAME_RETRY_DELAY_MS: u64 = 10;
const STATUS_PRINT_INTERVAL_SECS: u64 = 3;

/// Run the camera capture loop with trait-based abstraction
///
/// This is the main entry point for camera capture. It opens the camera,
/// configures it, and enters a loop that captures frames, processes them,
/// and emits them to the video stream topic.
///
/// # Errors
/// 
/// Returns an error if:
/// - Camera cannot be opened or configured
/// - Thread panics during execution
pub async fn run_camera_capture_loop<C: CameraDevice + 'static>(
    mut camera: C,
    config: CameraConfig,
    node_runner: Arc<peppygen::NodeRunner>,
    cancel_token: CancellationToken,
) -> Result<()> {
    println!("[uvc_camera] Starting camera capture loop...");

    let frame_rate = config.frame_rate.as_u16();
    let frame_duration = Duration::from_millis(1000 / u64::from(frame_rate));
    
    // Open and configure camera (blocking operation, done before the loop)
    println!("[uvc_camera] Opening camera {}...", config.device_path);
    
    let resolution = config.resolution;
    let encoding = config.encoding;
    let frame_rate = config.frame_rate.as_u16();
    
    camera = match tokio::task::spawn_blocking(move || {
        camera.open(&config)?;
        Ok::<_, Error>(camera)
    })
    .await
    {
        Err(join_err) => return Err(Error::ThreadPanic(format!("Camera open task panicked: {}", join_err))),
        Ok(result) => result?,
    };
    
    println!(
        "[uvc_camera] Camera configured: {}x{} @ {} fps, encoding: {}",
        resolution.width(),
        resolution.height(),
        frame_rate,
        encoding
    );

    // Run the capture loop in a blocking task
    tokio::task::spawn_blocking(move || {
        let mut frame_id = FrameId::default();
        let mut last_print_time = Instant::now();

        loop {
            if cancel_token.is_cancelled() {
                println!("[uvc_camera] Shutdown requested, stopping camera capture loop");
                break;
            }

            // Capture frame from camera
            let raw_frame = match camera.capture_frame() {
                Ok(frame) => frame,
                Err(e) => {
                    tracing::warn!("Failed to capture frame: {}", e);
                    std::thread::sleep(Duration::from_millis(FRAME_RETRY_DELAY_MS));
                    continue;
                }
            };

            // Process frame (convert encoding if needed)
            let frame = match pipeline::process_frame(raw_frame, frame_id, encoding) {
                Ok(frame) => frame,
                Err(e) => {
                    tracing::warn!("Failed to process frame: {}", e);
                    std::thread::sleep(Duration::from_millis(FRAME_RETRY_DELAY_MS));
                    continue;
                }
            };

            let header = MessageHeader {
                stamp: SystemTime::now(),
                frame_id: frame.frame_id().as_u32(),
            };

            // Emit frame using block_in_place to call async code from blocking context
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    if let Err(e) = video_stream::emit(
                        &node_runner,
                        header,
                        frame.encoding().to_string(),
                        frame.width_u32(),
                        frame.height_u32(),
                        frame.data().to_vec(),
                    )
                    .await {
                        tracing::warn!("Failed to emit frame: {}", e);
                    }
                });
            });

            if last_print_time.elapsed().as_secs() >= STATUS_PRINT_INTERVAL_SECS {
                println!("[uvc_camera] Emitted frame {}", frame.frame_id().as_u32());
                last_print_time = Instant::now();
            }

            frame_id = frame_id.next();

            // Rate limiting to target FPS
            std::thread::sleep(frame_duration);
        }
        
        Ok::<(), Error>(())
    })
    .await
    .map_err(|join_err| Error::ThreadPanic(format!("Camera capture task panicked: {}", join_err)))??;
    
    Ok(())
}

/// Helper function to create and run the capture loop with Nokhwa camera
pub async fn run_nokhwa_capture_loop(
    config: CameraConfig,
    node_runner: Arc<peppygen::NodeRunner>,
    cancel_token: CancellationToken,
) -> Result<()> {
    let camera = super::NokhwaCamera::new();
    run_camera_capture_loop(camera, config, node_runner, cancel_token).await
}
