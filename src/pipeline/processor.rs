use crate::types::{Encoding, Error, Frame, FrameId, Result};

/// JPEG encoding quality (1-100)
const JPEG_QUALITY: u8 = 85;

/// Convert RGB8 to BGR8 by swapping R and B channels
fn rgb_to_bgr(data: &[u8]) -> Vec<u8> {
    data.chunks_exact(3)
        .flat_map(|rgb| [rgb[2], rgb[1], rgb[0]])
        .collect()
}

/// Convert BGR8 to RGB8 by swapping B and R channels
fn bgr_to_rgb(data: &[u8]) -> Vec<u8> {
    // Same operation as rgb_to_bgr
    rgb_to_bgr(data)
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

/// Convert frame data to target encoding
fn convert_frame(data: &[u8], width: u32, height: u32, source: Encoding, target: Encoding, jpeg_quality: u8) -> Result<Vec<u8>> {
    match (source, target) {
        // No conversion needed
        (Encoding::Rgb8, Encoding::Rgb8) => Ok(data.to_vec()),
        (Encoding::Bgr8, Encoding::Bgr8) => Ok(data.to_vec()),
        (Encoding::Mjpeg, Encoding::Mjpeg) => Ok(data.to_vec()),
        
        // RGB <-> BGR conversion
        (Encoding::Rgb8, Encoding::Bgr8) => Ok(rgb_to_bgr(data)),
        (Encoding::Bgr8, Encoding::Rgb8) => Ok(bgr_to_rgb(data)),
        
        // RGB -> MJPEG
        (Encoding::Rgb8, Encoding::Mjpeg) => encode_jpeg(data, width, height, jpeg_quality),
        
        // BGR -> MJPEG (convert to RGB first)
        (Encoding::Bgr8, Encoding::Mjpeg) => {
            let rgb = bgr_to_rgb(data);
            encode_jpeg(&rgb, width, height, jpeg_quality)
        }
        
        // Unsupported conversions from MJPEG
        (Encoding::Mjpeg, Encoding::Rgb8) | (Encoding::Mjpeg, Encoding::Bgr8) => {
            Err(Error::EncodingError(
                "Decoding MJPEG is not supported".to_string(),
            ))
        }
    }
}

/// Process a raw frame from the camera into the target encoding
pub fn process_frame(
    frame: Frame,
    frame_id: FrameId,
    target_encoding: Encoding,
) -> Result<Frame> {
    // Source encoding is always RGB8 from camera
    let source_encoding = Encoding::Rgb8;
    
    // Convert if needed
    let data = convert_frame(
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
    
    #[test]
    fn test_rgb_to_bgr() {
        let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        let bgr = rgb_to_bgr(&rgb);
        assert_eq!(bgr, vec![0, 0, 255, 0, 255, 0, 255, 0, 0]);
    }
    
    #[test]
    fn test_bgr_to_rgb() {
        let bgr = vec![0, 0, 255, 0, 255, 0, 255, 0, 0];
        let rgb = bgr_to_rgb(&bgr);
        assert_eq!(rgb, vec![255, 0, 0, 0, 255, 0, 0, 0, 255]);
    }
    
    #[test]
    fn test_convert_frame_no_conversion() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let result = convert_frame(&data, 2, 1, Encoding::Rgb8, Encoding::Rgb8, 85).unwrap();
        assert_eq!(result, data);
    }
    
    #[test]
    fn test_convert_frame_rgb_to_bgr() {
        let rgb = vec![255, 0, 0, 0, 255, 0];
        let result = convert_frame(&rgb, 2, 1, Encoding::Rgb8, Encoding::Bgr8, 85).unwrap();
        assert_eq!(result, vec![0, 0, 255, 0, 255, 0]);
    }
    
    #[test]
    fn test_mjpeg_decode_unsupported() {
        let data = vec![0xFF, 0xD8]; // JPEG header
        let result = convert_frame(&data, 1, 1, Encoding::Mjpeg, Encoding::Rgb8, 85);
        assert!(result.is_err());
    }
}
