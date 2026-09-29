use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::{platform::xdg::XdgDirectories, profiles::ProfileId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfilePaths {
    config: PathBuf,
    data: PathBuf,
    state: PathBuf,
    cache: PathBuf,
}

impl ProfilePaths {
    pub(crate) fn new(xdg: &XdgDirectories, profile: &ProfileId) -> Self {
        let relative = Path::new("profiles").join(profile.as_str());
        Self {
            config: xdg.config().join(&relative),
            data: xdg.data().join(&relative),
            state: xdg.state().join(&relative),
            cache: xdg.cache().join(&relative),
        }
    }

    pub(crate) fn prepare(&self) -> io::Result<()> {
        for path in [
            &self.config,
            &self.data,
            &self.state,
            &self.cache,
            &self.webkit_data_dir(),
            &self.webkit_cache_dir(),
        ] {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    pub(crate) fn webkit_data_dir(&self) -> PathBuf {
        self.data.join("webkit")
    }

    pub(crate) fn webkit_cache_dir(&self) -> PathBuf {
        self.cache.join("webkit")
    }

    pub(crate) fn cookie_database(&self) -> PathBuf {
        self.data.join("cookies.sqlite")
    }

    pub(crate) fn window_state_file(&self) -> PathBuf {
        self.state.join("window-state")
    }

    pub(crate) fn draft_state_file(&self) -> PathBuf {
        self.state.join("drafts.json")
    }

    #[allow(dead_code)]
    pub(crate) fn remove_owned_data(&self) -> io::Result<()> {
        for path in [&self.config, &self.data, &self.state, &self.cache] {
            match fs::remove_dir_all(path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
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
            "mado-profile-test-{}-{nonce}-{}",
            std::process::id(),
            NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn test_xdg(root: &Path) -> XdgDirectories {
        XdgDirectories::from_roots(
            root.join("config"),
            root.join("data"),
            root.join("state"),
            root.join("cache"),
        )
    }

    #[test]
    fn profile_paths_are_isolated_under_each_xdg_root() {
        let root = PathBuf::from("/tmp/mado-xdg");
        let xdg = test_xdg(&root);
        let profile = ProfileId::parse("work").unwrap();
        let paths = ProfilePaths::new(&xdg, &profile);

        assert_eq!(paths.config, root.join("config/profiles/work"));
        assert_eq!(paths.data, root.join("data/profiles/work"));
        assert_eq!(paths.state, root.join("state/profiles/work"));
        assert_eq!(paths.cache, root.join("cache/profiles/work"));
        assert_eq!(
            paths.webkit_data_dir(),
            root.join("data/profiles/work/webkit")
        );
        assert_eq!(
            paths.webkit_cache_dir(),
            root.join("cache/profiles/work/webkit")
        );
        assert_eq!(
            paths.cookie_database(),
            root.join("data/profiles/work/cookies.sqlite")
        );
        assert_eq!(
            paths.window_state_file(),
            root.join("state/profiles/work/window-state")
        );
        assert_eq!(
            paths.draft_state_file(),
            root.join("state/profiles/work/drafts.json")
        );
    }

    #[test]
    fn deleting_one_profile_does_not_remove_another_profile() {
        let root = test_root();
        let xdg = test_xdg(&root);
        let a = ProfilePaths::new(&xdg, &ProfileId::parse("profile-a").unwrap());
        let b = ProfilePaths::new(&xdg, &ProfileId::parse("profile-b").unwrap());

        a.prepare().unwrap();
        b.prepare().unwrap();
        fs::write(a.data.join("owned.txt"), b"a").unwrap();
        fs::write(b.data.join("owned.txt"), b"b").unwrap();

        a.remove_owned_data().unwrap();

        assert!(!a.config.exists());
        assert!(!a.data.exists());
        assert!(!a.state.exists());
        assert!(!a.cache.exists());

        assert!(b.config.exists());
        assert_eq!(fs::read(b.data.join("owned.txt")).unwrap(), b"b");
        assert!(b.state.exists());
        assert!(b.cache.exists());

        fs::remove_dir_all(root).unwrap();
    }
}
