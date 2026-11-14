//! Convert camera frames for display without changing the Capture source.

use gpui::RenderImage;

use crate::gst_camera::Frame;

/// Wrap a BGRA frame for GPUI, mirroring only the displayed pixels when requested.
pub fn render_image(frame: Frame, mirror: bool) -> Option<RenderImage> {
    let mut buffer = image::RgbaImage::from_raw(frame.size.width, frame.size.height, frame.bgra)?;
    if mirror {
        image::imageops::flip_horizontal_in_place(&mut buffer);
    }
    Some(RenderImage::new([image::Frame::new(buffer)]))
}

/// Reverse the displayed frame immediately, including when the camera has stopped sending frames.
pub fn flipped_image(image: &RenderImage) -> Option<RenderImage> {
    let size = image.size(0);
    render_image(
        Frame {
            size: crate::app_core::resolution::Resolution {
                width: size.width.0.try_into().ok()?,
                height: size.height.0.try_into().ok()?,
            },
            bgra: image.as_bytes(0)?.to_vec(),
        },
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_core::resolution::Resolution;

    /// Mirror reverses columns on each row while preserving BGRA channels and row order.
    #[test]
    fn mirror_reverses_only_viewfinder_columns() {
        let pixels = vec![1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255];
        for (mirror, expected) in [
            (false, pixels.clone()),
            (
                true,
                vec![4, 5, 6, 255, 1, 2, 3, 255, 10, 11, 12, 255, 7, 8, 9, 255],
            ),
        ] {
            let frame = Frame {
                size: Resolution {
                    width: 2,
                    height: 2,
                },
                bgra: pixels.clone(),
            };
            let image = render_image(frame, mirror).unwrap();
            assert_eq!(image.as_bytes(0).unwrap(), expected);
            let flipped = flipped_image(&image).unwrap();
            let restored = flipped_image(&flipped).unwrap();
            assert_eq!(restored.as_bytes(0), image.as_bytes(0));
            if mirror {
                assert_eq!(flipped.as_bytes(0).unwrap(), pixels);
            }
        }
    }
}
