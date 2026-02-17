use peppygen::parameters::video::{Resolution as VideoResolution, Video};
use peppygen::{NodeBuilder, Parameters, Result, StandaloneConfig};
use std::sync::Arc;

use uvc_camera::camera::{run_camera_capture_loop, CameraParameters};
use uvc_camera::encoding::Encoding;
use uvc_camera::services::listen_for_video_stream_info_requests;

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

            println!("[uvc_camera] Device: {}", device);

            // Parse and validate encoding format
            let encoding = video_params.encoding.parse::<Encoding>()
                .unwrap_or_else(|e| {
                    panic!("Invalid encoding format '{}': {}", video_params.encoding, e)
                });

            // Create camera parameters
            let camera_params = CameraParameters {
                resolution: video_params.resolution.clone(),
                frame_rate: video_params.frame_rate,
                encoding,
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
                if let Err(e) = run_camera_capture_loop(node_runner, camera_params, cancel_token).await
                {
                    tracing::error!("Camera capture loop error: {e:?}");
                }
            });

            Ok(())
        })
}
