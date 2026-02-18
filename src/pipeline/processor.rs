use crate::types::{Encoding, Frame, FrameId, Result};
use super::converter;

/// JPEG encoding quality (1-100)
const JPEG_QUALITY: u8 = 85;

/// Process a raw frame from the camera into the target encoding
pub fn process_frame(
    frame: Frame,
    frame_id: FrameId,
    target_encoding: Encoding,
) -> Result<Frame> {
    // Source encoding is always RGB8 from camera
    let source_encoding = Encoding::Rgb8;
    
    // Convert if needed
    let data = converter::convert_frame(
        frame.data(),
        frame.width(),
        frame.height(),
        source_encoding,
        target_encoding,
        JPEG_QUALITY,
    )?;
    
    Ok(frame.with_encoding(data, target_encoding).with_frame_id(frame_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_process_frame_rgb8() {
        let data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // 3 pixels
        let raw = Frame::from_capture(data.clone(), 3, 1, Instant::now());
        let frame = process_frame(raw, FrameId::default(), Encoding::Rgb8).unwrap();
        
        assert_eq!(frame.data(), &data);
        assert_eq!(frame.width(), 3);
        assert_eq!(frame.height(), 1);
        assert_eq!(frame.encoding(), Encoding::Rgb8);
    }
    
    #[test]
    fn test_process_frame_bgr8() {
        let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        let raw = Frame::from_capture(rgb, 3, 1, Instant::now());
        let frame = process_frame(raw, FrameId::default(), Encoding::Bgr8).unwrap();
        
        assert_eq!(frame.data(), &[0, 0, 255, 0, 255, 0, 255, 0, 0]);
        assert_eq!(frame.encoding(), Encoding::Bgr8);
    }
    
    #[test]
    fn test_process_frame_mjpeg() {
        let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        let raw = Frame::from_capture(rgb, 3, 1, Instant::now());
        let frame = process_frame(raw, FrameId::default(), Encoding::Mjpeg).unwrap();
        
        // Check JPEG header
        assert!(frame.data().starts_with(&[0xFF, 0xD8]));
        assert_eq!(frame.encoding(), Encoding::Mjpeg);
    }
}
