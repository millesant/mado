use std::path::{Path, PathBuf};

use relm4::gtk::glib;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct XdgDirectories {
    config: PathBuf,
    data: PathBuf,
    state: PathBuf,
    cache: PathBuf,
}

impl XdgDirectories {
    pub(crate) fn discover() -> Self {
        Self {
            config: glib::user_config_dir().join("mado"),
            data: glib::user_data_dir().join("mado"),
            state: glib::user_state_dir().join("mado"),
            cache: glib::user_cache_dir().join("mado"),
        }
    }

    pub(crate) fn config(&self) -> &Path {
        &self.config
    }

    pub(crate) fn data(&self) -> &Path {
        &self.data
    }

    pub(crate) fn state(&self) -> &Path {
        &self.state
    }

    pub(crate) fn cache(&self) -> &Path {
        &self.cache
    }

    #[cfg(test)]
    pub(crate) fn from_roots(
        config: PathBuf,
        data: PathBuf,
        state: PathBuf,
        cache: PathBuf,
    ) -> Self {
        Self {
            config,
            data,
            state,
            cache,
        }
    }
}
