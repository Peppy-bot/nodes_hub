mod helpers;

use helpers::virtual_camera::VirtualCamera;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution};
use nokhwa::Camera;
use std::time::Duration;

/// Integration test: Verify we can open and capture from a virtual camera
#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_open_virtual_camera() {
    let vcam = match VirtualCamera::new(10, 640, 480, 30) {
        Ok(cam) => cam,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };
    
    let camera_index = CameraIndex::Index(10);
    let requested_format = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::AbsoluteHighestResolution
    );
    
    let camera = Camera::new(camera_index, requested_format);
    assert!(camera.is_ok(), "Should successfully open virtual camera");
    
    drop(vcam); // Cleanup
}

/// Integration test: Capture frames from virtual camera
#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_capture_frames_from_virtual_camera() {
    let vcam = match VirtualCamera::new(10, 640, 480, 30) {
        Ok(cam) => cam,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };
    
    let camera_index = CameraIndex::Index(10);
    let requested_format = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::AbsoluteHighestResolution
    );
    
    let mut camera = Camera::new(camera_index, requested_format)
        .expect("Failed to open camera");
    
    // Set resolution
    camera.set_resolution(Resolution::new(640, 480))
        .expect("Failed to set resolution");
    
    // Open stream
    camera.open_stream()
        .expect("Failed to open stream");
    
    // Capture a few frames
    for i in 0..5 {
        let frame = camera.frame();
        assert!(frame.is_ok(), "Frame {} should be captured successfully", i);
        
        if let Ok(frame) = frame {
            let rgb_frame = frame.decode_image::<RgbFormat>();
            assert!(rgb_frame.is_ok(), "Frame {} should decode to RGB", i);
            
            if let Ok(img) = rgb_frame {
                let data = img.into_raw();
                // 640 * 480 * 3 bytes (RGB)
                assert_eq!(data.len(), 640 * 480 * 3, "Frame {} should have correct size", i);
            }
        }
        
        std::thread::sleep(Duration::from_millis(100));
    }
    
    drop(vcam); // Cleanup
}

/// Integration test: Verify different resolutions work
#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_different_resolutions() {
    let resolutions = vec![
        (320, 240),
        (640, 480),
        (1280, 720),
    ];
    
    for (width, height) in resolutions {
        let vcam = match VirtualCamera::new(10, width, height, 30) {
            Ok(cam) => cam,
            Err(e) => {
                eprintln!("Skipping test for {}x{}: {}", width, height, e);
                continue;
            }
        };
        
        let camera_index = CameraIndex::Index(10);
        let requested_format = RequestedFormat::new::<RgbFormat>(
            RequestedFormatType::AbsoluteHighestResolution
        );
        
        let mut camera = Camera::new(camera_index, requested_format)
            .expect("Failed to open camera");
        
        camera.set_resolution(Resolution::new(width, height))
            .unwrap_or_else(|_| panic!("Failed to set resolution {}x{}", width, height));
        
        camera.open_stream()
            .expect("Failed to open stream");
        
        let frame = camera.frame();
        assert!(frame.is_ok(), "Should capture frame at {}x{}", width, height);
        
        if let Ok(frame) = frame {
            let rgb_frame = frame.decode_image::<RgbFormat>();
            assert!(rgb_frame.is_ok(), "Should decode frame at {}x{}", width, height);
        }
        
        drop(vcam); // Cleanup before next iteration
        std::thread::sleep(Duration::from_millis(500)); // Give time for device to be released
    }
}

/// Integration test: Test with color bars pattern
#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_capture_color_bars() {
    let vcam = match VirtualCamera::new_with_color_bars(10, 640, 480, 30) {
        Ok(cam) => cam,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };
    
    let camera_index = CameraIndex::Index(10);
    let requested_format = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::AbsoluteHighestResolution
    );
    
    let mut camera = Camera::new(camera_index, requested_format)
        .expect("Failed to open camera");
    
    camera.set_resolution(Resolution::new(640, 480))
        .expect("Failed to set resolution");
    
    camera.open_stream()
        .expect("Failed to open stream");
    
    // Capture frame with color bars
    let frame = camera.frame()
        .expect("Should capture color bars frame");
    
    let rgb_frame = frame.decode_image::<RgbFormat>()
        .expect("Should decode color bars");
    
    let data = rgb_frame.into_raw();
    assert_eq!(data.len(), 640 * 480 * 3);
    
    // Color bars should have non-zero, varied pixel values
    let mut has_variation = false;
    let first_pixel = [data[0], data[1], data[2]];
    for chunk in data.chunks(3).skip(100) {
        if chunk[0] != first_pixel[0] || chunk[1] != first_pixel[1] || chunk[2] != first_pixel[2] {
            has_variation = true;
            break;
        }
    }
    assert!(has_variation, "Color bars should have pixel variation");
    
    drop(vcam);
}

/// Integration test: Parse device path and capture
#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_parse_device_and_capture() {
    use uvc_camera::camera::parse_camera_index;
    
    let vcam = match VirtualCamera::new(10, 640, 480, 30) {
        Ok(cam) => cam,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };
    
    // Test that our parsing function works with the actual device
    let camera_index = parse_camera_index("/dev/video10");
    
    let requested_format = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::AbsoluteHighestResolution
    );
    
    let mut camera = Camera::new(camera_index, requested_format)
        .expect("Failed to open camera with parsed index");
    
    camera.open_stream()
        .expect("Failed to open stream");
    
    let frame = camera.frame();
    assert!(frame.is_ok(), "Should capture frame using parsed device index");
    
    drop(vcam);
}

/// Integration test: Test frame rate
#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_frame_rate_timing() {
    let vcam = match VirtualCamera::new(10, 640, 480, 30) {
        Ok(cam) => cam,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };
    
    let camera_index = CameraIndex::Index(10);
    let requested_format = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::AbsoluteHighestResolution
    );
    
    let mut camera = Camera::new(camera_index, requested_format)
        .expect("Failed to open camera");
    
    camera.set_resolution(Resolution::new(640, 480))
        .expect("Failed to set resolution");
    
    camera.set_frame_rate(30)
        .expect("Failed to set frame rate");
    
    camera.open_stream()
        .expect("Failed to open stream");
    
    // Capture 30 frames and measure time
    let start = std::time::Instant::now();
    let mut captured = 0;
    
    for _ in 0..30 {
        if camera.frame().is_ok() {
            captured += 1;
        }
        std::thread::sleep(Duration::from_millis(33)); // ~30 fps
    }
    
    let elapsed = start.elapsed();
    
    assert!(captured >= 25, "Should capture at least 25 of 30 frames");
    assert!(elapsed.as_secs() <= 2, 
            "Should take roughly 1 second to capture 30 frames at 30fps");
    
    drop(vcam);
}
