use uvc_camera::conversion::{convert_rgb_to_bgr, encode_jpeg};

#[test]
fn test_convert_rgb_to_bgr_basic() {
    // RGB: Red=255, Green=0, Blue=0
    let rgb = vec![255, 0, 0];
    let bgr = convert_rgb_to_bgr(rgb);
    // Should become BGR: Blue=0, Green=0, Red=255
    assert_eq!(bgr, vec![0, 0, 255]);
}

#[test]
fn test_convert_rgb_to_bgr_multiple_pixels() {
    // Two pixels: Red and Blue
    let rgb = vec![
        255, 0, 0,   // Red pixel
        0, 0, 255,   // Blue pixel
    ];
    let bgr = convert_rgb_to_bgr(rgb);
    assert_eq!(bgr, vec![
        0, 0, 255,   // Blue pixel (R and B swapped)
        255, 0, 0,   // Red pixel (R and B swapped)
    ]);
}

#[test]
fn test_convert_rgb_to_bgr_preserves_green() {
    // Green channel should remain unchanged
    let rgb = vec![
        0, 255, 0,   // Pure green
        128, 200, 64, // Mixed color
    ];
    let bgr = convert_rgb_to_bgr(rgb);
    assert_eq!(bgr, vec![
        0, 255, 0,     // Green unchanged, R and B swapped
        64, 200, 128,  // Green unchanged, R and B swapped
    ]);
}

#[test]
fn test_convert_rgb_to_bgr_empty() {
    let rgb: Vec<u8> = vec![];
    let bgr = convert_rgb_to_bgr(rgb);
    assert_eq!(bgr, Vec::<u8>::new());
}

#[test]
fn test_convert_rgb_to_bgr_single_pixel() {
    let rgb = vec![100, 150, 200];
    let bgr = convert_rgb_to_bgr(rgb);
    assert_eq!(bgr, vec![200, 150, 100]);
}

#[test]
fn test_encode_jpeg_valid_small_image() {
    // Create a 2x2 RGB image (12 bytes total: 2*2*3)
    let rgb_data = vec![
        255, 0, 0,     // Red
        0, 255, 0,     // Green
        0, 0, 255,     // Blue
        255, 255, 255, // White
    ];
    
    let result = encode_jpeg(&rgb_data, 2, 2);
    assert!(result.is_ok());
    
    let jpeg_data = result.unwrap();
    // JPEG data should start with JPEG magic bytes (0xFF 0xD8)
    assert_eq!(jpeg_data[0], 0xFF);
    assert_eq!(jpeg_data[1], 0xD8);
    // JPEG data should be non-empty and smaller or similar size for such small images
    assert!(!jpeg_data.is_empty());
}

#[test]
fn test_encode_jpeg_invalid_data_length() {
    // 10x10 image should have 300 bytes (10*10*3), but we only provide 100
    let rgb_data = vec![0u8; 100];
    let result = encode_jpeg(&rgb_data, 10, 10);
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Invalid data length"));
    assert!(error.contains("expected 300 bytes"));
}

#[test]
fn test_encode_jpeg_zero_dimensions() {
    let rgb_data = vec![];
    let result = encode_jpeg(&rgb_data, 0, 0);
    
    // 0x0 image should succeed with empty data
    // The image crate may or may not accept this - let's just verify it handles it
    // Either succeeding with empty output or failing gracefully is acceptable
    match result {
        Err(error) => {
            // If it errors, it should be a reasonable error message
            assert!(!error.is_empty());
        }
        Ok(jpeg_data) => {
            // If it succeeds, JPEG data should at least have magic bytes
            // Empty images might result in minimal JPEG headers
            assert!(!jpeg_data.is_empty());
        }
    }
}

#[test]
fn test_encode_jpeg_width_only() {
    let rgb_data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // 3 pixels = 9 bytes
    let result = encode_jpeg(&rgb_data, 3, 1);
    
    assert!(result.is_ok());
    let jpeg_data = result.unwrap();
    assert_eq!(jpeg_data[0], 0xFF);
    assert_eq!(jpeg_data[1], 0xD8);
}

#[test]
fn test_encode_jpeg_height_only() {
    let rgb_data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // 3 pixels = 9 bytes
    let result = encode_jpeg(&rgb_data, 1, 3);
    
    assert!(result.is_ok());
    let jpeg_data = result.unwrap();
    assert_eq!(jpeg_data[0], 0xFF);
    assert_eq!(jpeg_data[1], 0xD8);
}

#[test]
fn test_encode_jpeg_overflow_protection() {
    // Test that extremely large dimensions are handled gracefully
    let rgb_data = vec![0u8; 10];
    let result = encode_jpeg(&rgb_data, u32::MAX, u32::MAX);
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Image dimensions too large"));
}

#[test]
fn test_encode_jpeg_mismatched_dimensions() {
    // Provide correct amount of data but wrong dimensions
    let rgb_data = vec![0u8; 12]; // 4 pixels worth
    let result = encode_jpeg(&rgb_data, 2, 1); // Claims to be 2 pixels
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Invalid data length"));
}
