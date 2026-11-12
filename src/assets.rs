use std::borrow::Cow;

use anyhow::Result;
use gpui::{App, AssetSource, SharedString};

/// Embeds files from the `assets/` folder into the binary, keyed by their path relative to it.
macro_rules! embed {
    ($($path:literal),* $(,)?) => {
        &[$(($path, include_bytes!(concat!("../assets/", $path)) as &[u8])),*]
    };
}

const FONTS: &[(&str, &[u8])] = embed![
    "fonts/inter-300.ttf",
    "fonts/inter-400.ttf",
    "fonts/inter-500.ttf",
    "fonts/inter-600.ttf",
];

const ICONS: &[(&str, &[u8])] = embed![
    "icons/timer.svg",
    "icons/flip-horizontal.svg",
    "icons/camera.svg",
    "icons/camera-off.svg",
    "icons/check.svg",
    "icons/x.svg",
    "icons/rotate-cw.svg",
    "icons/play.svg",
    "icons/folder.svg",
    "icons/chevron-down.svg",
];

/// GPUI asset source serving the files embedded in the binary.
pub struct Assets;

impl Assets {
    /// Register the bundled Inter weights so `font::FAMILY` resolves to them.
    pub fn register_fonts(cx: &App) {
        let fonts = FONTS.iter().map(|(_, data)| Cow::Borrowed(*data)).collect();
        // The fonts are compile-time constants, so a parse failure is a build defect, not a runtime condition.
        cx.text_system()
            .add_fonts(fonts)
            .expect("bundled Inter fonts are valid TTF files");
    }
}

/// Iterate over every embedded file as `(path, data)`.
fn all_files() -> impl Iterator<Item = &'static (&'static str, &'static [u8])> {
    ICONS.iter().chain(FONTS)
}

impl AssetSource for Assets {
    /// Return the embedded file at `path`, or `None` when it is not bundled.
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let file = all_files().find(|(name, _)| *name == path);
        Ok(file.map(|(_, data)| Cow::Borrowed(*data)))
    }

    /// List the embedded files whose path starts with `path`.
    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(all_files()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| SharedString::from(*name))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQUIRED_ICONS: [&str; 10] = [
        "timer",
        "flip-horizontal",
        "camera",
        "camera-off",
        "check",
        "x",
        "rotate-cw",
        "play",
        "folder",
        "chevron-down",
    ];

    /// Every icon the design needs is bundled and looks like SVG.
    #[test]
    fn every_required_icon_loads_as_non_empty_svg() {
        for name in REQUIRED_ICONS {
            let data = Assets
                .load(&format!("icons/{name}.svg"))
                .unwrap()
                .unwrap_or_else(|| panic!("icon `{name}` is not bundled"));
            let text = std::str::from_utf8(&data).unwrap();
            assert!(text.contains("<svg"), "icon `{name}` is not SVG data");
        }
    }

    /// A path that is not bundled yields `None` rather than an error.
    #[test]
    fn unknown_asset_loads_as_none() {
        assert!(Assets.load("icons/missing.svg").unwrap().is_none());
    }
}
