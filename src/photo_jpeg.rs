//! JPEG transformation for saved Photos; the camera's source frame stays untouched.

/// Decode a camera JPEG and encode its horizontally mirrored pixels at high quality.
pub fn mirrored(jpeg: &[u8]) -> image::ImageResult<Vec<u8>> {
    let mut pixels = image::load_from_memory_with_format(jpeg, image::ImageFormat::Jpeg)?.to_rgb8();
    image::imageops::flip_horizontal_in_place(&mut pixels);
    let mut encoded = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut encoded, 95).encode_image(&pixels)?;
    Ok(encoded)
}
