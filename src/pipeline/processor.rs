use crate::types::{Encoding, Error, Frame, FrameId, Result};

/// JPEG encoding quality (1-100)
const JPEG_QUALITY: u8 = 85;

/// Convert RGB8 to BGR8 by swapping R and B channels
fn rgb_to_bgr(data: &[u8]) -> Vec<u8> {
    data.chunks_exact(3)
        .flat_map(|rgb| [rgb[2], rgb[1], rgb[0]])
        .collect()
}

/// Encode RGB8 data as JPEG
fn encode_jpeg(data: &[u8], width: u32, height: u32, _quality: u8) -> Result<Vec<u8>> {
    use image::{ImageBuffer, Rgb};
    
    let img = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, data.to_vec())
        .ok_or_else(|| Error::EncodingError("Failed to create image buffer".to_string()))?;
    
    let mut jpeg_data = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut jpeg_data),
        image::ImageFormat::Jpeg,
    )
    .map_err(|e| Error::EncodingError(format!("JPEG encoding failed: {}", e)))?;
    
    Ok(jpeg_data)
}

/// Process a raw frame from the camera into the target encoding
pub fn process_frame(
    frame: Frame,
    frame_id: FrameId,
    target_encoding: Encoding,
) -> Result<Frame> {
    // Convert RGB8 from camera to target encoding
    let data = match target_encoding {
        // No conversion needed
        Encoding::Rgb8 => frame.data().to_vec(),
        
        // RGB -> BGR conversion
        Encoding::Bgr8 => rgb_to_bgr(frame.data()),
        
        // RGB -> MJPEG
        Encoding::Mjpeg => encode_jpeg(frame.data(), frame.width(), frame.height(), JPEG_QUALITY)?,
    };
    
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
    
    #[test]
    fn test_rgb_to_bgr() {
        let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        let bgr = rgb_to_bgr(&rgb);
        assert_eq!(bgr, vec![0, 0, 255, 0, 255, 0, 255, 0, 0]);
        
        // Verify the operation is reversible (BGR to RGB is the same)
        let rgb_again = rgb_to_bgr(&bgr);
        assert_eq!(rgb_again, rgb);
    }
}
