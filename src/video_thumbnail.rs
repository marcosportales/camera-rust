//! Decode a saved Video without touching the live camera or recording pipeline.

use std::path::Path;

use anyhow::{Context, Result, anyhow};
use gstreamer::{self as gst, prelude::*};
use gstreamer_app::AppSink;

/// Extract the first frame of a finalized MP4, bounding the wait for corrupt or unreadable files.
pub fn extract(path: &Path) -> Result<image::RgbaImage> {
    gst::init()?;
    let pipeline = gst::parse::launch(
        "filesrc name=file ! qtdemux ! decodebin ! videoconvert \
         ! video/x-raw,format=RGBA ! appsink name=pixels max-buffers=1",
    )?
    .downcast::<gst::Pipeline>()
    .map_err(|_| anyhow!("Thumbnail decoder is not a pipeline"))?;
    // Always release the decoder and its file, including state changes and decoding failures.
    let result = decode(&pipeline, path);
    let stopped = pipeline.set_state(gst::State::Null);
    stopped?;
    result
}

/// Preroll only one frame; audio and the rest of the Video need not be played.
fn decode(pipeline: &gst::Pipeline, path: &Path) -> Result<image::RgbaImage> {
    pipeline
        .by_name("file")
        .context("Missing Thumbnail file source")?
        .set_property("location", path);
    let sink = pipeline
        .by_name("pixels")
        .context("Missing Thumbnail sink")?
        .downcast::<AppSink>()
        .map_err(|_| anyhow!("Thumbnail sink is not an appsink"))?;
    pipeline.set_state(gst::State::Paused)?;
    let sample = sink
        .try_pull_preroll(gst::ClockTime::from_seconds(5))
        .context("Video did not yield a Thumbnail frame")?;
    let caps = sample.caps().context("Missing frame caps")?;
    let format = caps.structure(0).context("Missing frame format")?;
    let width = u32::try_from(format.get::<i32>("width")?)?;
    let height = u32::try_from(format.get::<i32>("height")?)?;
    let buffer = sample.buffer().context("Missing frame pixels")?;
    let pixels = buffer.map_readable()?;
    // RGBA rows are naturally aligned to GStreamer's four-byte row alignment.
    image::RgbaImage::from_raw(width, height, pixels.to_vec())
        .context("Invalid Thumbnail frame dimensions")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Missing and malformed Videos fail without yielding an image or leaving a running pipeline.
    #[test]
    fn invalid_video_has_no_thumbnail() {
        let folder = tempfile::tempdir().unwrap();
        assert!(extract(&folder.path().join("missing.mp4")).is_err());
        let path = folder.path().join("broken.mp4");
        std::fs::write(&path, b"not an MP4").unwrap();
        assert!(extract(&path).is_err());
    }
}
