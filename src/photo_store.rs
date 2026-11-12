//! Where Photos live on disk and how they are named.

use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};

/// The folder Photos are saved in: the XDG Pictures directory plus `Camera`.
pub fn photos_folder() -> PathBuf {
    let pictures = dirs::picture_dir()
        .or_else(|| dirs::home_dir().map(|home| home.join("Pictures")))
        .unwrap_or_else(|| PathBuf::from("Pictures"));
    pictures.join("Camera")
}

/// Write `jpeg` to a new `YYYY-MM-DD_HH-MM-SS.jpg` in `folder`, creating the folder if missing.
///
/// A name that is taken gets `_1`, `_2`, … before the extension; an existing file is never overwritten.
pub fn save_jpeg(folder: &Path, jpeg: &[u8], now: DateTime<Local>) -> io::Result<PathBuf> {
    fs::create_dir_all(folder)?;
    let stamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let mut attempt = 0u32;
    loop {
        let name = match attempt {
            0 => format!("{stamp}.jpg"),
            n => format!("{stamp}_{n}.jpg"),
        };
        attempt += 1;
        let path = folder.join(name);
        // `create_new` fails atomically if the file exists, so two Photos in one second cannot collide.
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return write_all_or_remove(file, &path, jpeg).map(|()| path),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
}

/// Write and flush `bytes`; on failure remove the partial file at `path`.
fn write_all_or_remove(mut file: fs::File, path: &Path, bytes: &[u8]) -> io::Result<()> {
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    if written.is_err() {
        let _ = fs::remove_file(path);
    }
    written
}

/// `folder` relative to the home directory (`Pictures/Camera`), or as is when it is elsewhere.
pub fn display_folder(folder: &Path) -> String {
    let relative =
        dirs::home_dir().and_then(|home| folder.strip_prefix(home).ok().map(Path::to_path_buf));
    relative.as_deref().unwrap_or(folder).display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    /// 2026-11-12 09:05:07 local time.
    fn sample_time() -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 11, 12, 9, 5, 7).unwrap()
    }

    #[test]
    fn creates_the_folder_and_names_the_file_after_local_time() {
        let root = tempfile::tempdir().unwrap();
        let folder = root.path().join("Pictures/Camera");
        let path = save_jpeg(&folder, b"jpeg", sample_time()).unwrap();
        assert_eq!(path, folder.join("2026-11-12_09-05-07.jpg"));
        assert_eq!(fs::read(path).unwrap(), b"jpeg");
    }

    #[test]
    fn a_taken_name_gets_a_numeric_suffix_and_nothing_is_overwritten() {
        let folder = tempfile::tempdir().unwrap();
        let paths: Vec<_> = (0..3)
            .map(|n| save_jpeg(folder.path(), &[n], sample_time()).unwrap())
            .collect();
        let names: Vec<_> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "2026-11-12_09-05-07.jpg",
                "2026-11-12_09-05-07_1.jpg",
                "2026-11-12_09-05-07_2.jpg"
            ]
        );
        assert_eq!(fs::read(&paths[0]).unwrap(), [0]);
    }

    #[test]
    fn an_unwritable_folder_is_an_error() {
        let file = tempfile::NamedTempFile::new().unwrap();
        assert!(save_jpeg(&file.path().join("Camera"), b"jpeg", sample_time()).is_err());
    }

    #[test]
    fn folders_inside_home_are_shown_relative_to_it() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(
            display_folder(&home.join("Pictures/Camera")),
            "Pictures/Camera"
        );
        assert_eq!(display_folder(Path::new("/srv/photos")), "/srv/photos");
    }
}
