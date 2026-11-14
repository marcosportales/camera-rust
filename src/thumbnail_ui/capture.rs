//! Cache square Captures, decoding Photos and extracting Video frames off the UI thread.

use std::{path::PathBuf, sync::Arc};

use gpui::{App, Asset, RenderImage};
use image::imageops::FilterType;

use crate::theme::size;

/// A cached Capture image; unavailable or malformed files produce the empty Thumbnail.
pub(super) struct SquareCapture;

impl Asset for SquareCapture {
    type Source = PathBuf;
    type Output = Option<Arc<RenderImage>>;

    /// Decode and center-crop a Capture at twice its logical size in GPUI's BGRA format.
    fn load(
        path: PathBuf,
        _: &mut App,
    ) -> impl std::future::Future<Output = Self::Output> + Send + 'static {
        load_pixels(path)
    }
}

/// Load pixels in a future that does not capture the non-Send App reference.
async fn load_pixels(path: PathBuf) -> Option<Arc<RenderImage>> {
    let image = if path.extension().is_some_and(|ext| ext == "mp4") {
        image::DynamicImage::ImageRgba8(crate::video_thumbnail::extract(&path).ok()?)
    } else {
        image::open(path).ok()?
    };
    let side = (size::THUMB * 2.0) as u32;
    let mut pixels = image
        .resize_to_fill(side, side, FilterType::Triangle)
        .into_rgba8();
    for pixel in pixels.pixels_mut() {
        pixel.0.swap(0, 2);
    }
    Some(Arc::new(RenderImage::new([image::Frame::new(pixels)])))
}
