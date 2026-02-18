use crate::config;
use crate::types::{Encoding, Frame, FrameId, RawFrame, Result};
use super::converter;

/// Process a raw frame from the camera into the target encoding
pub fn process_frame(
    raw_frame: RawFrame,
    frame_id: FrameId,
    target_encoding: Encoding,
) -> Result<Frame> {
    // Source encoding is always RGB8 from nokhwa
    let source_encoding = Encoding::Rgb8;
    
    // Convert if needed
    let data = converter::convert_frame(
        raw_frame.data(),
        raw_frame.width(),
        raw_frame.height(),
        source_encoding,
        target_encoding,
        config::jpeg::QUALITY,
    )?;
    
    Ok(Frame::new(
        data,
        raw_frame.width(),
        raw_frame.height(),
        frame_id,
        raw_frame.timestamp(),
        target_encoding,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_process_frame_rgb8() {
        let data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // 3 pixels
        let raw = RawFrame::new(data.clone(), 3, 1, Instant::now());
        let frame = process_frame(raw, FrameId::default(), Encoding::Rgb8).unwrap();
        
        assert_eq!(frame.data(), &data);
        assert_eq!(frame.width(), 3);
        assert_eq!(frame.height(), 1);
        assert_eq!(frame.encoding(), Encoding::Rgb8);
    }
    
    #[test]
    fn test_process_frame_bgr8() {
        let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        let raw = RawFrame::new(rgb, 3, 1, Instant::now());
        let frame = process_frame(raw, FrameId::default(), Encoding::Bgr8).unwrap();
        
        assert_eq!(frame.data(), &[0, 0, 255, 0, 255, 0, 255, 0, 0]);
        assert_eq!(frame.encoding(), Encoding::Bgr8);
    }
    
    #[test]
    fn test_process_frame_mjpeg() {
        let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        let raw = RawFrame::new(rgb, 3, 1, Instant::now());
        let frame = process_frame(raw, FrameId::default(), Encoding::Mjpeg).unwrap();
        
        // Check JPEG header
        assert!(frame.data().starts_with(&[0xFF, 0xD8]));
        assert_eq!(frame.encoding(), Encoding::Mjpeg);
    }
}
