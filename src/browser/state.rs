use std::{fs, io, path::Path};

use relm4::gtk::{self, prelude::*};

const DEFAULT_WIDTH: i32 = 1280;
const DEFAULT_HEIGHT: i32 = 900;
const MIN_WIDTH: i32 = 320;
const MIN_HEIGHT: i32 = 240;
const MAX_DIMENSION: i32 = 16_384;
const MAX_STATE_BYTES: u64 = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WindowState {
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            maximized: false,
        }
    }
}

impl WindowState {
    pub(crate) fn load(path: &Path) -> io::Result<Self> {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };

        if metadata.len() > MAX_STATE_BYTES {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)?;
        Ok(Self::parse(&contents).unwrap_or_default())
    }

    pub(crate) fn capture(window: &gtk::ApplicationWindow) -> Self {
        Self {
            width: window.width().clamp(MIN_WIDTH, MAX_DIMENSION),
            height: window.height().clamp(MIN_HEIGHT, MAX_DIMENSION),
            maximized: window.is_maximized(),
        }
    }

    pub(crate) fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temporary = path.with_extension("tmp");
        fs::write(&temporary, self.serialize())?;
        fs::rename(temporary, path)
    }

    fn parse(contents: &str) -> Option<Self> {
        let mut version = None;
        let mut width = None;
        let mut height = None;
        let mut maximized = None;

        for line in contents.lines() {
            let (key, value) = line.split_once('=')?;
            match key {
                "version" => version = value.parse::<u32>().ok(),
                "width" => width = value.parse::<i32>().ok(),
                "height" => height = value.parse::<i32>().ok(),
                "maximized" => maximized = value.parse::<bool>().ok(),
                _ => return None,
            }
        }

        let width = width?;
        let height = height?;

        if version != Some(1)
            || !(MIN_WIDTH..=MAX_DIMENSION).contains(&width)
            || !(MIN_HEIGHT..=MAX_DIMENSION).contains(&height)
        {
            return None;
        }

        Some(Self {
            width,
            height,
            maximized: maximized?,
        })
    }

    fn serialize(self) -> String {
        format!(
            "version=1\nwidth={}\nheight={}\nmaximized={}\n",
            self.width, self.height, self.maximized
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_state_round_trips() {
        let state = WindowState {
            width: 1440,
            height: 960,
            maximized: true,
        };

        assert_eq!(WindowState::parse(&state.serialize()), Some(state));
    }

    #[test]
    fn window_state_file_round_trips() {
        let path =
            std::env::temp_dir().join(format!("mado-window-state-test-{}", std::process::id()));
        let state = WindowState {
            width: 1111,
            height: 777,
            maximized: false,
        };

        state.save(&path).unwrap();
        assert_eq!(WindowState::load(&path).unwrap(), state);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_or_out_of_range_state_falls_back() {
        assert_eq!(
            WindowState::parse("version=2\nwidth=800\nheight=600\nmaximized=false\n"),
            None
        );
        assert_eq!(
            WindowState::parse("version=1\nwidth=10\nheight=600\nmaximized=false\n"),
            None
        );
        assert_eq!(
            WindowState::parse("version=1\nwidth=800\nheight=600\nmaximized=maybe\n"),
            None
        );
        assert_eq!(
            WindowState::parse("version=1\nwidth=800\nheight=600\nmaximized=false\nextra=1\n"),
            None
        );
    }
}
