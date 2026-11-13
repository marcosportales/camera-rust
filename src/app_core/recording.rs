/// Where the Video being captured stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCapture {
    Recording(Recording),
    /// Stopped; its file is still being finalized.
    Saving,
}

/// A Video being recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recording {
    /// Whole seconds since the recording started.
    pub elapsed_secs: u64,
}

impl Recording {
    /// The elapsed time as `MM:SS`; minutes keep counting past 59 (`61:05`).
    pub fn timecode(self) -> String {
        format!(
            "{:02}:{:02}",
            self.elapsed_secs / 60,
            self.elapsed_secs % 60
        )
    }
}
