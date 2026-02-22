use peppygen::parameters::video::{Resolution as VideoResolution, Video};
use peppygen::{NodeBuilder, Parameters, Result, StandaloneConfig};
use std::sync::Arc;

use uvc_camera::camera::{create_control_channel, run_nokhwa_capture_loop};
use uvc_camera::services::{
    listen_for_set_brightness_requests, listen_for_set_contrast_requests,
    listen_for_set_exposure_requests, listen_for_set_gain_requests,
    listen_for_set_white_balance_requests, listen_for_video_stream_info_requests,
};
use uvc_camera::types::{CameraConfigBuilder, Encoding};

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

            // Parse and validate encoding format
            let encoding = video_params.encoding.parse::<Encoding>()
                .unwrap_or_else(|e| {
                    panic!("Invalid encoding format '{}': {}", video_params.encoding, e)
                });

            // Create camera configuration
            let camera_config = CameraConfigBuilder::new()
                .device_path(device.clone())
                .resolution(video_params.resolution.width, video_params.resolution.height)
                .frame_rate(video_params.frame_rate)
                .encoding(encoding)
                .build()
                .unwrap_or_else(|e| panic!("Failed to create camera config: {}", e));

            // Create control channel shared between service handlers and capture loop
            let (control_tx, control_rx) = create_control_channel();

            // ── video_stream_info ──────────────────────────────────────────
            let info_runner = Arc::clone(&node_runner);
            let info_config = camera_config.clone();
            tokio::spawn(async move {
                listen_for_video_stream_info_requests(info_runner, info_config).await;
            });

            // ── set_exposure ───────────────────────────────────────────────
            let exposure_runner = Arc::clone(&node_runner);
            let exposure_tx = control_tx.clone();
            tokio::spawn(async move {
                listen_for_set_exposure_requests(exposure_runner, exposure_tx).await;
            });

            // ── set_white_balance ──────────────────────────────────────────
            let wb_runner = Arc::clone(&node_runner);
            let wb_tx = control_tx.clone();
            tokio::spawn(async move {
                listen_for_set_white_balance_requests(wb_runner, wb_tx).await;
            });

            // ── set_gain ───────────────────────────────────────────────────
            let gain_runner = Arc::clone(&node_runner);
            let gain_tx = control_tx.clone();
            tokio::spawn(async move {
                listen_for_set_gain_requests(gain_runner, gain_tx).await;
            });

            // ── set_brightness ─────────────────────────────────────────────
            let brightness_runner = Arc::clone(&node_runner);
            let brightness_tx = control_tx.clone();
            tokio::spawn(async move {
                listen_for_set_brightness_requests(brightness_runner, brightness_tx).await;
            });

            // ── set_contrast ───────────────────────────────────────────────
            let contrast_runner = Arc::clone(&node_runner);
            let contrast_tx = control_tx.clone();
            tokio::spawn(async move {
                listen_for_set_contrast_requests(contrast_runner, contrast_tx).await;
            });

            // ── capture loop (long-running) ────────────────────────────────
            let cancel_token = node_runner.cancellation_token().clone();
            tokio::spawn(async move {
                if let Err(e) =
                    run_nokhwa_capture_loop(camera_config, node_runner, cancel_token, control_rx)
                        .await
                {
                    tracing::error!("Camera capture loop error: {e:?}");
                }
            });

            Ok(())
        })
}
