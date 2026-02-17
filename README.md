# UVC Camera Node

A basic USB Video Class (UVC) camera node for the Peppy framework that captures real camera streams using the `nokhwa` library.

## Overview

This node captures video frames from a UVC-compatible camera and publishes them on the `video_stream` topic. It supports multiple encoding formats (RGB8, BGR8, MJPEG) and provides a simple interface for integrating USB cameras into Peppy-based robotics systems.

## Features

- **Real-time video capture** from UVC cameras
- **Multiple encoding formats**: RGB8, BGR8, and MJPEG
- **Configurable resolution and frame rate**
- **Camera info service** for querying capabilities
- **Graceful shutdown** handling
- **Cross-platform** support (Linux, macOS)

## Requirements

### Hardware
- UVC-compatible USB camera
- Camera device path configurable via parameters (e.g., `/dev/video0`, `/dev/video1`)

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
  device: "/dev/video0",      // Device path (e.g., "/dev/video0", "/dev/video1") or index ("0", "1")
  video: {
    frame_rate: 30,           // Target frames per second
    resolution: {
      width: 640,             // Frame width in pixels
      height: 480             // Frame height in pixels
    },
    encoding: "rgb8"          // Output encoding: "rgb8", "bgr8", or "mjpeg"
  }
}
```

### Supported Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `device` | string | "/dev/video0" | Camera device path or index (e.g., "/dev/video0", "0") |
| `video.frame_rate` | u16 | 30 | Target frame rate (fps) |
| `video.resolution.width` | u16 | 640 | Frame width |
| `video.resolution.height` | u16 | 480 | Frame height |
| `video.encoding` | string | "rgb8" | Output encoding: "rgb8", "bgr8", or "mjpeg" |

## Published Topics

### `video_stream`
Publishes camera frames with the following message structure:

```rust
{
  header: {
    stamp: SystemTime,    // Capture timestamp
    frame_id: u32         // Sequential frame counter
  },
  encoding: string,       // "rgb8", "bgr8", or "mjpeg"
  width: u32,             // Frame width
  height: u32,            // Frame height
  frame: Vec<u8>           // Raw pixel data (format depends on encoding)
}
```

**Encoding Formats:**
- `rgb8`: 24-bit RGB (3 bytes per pixel: R, G, B)
- `bgr8`: 24-bit BGR (3 bytes per pixel: B, G, R)
- `mjpeg`: JPEG-compressed image data (variable size)

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
Error: Failed to open camera /dev/video0
```

**Solutions:**
1. Check camera is connected: `ls -l /dev/video*`
2. Verify permissions: `groups` should include `video`
3. Try different camera index in configuration (e.g., "/dev/video1")
4. Check camera works: `v4l2-ctl --list-devices` (Linux)

### Permission Denied
```
Error: Permission denied when accessing /dev/videoX
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
│   ├── main.rs          # Entry point and configuration
│   ├── encoding.rs      # Encoding enum (Rgb8, Bgr8, Mjpeg)
│   ├── conversion.rs    # Format conversion functions
│   ├── camera.rs        # Camera operations and capture loop
│   └── services.rs      # Service handlers
├── Cargo.toml           # Rust dependencies
├── peppy.json5          # Peppy node configuration
└── README.md            # This file
```

### Key Dependencies
- `nokhwa` (0.10) - Camera capture library
- `peppygen` - Peppy code generation
- `tokio` - Async runtime
- `image` - JPEG encoding for MJPEG format
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

- No runtime camera control adjustment (exposure, white balance, etc.)
- Camera must be connected at node startup
- No automatic reconnection on disconnect

## Future Enhancements

See [plan.md](../plan.md) for planned features:
- Individual camera control services (exposure, white balance, gain, brightness, contrast)
- Graceful shutdown service
- Camera reconnection handling
- Comprehensive unit and integration tests
- Configurable JPEG quality parameter

## License

[Add your license information here]

## Contributing

[Add contribution guidelines here]
