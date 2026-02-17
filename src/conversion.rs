/// JPEG quality for compression
const JPEG_QUALITY: u8 = 85;

/// Convert RGB data to BGR by swapping R and B channels
pub fn convert_rgb_to_bgr(mut rgb_data: Vec<u8>) -> Vec<u8> {
    for chunk in rgb_data.chunks_exact_mut(3) {
        chunk.swap(0, 2);
    }
    rgb_data
}

/// Encode RGB data as JPEG
pub fn encode_jpeg(rgb_data: &[u8], width: u32, height: u32) -> std::result::Result<Vec<u8>, String> {
    // Validate data length
    let expected_len = (width as usize)
        .checked_mul(height as usize)
        .and_then(|size| size.checked_mul(3))
        .ok_or_else(|| "Image dimensions too large".to_string())?;

    if rgb_data.len() != expected_len {
        return Err(format!(
            "Invalid data length: expected {} bytes for {}x{} RGB image, got {}",
            expected_len, width, height, rgb_data.len()
        ));
    }

    // Encode as JPEG
    let mut jpeg_data = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_data, JPEG_QUALITY)
        .encode(
            rgb_data,
            width,
            height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("JPEG encoding failed: {}", e))?;

    Ok(jpeg_data)
}
