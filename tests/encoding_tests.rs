use uvc_camera::encoding::Encoding;

#[test]
fn test_encoding_from_str_valid() {
    assert_eq!("rgb8".parse::<Encoding>().unwrap(), Encoding::Rgb8);
    assert_eq!("bgr8".parse::<Encoding>().unwrap(), Encoding::Bgr8);
    assert_eq!("mjpeg".parse::<Encoding>().unwrap(), Encoding::Mjpeg);
}

#[test]
fn test_encoding_from_str_case_insensitive() {
    assert_eq!("RGB8".parse::<Encoding>().unwrap(), Encoding::Rgb8);
    assert_eq!("Rgb8".parse::<Encoding>().unwrap(), Encoding::Rgb8);
    assert_eq!("BGR8".parse::<Encoding>().unwrap(), Encoding::Bgr8);
    assert_eq!("Bgr8".parse::<Encoding>().unwrap(), Encoding::Bgr8);
    assert_eq!("MJPEG".parse::<Encoding>().unwrap(), Encoding::Mjpeg);
    assert_eq!("Mjpeg".parse::<Encoding>().unwrap(), Encoding::Mjpeg);
    assert_eq!("MjPeG".parse::<Encoding>().unwrap(), Encoding::Mjpeg);
}

#[test]
fn test_encoding_from_str_invalid() {
    let result = "invalid".parse::<Encoding>();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Invalid encoding"));
    assert!(error.contains("rgb8, bgr8, mjpeg"));
}

#[test]
fn test_encoding_display() {
    assert_eq!(Encoding::Rgb8.to_string(), "rgb8");
    assert_eq!(Encoding::Bgr8.to_string(), "bgr8");
    assert_eq!(Encoding::Mjpeg.to_string(), "mjpeg");
}

#[test]
fn test_encoding_roundtrip() {
    // Verify that parsing and displaying are consistent
    for encoding in [Encoding::Rgb8, Encoding::Bgr8, Encoding::Mjpeg] {
        let string = encoding.to_string();
        let parsed = string.parse::<Encoding>().unwrap();
        assert_eq!(encoding, parsed);
    }
}

#[test]
fn test_encoding_equality() {
    assert_eq!(Encoding::Rgb8, Encoding::Rgb8);
    assert_ne!(Encoding::Rgb8, Encoding::Bgr8);
    assert_ne!(Encoding::Bgr8, Encoding::Mjpeg);
}

#[test]
fn test_encoding_copy_trait() {
    let enc1 = Encoding::Rgb8;
    let enc2 = enc1; // Should work because Encoding implements Copy
    assert_eq!(enc1, enc2);
    assert_eq!(enc1, Encoding::Rgb8); // enc1 should still be usable
}
