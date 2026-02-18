use nokhwa::utils::CameraIndex;
use uvc_camera::old_camera::parse_camera_index;

#[test]
fn test_parse_camera_index_dev_video() {
    match parse_camera_index("/dev/video0") {
        CameraIndex::Index(i) => assert_eq!(i, 0),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
    
    match parse_camera_index("/dev/video1") {
        CameraIndex::Index(i) => assert_eq!(i, 1),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
    
    match parse_camera_index("/dev/video42") {
        CameraIndex::Index(i) => assert_eq!(i, 42),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
}

#[test]
fn test_parse_camera_index_numeric_string() {
    match parse_camera_index("0") {
        CameraIndex::Index(i) => assert_eq!(i, 0),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
    
    match parse_camera_index("1") {
        CameraIndex::Index(i) => assert_eq!(i, 1),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
    
    match parse_camera_index("99") {
        CameraIndex::Index(i) => assert_eq!(i, 99),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
}

#[test]
fn test_parse_camera_index_video_prefix_variations() {
    // Test with different paths containing "video"
    match parse_camera_index("/custom/path/video5") {
        CameraIndex::Index(i) => assert_eq!(i, 5),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
    
    match parse_camera_index("video2") {
        CameraIndex::Index(i) => assert_eq!(i, 2),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
}

#[test]
fn test_parse_camera_index_invalid_defaults_to_zero() {
    // Invalid inputs should default to index 0
    match parse_camera_index("invalid") {
        CameraIndex::Index(i) => assert_eq!(i, 0),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
    
    match parse_camera_index("/dev/camera0") {
        CameraIndex::Index(i) => assert_eq!(i, 0),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
    
    match parse_camera_index("") {
        CameraIndex::Index(i) => assert_eq!(i, 0),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
    
    match parse_camera_index("abc123") {
        CameraIndex::Index(i) => assert_eq!(i, 0),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
}

#[test]
fn test_parse_camera_index_video_without_number() {
    // "video" without a number should default to 0
    match parse_camera_index("/dev/video") {
        CameraIndex::Index(i) => assert_eq!(i, 0),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
    
    match parse_camera_index("video") {
        CameraIndex::Index(i) => assert_eq!(i, 0),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
}

#[test]
fn test_parse_camera_index_large_numbers() {
    match parse_camera_index("/dev/video1000") {
        CameraIndex::Index(i) => assert_eq!(i, 1000),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
    
    match parse_camera_index("12345") {
        CameraIndex::Index(i) => assert_eq!(i, 12345),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
}

#[test]
fn test_parse_camera_index_multiple_video_occurrences() {
    // Should use the last occurrence of "video"
    match parse_camera_index("/video1/video2/video3") {
        CameraIndex::Index(i) => assert_eq!(i, 3),
        CameraIndex::String(_) => panic!("Expected CameraIndex::Index"),
    }
}
