/// Pixel size at which the camera delivers the Viewfinder and Captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    /// Whether this Resolution is 16:9, within a 1% tolerance.
    pub fn is_16_9(self) -> bool {
        // `width * 9` vs `height * 16`, compared in integers: |a - b| <= 1% of b.
        let (a, b) = (u64::from(self.width) * 9, u64::from(self.height) * 16);
        a.abs_diff(b) * 100 <= b
    }

    /// Total pixel count, used to rank Resolutions.
    pub fn pixels(self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }
}

/// Pick the default Resolution: the biggest 16:9 one, else the biggest overall.
pub fn default_resolution(resolutions: &[Resolution]) -> Option<Resolution> {
    let biggest_16_9 = resolutions
        .iter()
        .copied()
        .filter(|r| r.is_16_9())
        .max_by_key(|r| r.pixels());
    biggest_16_9.or_else(|| resolutions.iter().copied().max_by_key(|r| r.pixels()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shorthand for building a Resolution in tests.
    fn res(width: u32, height: u32) -> Resolution {
        Resolution { width, height }
    }

    #[test]
    fn common_16_9_sizes_are_16_9() {
        for (w, h) in [(1280, 720), (960, 540), (848, 480), (640, 360)] {
            assert!(res(w, h).is_16_9(), "{w}x{h}");
        }
    }

    #[test]
    fn four_by_three_is_not_16_9() {
        assert!(!res(640, 480).is_16_9());
    }

    #[test]
    fn default_is_the_biggest_16_9_when_there_are_several() {
        let list = [
            res(640, 360),
            res(1920, 1080),
            res(1280, 720),
            res(2560, 1920),
        ];
        assert_eq!(default_resolution(&list), Some(res(1920, 1080)));
    }

    #[test]
    fn default_is_the_biggest_overall_when_none_is_16_9() {
        let list = [res(640, 480), res(1600, 1200), res(800, 600)];
        assert_eq!(default_resolution(&list), Some(res(1600, 1200)));
    }

    #[test]
    fn default_of_an_empty_list_is_none() {
        assert_eq!(default_resolution(&[]), None);
    }
}
