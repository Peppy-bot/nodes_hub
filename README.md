# UVC Camera Node

A basic USB Video Class (UVC) camera node for the Peppy framework that captures real camera streams using the `nokhwa` library.

## Overview

This node captures video frames from a UVC-compatible camera and publishes them as RGB8 frames on the `video_stream` topic. It provides a simple interface for integrating USB cameras into Peppy-based robotics systems.

## Features

- **Real-time video capture** from UVC cameras
- **RGB8 encoding** output (24-bit RGB)
- **Configurable resolution and frame rate**
- **Camera info service** for querying capabilities
- **Graceful shutdown** handling
- **Cross-platform** support (Linux, macOS)

## Requirements

### Hardware
- UVC-compatible USB camera
- Camera must be accessible at `/dev/video1` (currently hardcoded)

### Software
- Rust (latest stable)
- Peppy framework
- Camera drivers:
  - **Linux**: V4L2 (Video4Linux2) - usually pre-installed
  - **macOS**: AVFoundation (built-in)

### Linux Permissions
On Linux, ensure your user has access to video devices:
```bash
sudo usermod -a -G video $USER
# Log out and back in for changes to take effect
```

## Building

```bash
cargo build --release
```

## Running

### Standalone Mode
```bash
cargo run --release
```

### With Peppy Daemon
```bash
# Add the node to peppy
peppy add /path/to/uvc_camera

# Launch the node
peppy launch uvc_camera
```

## Configuration

The node is configured via the `peppy.json5` parameters:

```json5
{
  device: {
    physical: "/dev/video1",    // Device path (Linux) or index (macOS)
    priority: "physical"         // "physical" for real camera
  },
  video: {
    frame_rate: 30,              // Target frames per second
    resolution: {
      width: 640,                // Frame width in pixels
      height: 480                // Frame height in pixels
    },
    encoding: "rgb8"             // Output encoding (rgb8/rgb)
  }
}
```

### Supported Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `device.physical` | string | "/dev/video1" | Camera device path |
| `device.priority` | string | "physical" | Device priority mode |
| `video.frame_rate` | u16 | 30 | Target frame rate (fps) |
| `video.resolution.width` | u16 | 640 | Frame width |
| `video.resolution.height` | u16 | 480 | Frame height |
| `video.encoding` | string | "rgb8" | Output encoding |

## Published Topics

### `video_stream`
Publishes camera frames with the following message structure:

```rust
{
  header: {
    stamp: SystemTime,    // Capture timestamp
    frame_id: u32         // Sequential frame counter
  },
  encoding: string,       // "rgb8"
  width: u32,             // Frame width
  height: u32,            // Frame height
  data: Vec<u8>           // Raw RGB pixel data
}
```

**QoS Profile**: `sensor_data` (best-effort, volatile)

## Exposed Services

### `video_stream_info`
Query camera capabilities and current configuration.

**Response:**
```rust
{
  width: u32,           // Current frame width
  height: u32,          // Current frame height
  fps: u8,              // Current frame rate
  encoding: string      // Current encoding
}
```

## Troubleshooting

### Camera Not Found
```
Error: Failed to open camera /dev/video1
```

**Solutions:**
1. Check camera is connected: `ls -l /dev/video*`
2. Verify permissions: `groups` should include `video`
3. Try different camera index (e.g., `/dev/video0`)
4. Check camera works: `v4l2-ctl --list-devices` (Linux)

### Permission Denied
```
Error: Permission denied when accessing /dev/video1
```

**Solution:**
```bash
sudo usermod -a -G video $USER
# Log out and back in
```

### Low Frame Rate
If actual FPS is lower than configured:
1. Reduce resolution
2. Ensure good USB connection (use USB 3.0 if available)
3. Check CPU usage
4. Verify camera supports requested resolution/FPS

### Build Errors
If you encounter build errors:
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

## Development

### Project Structure
```
uvc_camera/
├── src/
│   └── main.rs          # Main implementation
├── Cargo.toml           # Rust dependencies
├── peppy.json5          # Peppy node configuration
└── README.md            # This file
```

### Key Dependencies
- `nokhwa` (0.10) - Camera capture library
- `peppygen` - Peppy code generation
- `tokio` - Async runtime
- `tracing` - Logging

### Implementation Notes
- Camera operations run in a blocking task (`spawn_blocking`) because nokhwa's `Camera` type is not `Send`
- Frame emission uses `block_in_place` to call async code from blocking context
- Rate limiting ensures target FPS is maintained

## Example Usage

### Basic Launch
```bash
# Start the node
cargo run --release
```

### Subscribing to Frames
Create a subscriber node to receive camera frames:

```rust
use peppygen::exposed_topics::video_stream;

video_stream::subscribe(&node_runner, |header, encoding, width, height, data| {
    println!("Received frame {}: {}x{} ({})", 
        header.frame_id, width, height, encoding);
    // Process frame data...
});
```

### Querying Camera Info
```rust
use peppygen::exposed_services::video_stream_info;

let response = video_stream_info::call(&node_runner, video_stream_info::Request::new())
    .await?;
    
println!("Camera: {}x{} @ {} fps ({})", 
    response.width, response.height, response.fps, response.encoding);
```

## Known Limitations

- Device path is currently hardcoded to `/dev/video1`
- Only RGB8 encoding is supported
- No runtime camera control adjustment (exposure, white balance, etc.)
- Camera must be connected at node startup
- No automatic reconnection on disconnect

## Future Enhancements

See [plan.md](../plan.md) for planned features:
- Individual camera control services (exposure, white balance, gain, brightness, contrast)
- Graceful shutdown service
- Dynamic device selection
- Additional encoding support (BGR8, MJPEG, YUV420P)
- Camera reconnection handling
- Comprehensive unit and integration tests

## License

[Add your license information here]

## Contributing

[Add contribution guidelines here]
