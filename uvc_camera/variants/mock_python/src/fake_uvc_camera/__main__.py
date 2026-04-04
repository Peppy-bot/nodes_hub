import av
import asyncio
import json
import time
from pathlib import Path


from peppygen import NodeBuilder, NodeRunner, StandaloneConfig
from peppygen.exposed_services import video_stream_info
from peppygen.emitted_topics import video_stream
from peppygen.emitted_topics.video_stream import MessageHeader
from peppygen.parameters import Parameters

ASSETS_DIR = Path(__file__).resolve().parent / "assets"


def get_source_video_fps(video_path: Path) -> int:
    container = av.open(str(video_path))
    stream = container.streams.video[0]
    fps = stream.average_rate
    container.close()
    if fps and fps > 0:
        return round(float(fps))
    return 30  # Default fallback


async def setup(params: Parameters, node_runner: NodeRunner) -> list[asyncio.Task]:
    video_params = params.video

    print(
        f"[uvc_camera] Video params: {video_params.resolution.width}x{video_params.resolution.height} "
        f"@ {video_params.frame_rate} fps, encoding: {video_params.topic_encoding}"
    )

    # Validate encoding - this node outputs RGB24 format data
    encoding = video_params.topic_encoding
    if encoding not in ("rgb8", "rgb"):
        raise ValueError(
            f"Invalid encoding '{encoding}'. This camera node outputs RGB24 data, "
            "so encoding must be 'rgb8' or 'rgb'"
        )

    # Probe source video to get its actual frame rate
    video_path = ASSETS_DIR / "robot.mp4"
    if not video_path.exists():
        raise FileNotFoundError(f"Video file not found: {video_path}")

    actual_fps = get_source_video_fps(video_path)
    print(f"[uvc_camera] Detected source video frame rate: {actual_fps} fps")

    return [
        # Service to expose camera info
        asyncio.create_task(
            listen_for_video_stream_info_requests(node_runner, video_params, actual_fps)
        ),
        # Video loop
        asyncio.create_task(run_video_loop(node_runner, video_params)),
    ]


async def run_video_loop(node_runner: NodeRunner, video_params):
    print("[uvc_camera] Starting video loop...")
    video_path = ASSETS_DIR / "robot.mp4"

    if not video_path.exists():
        raise FileNotFoundError(f"Video file not found: {video_path}")
    print(f"[uvc_camera] Video file found: {video_path}")

    frame_id = 0
    last_print_time = time.monotonic()

    width = video_params.resolution.width
    height = video_params.resolution.height
    encoding = video_params.topic_encoding
    frame_duration = 1.0 / video_params.frame_rate

    while True:
        print("[uvc_camera] Opening video file for playback...")
        container = av.open(str(video_path))

        for frame in container.decode(video=0):
            # Reformat to RGB24 at target resolution
            rgb_frame = frame.reformat(width=width, height=height, format="rgb24")
            data = bytes(rgb_frame.planes[0])

            header = MessageHeader(
                stamp=time.time(),
                frame_id=frame_id,
            )

            await video_stream.emit(node_runner, header, encoding, width, height, data)

            if time.monotonic() - last_print_time >= 3:
                print(f"[uvc_camera] Emitted frame {frame_id}")
                last_print_time = time.monotonic()

            frame_id = (frame_id + 1) % (2**32)

            await asyncio.sleep(frame_duration)

        container.close()
        print("[uvc_camera] Video ended, restarting from beginning...")


async def listen_for_video_stream_info_requests(
    node_runner: NodeRunner, video_params, actual_fps: int
):
    while True:
        try:
            await video_stream_info.handle_next_request(
                node_runner,
                lambda _request: video_stream_info.Response(
                    width=video_params.resolution.width,
                    height=video_params.resolution.height,
                    frames_per_second=actual_fps,
                    encoding=video_params.topic_encoding,
                ),
            )
        except Exception as e:
            print(f"get_camera_info service error: {e}")


def main():
    # Fallback configuration for standalone execution (e.g., `uv run`).
    # Ignored when the node is launched by the peppy daemon, which provides its own parameters.
    standalone_config = StandaloneConfig()

    mock_params_path = (
        Path(__file__).resolve().parent.parent.parent / "mock_parameters.json"
    )
    if mock_params_path.exists():
        mock_params = json.loads(mock_params_path.read_text())
        standalone_config = standalone_config.with_parameters(mock_params)

    NodeBuilder().standalone(standalone_config).run(setup)


if __name__ == "__main__":
    main()
