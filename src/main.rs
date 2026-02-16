use image::{ImageBuffer, Rgb};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution};
use nokhwa::Camera;
use peppygen::exposed_services::video_stream_info;
use peppygen::exposed_topics::video_stream::{self, MessageHeader};
use peppygen::parameters::{
    self,
    device::Device,
    video::{Resolution as VideoResolution, Video},
};
use peppygen::{NodeBuilder, Parameters, Result, StandaloneConfig};
use std::io::Cursor;
use std::sync::Arc;
use std::time::{Instant, SystemTime};
use tokio_util::sync::CancellationToken;

fn main() -> Result<()> {
    // Example configuration for standalone execution
    let standalone_config = StandaloneConfig::new().with_parameters(&Parameters {
        device: Device {
            physical: "/dev/video0".to_string(),
            priority: "physical".to_string(),
        },
        video: Video {
            encoding: "rgb8".to_string(),
            frame_rate: 30,
            resolution: VideoResolution {
                width: 640,
                height: 480,
            },
        },
    });

    NodeBuilder::new()
        .standalone(standalone_config)
        .run(move |args: Parameters, node_runner| async move {
            let video_params = args.video.clone();
            let device_params = args.device.clone();

            println!(
                "[uvc_camera] Video params: {}x{} @ {} fps, encoding: {}",
                video_params.resolution.width,
                video_params.resolution.height,
                video_params.frame_rate,
                video_params.encoding
            );

            println!("[uvc_camera] Device: {}", device_params.physical);

            // Validate encoding format
            let encoding = &video_params.encoding;
            if encoding != "rgb8" && encoding != "bgr8" && encoding != "mjpeg" {
                panic!(
                    "Invalid encoding '{}'. Supported encodings are: 'rgb8', 'bgr8', 'mjpeg'",
                    encoding
                );
            }

            // Service to expose camera info
            let service_node_runner = Arc::clone(&node_runner);
            let service_video_params = video_params.clone();
            tokio::spawn(async move {
                listen_for_video_stream_info_requests(service_node_runner, service_video_params)
                    .await;
            });

            // Long running capture task
            let cancel_token = node_runner.cancellation_token().clone();
            tokio::spawn(async move {
                if let Err(e) = run_camera_capture_loop(node_runner, video_params, cancel_token).await
                {
                    tracing::error!("Camera capture loop error: {e:?}");
                }
            });

            Ok(())
        })
}

async fn run_camera_capture_loop(
    node_runner: Arc<peppygen::NodeRunner>,
    video_params: parameters::video::Video,
    cancel_token: CancellationToken,
) -> Result<()> {
    println!("[uvc_camera] Starting camera capture loop...");

    let width = video_params.resolution.width as u32;
    let height = video_params.resolution.height as u32;
    let encoding = video_params.encoding.clone();
    let frame_duration_ms = 1000 / video_params.frame_rate as u64;

    // Initialize camera in a blocking task since Camera is not Send
    let (width, height, frame_rate) = (width, height, video_params.frame_rate);
    
    // Run the entire camera loop in a blocking task
    tokio::task::spawn_blocking(move || {
        // Hardcoded to /dev/video0 as requested
        let camera_index = CameraIndex::Index(0);
        let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);

        println!("[uvc_camera] Opening camera /dev/video1...");
        let mut camera = Camera::new(camera_index, requested_format)
            .unwrap_or_else(|e| panic!("Failed to open camera /dev/video1: {}", e));

        // Set camera resolution
        let resolution = Resolution::new(width, height);
        camera
            .set_resolution(resolution)
            .unwrap_or_else(|e| panic!("Failed to set resolution: {}", e));

        // Set frame rate
        camera
            .set_frame_rate(frame_rate as u32)
            .unwrap_or_else(|e| panic!("Failed to set frame rate: {}", e));

        println!(
            "[uvc_camera] Camera configured: {}x{} @ {} fps",
            width, height, frame_rate
        );

        // Open camera stream
        camera
            .open_stream()
            .unwrap_or_else(|e| panic!("Failed to open camera stream: {}", e));

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
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
            };

            // Convert frame to RGB bytes
            let rgb_frame = match frame.decode_image::<RgbFormat>() {
                Ok(img) => img,
                Err(e) => {
                    tracing::warn!("Failed to decode frame: {}", e);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
            };

            let rgb_data = rgb_frame.into_raw();

            // Convert data based on requested encoding
            let data = match encoding.as_str() {
                "rgb8" => rgb_data,
                "bgr8" => {
                    // Convert RGB to BGR by swapping R and B channels
                    let mut bgr_data = rgb_data;
                    for chunk in bgr_data.chunks_exact_mut(3) {
                        chunk.swap(0, 2); // Swap R and B
                    }
                    bgr_data
                }
                "mjpeg" => {
                    // Encode as JPEG
                    match encode_jpeg(&rgb_data, width, height) {
                        Ok(jpeg_data) => jpeg_data,
                        Err(e) => {
                            tracing::warn!("Failed to encode JPEG: {}", e);
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        }
                    }
                }
                _ => rgb_data, // Should never happen due to validation
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
                        encoding.clone(),
                        width,
                        height,
                        data,
                    )
                    .await {
                        tracing::warn!("Failed to emit frame: {}", e);
                    }
                })
            });

            if last_print_time.elapsed().as_secs() >= 3 {
                println!("[uvc_camera] Emitted frame {}", frame_id);
                last_print_time = Instant::now();
            }

            frame_id = frame_id.wrapping_add(1);

            // Rate limiting to target FPS
            std::thread::sleep(std::time::Duration::from_millis(frame_duration_ms));
        }
    })
    .await
    .expect("Camera thread panicked");

    Ok(())
}

/// Encode RGB data as JPEG
fn encode_jpeg(rgb_data: &[u8], width: u32, height: u32) -> std::result::Result<Vec<u8>, String> {
    // Create image buffer from RGB data
    let img = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, rgb_data)
        .ok_or_else(|| "Failed to create image buffer".to_string())?;

    // Encode as JPEG with quality 85
    let mut jpeg_data = Vec::new();
    let mut cursor = Cursor::new(&mut jpeg_data);
    
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 85);
    encoder
        .encode(
            img.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("JPEG encoding failed: {}", e))?;

    Ok(jpeg_data)
}

async fn listen_for_video_stream_info_requests(
    node_runner: Arc<peppygen::NodeRunner>,
    video_params: parameters::video::Video,
) {
    loop {
        let params = video_params.clone();
        if let Err(e) = video_stream_info::handle_next_request(&node_runner, move |_request| {
            Ok(video_stream_info::Response::new(
                params.resolution.width as u32,
                params.resolution.height as u32,
                params.frame_rate as u8,
                params.encoding.clone(),
            ))
        })
        .await
        {
            tracing::error!("video_stream_info service error: {e:?}");
        }
    }
}
