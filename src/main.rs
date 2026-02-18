use peppygen::parameters::video::{Resolution as VideoResolution, Video};
use peppygen::{NodeBuilder, Parameters, Result, StandaloneConfig};
use std::sync::Arc;

use uvc_camera::camera::run_nokhwa_capture_loop;
use uvc_camera::encoding::Encoding as OldEncoding;
use uvc_camera::old_camera::CameraParameters;
use uvc_camera::services::listen_for_video_stream_info_requests;
use uvc_camera::types::{CameraConfigBuilder, Encoding, Resolution};

fn main() -> Result<()> {
    // Example configuration for standalone execution
    let standalone_config = StandaloneConfig::new().with_parameters(&Parameters {
        device: "/dev/video0".to_string(),
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
            let device = args.device.clone();

            println!(
                "[uvc_camera] Video params: {}x{} @ {} fps, encoding: {}",
                video_params.resolution.width,
                video_params.resolution.height,
                video_params.frame_rate,
                video_params.encoding
            );

            println!("[uvc_camera] Device: {device}");

            // Parse and validate encoding format (both old and new)
            let encoding = video_params.encoding.parse::<Encoding>()
                .unwrap_or_else(|e| {
                    panic!("Invalid encoding format '{}': {}", video_params.encoding, e)
                });
            
            let old_encoding = video_params.encoding.parse::<OldEncoding>()
                .unwrap_or_else(|e| {
                    panic!("Invalid encoding format '{}': {}", video_params.encoding, e)
                });

            // Validate resolution (for early error detection)
            let _resolution = Resolution::new(video_params.resolution.width, video_params.resolution.height);

            // Create camera configuration
            let camera_config = CameraConfigBuilder::new()
                .device_path(device.clone())
                .resolution(video_params.resolution.width, video_params.resolution.height)
                .frame_rate(video_params.frame_rate)
                .encoding(encoding)
                .build()
                .unwrap_or_else(|e| panic!("Failed to create camera config: {}", e));

            // Legacy parameters for services (to be migrated)
            let camera_params = CameraParameters {
                resolution: video_params.resolution.clone(),
                frame_rate: video_params.frame_rate,
                encoding: old_encoding,
                device_path: device,
            };

            // Service to expose camera info
            let service_node_runner = Arc::clone(&node_runner);
            let service_params = camera_params.clone();
            tokio::spawn(async move {
                listen_for_video_stream_info_requests(service_node_runner, service_params).await;
            });

            // Long running capture task
            let cancel_token = node_runner.cancellation_token().clone();
            tokio::spawn(async move {
                if let Err(e) = run_nokhwa_capture_loop(camera_config, node_runner, cancel_token).await
                {
                    tracing::error!("Camera capture loop error: {e:?}");
                }
            });

            Ok(())
        })
}
