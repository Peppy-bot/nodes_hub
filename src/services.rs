use peppygen::exposed_services::video_stream_info;
use std::sync::Arc;

use crate::types::CameraConfig;

/// Listen for and handle video stream info service requests
pub async fn listen_for_video_stream_info_requests(
    node_runner: Arc<peppygen::NodeRunner>,
    config: CameraConfig,
) {
    loop {
        if let Err(e) = video_stream_info::handle_next_request(&node_runner, |_request| {
            Ok(video_stream_info::Response::new(
                config.resolution.width(),
                config.resolution.height(),
                u8::try_from(config.frame_rate.as_u16()).unwrap_or(u8::MAX),
                config.encoding.to_string(),
            ))
        })
        .await
        {
            tracing::error!("video_stream_info service error: {e:?}");
        }
    }
}
