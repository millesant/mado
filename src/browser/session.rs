use std::{
    fs, io,
    path::{Path, PathBuf},
};

use relm4::gtk::glib;
use webkit6::{CookiePersistentStorage, NetworkSession};

pub(crate) fn configure_cookie_persistence() -> io::Result<()> {
    let data_dir = glib::user_data_dir().join("mado");
    fs::create_dir_all(&data_dir)?;

    let cookie_database = cookie_database_path(&data_dir);
    let cookie_database = cookie_database.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Mado cookie database path is not valid UTF-8",
        )
    })?;

    if let Some(session) = NetworkSession::default()
        && let Some(cookie_manager) = session.cookie_manager()
    {
        cookie_manager.set_persistent_storage(cookie_database, CookiePersistentStorage::Sqlite);
    }

    Ok(())
}

fn cookie_database_path(data_dir: &Path) -> PathBuf {
    data_dir.join("cookies.sqlite")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_database_lives_under_mado_data_directory() {
        assert_eq!(
            cookie_database_path(Path::new("/tmp/mado-test")),
            PathBuf::from("/tmp/mado-test/cookies.sqlite")
        );
    }
}
