//! Cache square Photos so GPUI rounds the visible Thumbnail rather than an oversized Cover image.

use std::{path::PathBuf, sync::Arc};

use gpui::{App, Asset, ImageCacheError, ImageSource, RenderImage};
use image::imageops::FilterType;

use crate::theme::size;

/// Load and center-crop each Photo once, keeping decoding off the UI thread.
struct SquarePhoto;

impl Asset for SquarePhoto {
    type Source = PathBuf;
    type Output = Result<Arc<RenderImage>, ImageCacheError>;

    /// Decode a square Thumbnail at twice its logical size, in GPUI's BGRA pixel format.
    fn load(
        path: PathBuf,
        _: &mut App,
    ) -> impl std::future::Future<Output = Self::Output> + Send + 'static {
        let side = (size::THUMB * 2.0) as u32;
        // Keep the non-Send App reference out of the background loader future.
        async move {
            let mut pixels = image::open(path)?
                .resize_to_fill(side, side, FilterType::Triangle)
                .into_rgba8();
            for pixel in pixels.pixels_mut() {
                pixel.0.swap(0, 2);
            }
            Ok(Arc::new(RenderImage::new([image::Frame::new(pixels)])))
        }
    }
}

/// Supply a cached square Photo and redraw the window once loading completes.
pub(super) fn source(path: PathBuf) -> ImageSource {
    ImageSource::from(move |window: &mut gpui::Window, cx: &mut App| {
        window.use_asset::<SquarePhoto>(&path, cx)
    })
}
