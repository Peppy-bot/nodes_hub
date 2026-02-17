use peppygen::exposed_services::video_stream_info;
use std::sync::Arc;

use crate::camera::CameraParameters;

/// Listen for and handle video stream info service requests
pub async fn listen_for_video_stream_info_requests(
    node_runner: Arc<peppygen::NodeRunner>,
    params: CameraParameters,
) {
    loop {
        if let Err(e) = video_stream_info::handle_next_request(&node_runner, |_request| {
            Ok(video_stream_info::Response::new(
                u32::from(params.resolution.width),
                u32::from(params.resolution.height),
                u8::try_from(params.frame_rate).unwrap_or(u8::MAX),
                params.encoding.to_string(),
            ))
        })
        .await
        {
            tracing::error!("video_stream_info service error: {e:?}");
        }
    }
}
