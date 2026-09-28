use std::{io, path::Path};

use webkit6::{CookiePersistentStorage, NetworkSession};

use crate::profiles::storage::ProfilePaths;

pub(crate) fn create_profile_network_session(paths: &ProfilePaths) -> io::Result<NetworkSession> {
    paths.prepare()?;

    let webkit_data = paths.webkit_data_dir();
    let webkit_cache = paths.webkit_cache_dir();
    let cookie_database = paths.cookie_database();

    let session = NetworkSession::new(
        Some(path_as_utf8(&webkit_data)?),
        Some(path_as_utf8(&webkit_cache)?),
    );

    if let Some(cookie_manager) = session.cookie_manager() {
        cookie_manager.set_persistent_storage(
            path_as_utf8(&cookie_database)?,
            CookiePersistentStorage::Sqlite,
        );
    }

    Ok(session)
}

fn path_as_utf8(path: &Path) -> io::Result<&str> {
    path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Mado profile storage path is not valid UTF-8",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_utf8_paths_on_unix() {
        #[cfg(unix)]
        {
            use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

            let path = Path::new(OsStr::from_bytes(b"/tmp/mado-\xff"));
            assert_eq!(
                path_as_utf8(path).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
        }
    }
}
