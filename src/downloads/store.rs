use std::{
    fs, io,
    path::{Path, PathBuf},
};

use relm4::gtk::glib;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DownloadPaths {
    pub(crate) partial_path: PathBuf,
    pub(crate) final_path: PathBuf,
}

impl DownloadPaths {
    pub(crate) fn new(directory: &Path, final_path: PathBuf, id: u64) -> Self {
        let mut suffix = 0_u32;
        let partial_path = loop {
            let name = if suffix == 0 {
                format!(".mado-download-{}-{id}.part", std::process::id())
            } else {
                format!(".mado-download-{}-{id}-{suffix}.part", std::process::id())
            };
            let candidate = directory.join(name);
            if !candidate.exists() {
                break candidate;
            }
            suffix = suffix.saturating_add(1);
        };

        Self {
            partial_path,
            final_path,
        }
    }

    pub(crate) fn complete(&self) -> io::Result<()> {
        fs::hard_link(&self.partial_path, &self.final_path)?;
        match fs::remove_file(&self.partial_path) {
            Ok(()) => Ok(()),
            Err(error) => {
                let _ = fs::remove_file(&self.final_path);
                Err(error)
            }
        }
    }

    pub(crate) fn discard_partial(&self) -> io::Result<()> {
        match fs::remove_file(&self.partial_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

pub(crate) fn downloads_directory() -> PathBuf {
    glib::user_special_dir(glib::UserDirectory::Downloads).unwrap_or_else(glib::home_dir)
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    fn test_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "mado-download-store-test-{}-{nonce}-{}",
            std::process::id(),
            NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn partial_and_final_paths_share_directory_but_never_name() {
        let directory = PathBuf::from("/tmp/mado-downloads");
        let final_path = directory.join("report.txt");
        let paths = DownloadPaths::new(&directory, final_path.clone(), 7);

        assert_eq!(paths.partial_path.parent(), Some(directory.as_path()));
        assert_eq!(paths.final_path, final_path);
        assert_ne!(paths.partial_path, paths.final_path);
        assert_eq!(
            paths
                .partial_path
                .extension()
                .and_then(|value| value.to_str()),
            Some("part")
        );
    }

    #[test]
    fn complete_promotes_partial_file_only_after_success() {
        let directory = test_root();
        fs::create_dir_all(&directory).unwrap();
        let paths = DownloadPaths::new(&directory, directory.join("report.txt"), 1);

        fs::write(&paths.partial_path, b"complete bytes").unwrap();
        assert!(!paths.final_path.exists());

        paths.complete().unwrap();

        assert!(!paths.partial_path.exists());
        assert_eq!(fs::read(&paths.final_path).unwrap(), b"complete bytes");

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn discard_removes_partial_without_creating_final_file() {
        let directory = test_root();
        fs::create_dir_all(&directory).unwrap();
        let paths = DownloadPaths::new(&directory, directory.join("report.txt"), 2);

        fs::write(&paths.partial_path, b"partial").unwrap();
        paths.discard_partial().unwrap();

        assert!(!paths.partial_path.exists());
        assert!(!paths.final_path.exists());

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn complete_never_overwrites_a_file_that_appears_mid_download() {
        let directory = test_root();
        fs::create_dir_all(&directory).unwrap();
        let paths = DownloadPaths::new(&directory, directory.join("report.txt"), 3);

        fs::write(&paths.partial_path, b"downloaded bytes").unwrap();
        fs::write(&paths.final_path, b"other process").unwrap();

        assert_eq!(
            paths.complete().unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(&paths.final_path).unwrap(), b"other process");
        assert_eq!(fs::read(&paths.partial_path).unwrap(), b"downloaded bytes");

        paths.discard_partial().unwrap();
        fs::remove_dir_all(directory).unwrap();
    }
}
