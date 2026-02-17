use peppygen::exposed_services::video_stream_info;
use peppygen::parameters;
use std::sync::Arc;

/// Listen for and handle video stream info service requests
pub async fn listen_for_video_stream_info_requests(
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
