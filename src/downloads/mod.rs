mod filename;
mod store;

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fs,
    path::PathBuf,
    rc::Rc,
};

use relm4::gtk::{self, prelude::*};
use webkit6::{Download, NetworkSession};

use filename::safe_unique_destination;
use store::{DownloadPaths, downloads_directory};

#[derive(Clone)]
pub(crate) struct DownloadController {
    inner: Rc<DownloadControllerInner>,
}

struct DownloadControllerInner {
    next_id: Cell<u64>,
    active: RefCell<HashMap<u64, DownloadRecord>>,
    banner: gtk::Box,
    label: gtk::Label,
    progress: gtk::ProgressBar,
    download_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct DownloadRecord {
    filename: String,
    paths: Option<DownloadPaths>,
    received_bytes: u64,
}

impl DownloadController {
    pub(crate) fn new() -> Self {
        let banner = gtk::Box::new(gtk::Orientation::Vertical, 6);
        banner.set_halign(gtk::Align::End);
        banner.set_valign(gtk::Align::Start);
        banner.set_margin_top(18);
        banner.set_margin_end(18);
        banner.set_size_request(340, -1);
        banner.add_css_class("card");
        banner.set_visible(false);

        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::Start);
        label.set_wrap(true);

        let progress = gtk::ProgressBar::new();
        progress.set_show_text(false);

        banner.append(&label);
        banner.append(&progress);

        Self {
            inner: Rc::new(DownloadControllerInner {
                next_id: Cell::new(1),
                active: RefCell::new(HashMap::new()),
                banner,
                label,
                progress,
                download_dir: downloads_directory(),
            }),
        }
    }

    pub(crate) fn widget(&self) -> gtk::Box {
        self.inner.banner.clone()
    }

    pub(crate) fn attach(&self, session: &NetworkSession) {
        let controller = self.clone();
        session.connect_download_started(move |_, download| {
            controller.begin_download(download);
        });
    }

    fn begin_download(&self, download: &Download) {
        let id = self.inner.next_id.get();
        self.inner.next_id.set(id.saturating_add(1));

        self.inner.active.borrow_mut().insert(
            id,
            DownloadRecord {
                filename: "ChatGPT download".to_owned(),
                paths: None,
                received_bytes: 0,
            },
        );

        self.show_status("Preparing download…", Some(0.0));

        let destination_controller = self.clone();
        download.connect_decide_destination(move |download, suggested_filename| {
            destination_controller.choose_destination(id, download, suggested_filename)
        });

        let progress_controller = self.clone();
        download.connect_received_data(move |download, length| {
            progress_controller.received_data(id, download, length);
        });

        let failed_controller = self.clone();
        download.connect_failed(move |_, error| {
            failed_controller.download_failed(id, &error.message());
        });

        let finished_controller = self.clone();
        download.connect_finished(move |_| {
            finished_controller.download_finished(id);
        });
    }

    fn choose_destination(&self, id: u64, download: &Download, suggested_filename: &str) -> bool {
        if fs::create_dir_all(&self.inner.download_dir).is_err() {
            download.cancel();
            self.download_failed(id, "could not create the Downloads directory");
            return true;
        }

        let reserved: Vec<PathBuf> = self
            .inner
            .active
            .borrow()
            .values()
            .filter_map(|record| record.paths.as_ref().map(|paths| paths.final_path.clone()))
            .collect();
        let final_path =
            safe_unique_destination(&self.inner.download_dir, suggested_filename, |candidate| {
                reserved.iter().any(|path| path == candidate)
            });
        let paths = DownloadPaths::new(&self.inner.download_dir, final_path, id);

        let filename = paths
            .final_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("chatgpt-download")
            .to_owned();

        if let Some(record) = self.inner.active.borrow_mut().get_mut(&id) {
            record.filename = filename.clone();
            record.paths = Some(paths.clone());
        }

        let destination = gtk::gio::File::for_path(&paths.partial_path).uri();
        download.set_allow_overwrite(false);
        download.set_destination(destination.as_str());
        self.show_status(&format!("Downloading {filename}"), Some(0.0));
        true
    }

    fn received_data(&self, id: u64, download: &Download, length: u64) {
        let mut active = self.inner.active.borrow_mut();
        let Some(record) = active.get_mut(&id) else {
            return;
        };

        record.received_bytes = record.received_bytes.saturating_add(length);
        let filename = record.filename.clone();
        drop(active);

        let progress = download.estimated_progress();
        if progress.is_finite() && progress > 0.0 {
            let bounded = progress.clamp(0.0, 1.0);
            self.show_status(
                &format!("Downloading {filename} — {:.0}%", bounded * 100.0),
                Some(bounded),
            );
        } else {
            let received = format_bytes(
                self.inner
                    .active
                    .borrow()
                    .get(&id)
                    .map(|record| record.received_bytes)
                    .unwrap_or(0),
            );
            self.show_status(&format!("Downloading {filename} — {received}"), None);
        }
    }

    fn download_finished(&self, id: u64) {
        let Some(record) = self.inner.active.borrow_mut().remove(&id) else {
            return;
        };
        let Some(paths) = record.paths else {
            self.show_status("Download finished without a destination", None);
            return;
        };

        match paths.complete() {
            Ok(()) => {
                self.show_status(&format!("Downloaded {}", record.filename), Some(1.0));
            }
            Err(error) => {
                let _ = paths.discard_partial();
                self.show_status(
                    &format!("Download failed while saving {}: {error}", record.filename),
                    None,
                );
            }
        }
    }

    fn download_failed(&self, id: u64, message: &str) {
        let record = self.inner.active.borrow_mut().remove(&id);
        if let Some(record) = record {
            if let Some(paths) = record.paths {
                let _ = paths.discard_partial();
            }
            self.show_status(
                &format!("Download failed: {} ({message})", record.filename),
                None,
            );
        } else {
            self.show_status(&format!("Download failed: {message}"), None);
        }
    }

    fn show_status(&self, text: &str, fraction: Option<f64>) {
        self.inner.label.set_label(text);
        match fraction {
            Some(fraction) => {
                self.inner.progress.set_fraction(fraction.clamp(0.0, 1.0));
                self.inner.progress.set_visible(true);
            }
            None => {
                self.inner.progress.set_fraction(0.0);
                self.inner.progress.set_visible(false);
            }
        }
        self.inner.banner.set_visible(true);
    }
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * KIB;
    const GIB: f64 = 1024.0 * MIB;

    let bytes_f64 = bytes as f64;
    if bytes_f64 >= GIB {
        format!("{:.1} GiB", bytes_f64 / GIB)
    } else if bytes_f64 >= MIB {
        format!("{:.1} MiB", bytes_f64 / MIB)
    } else if bytes_f64 >= KIB {
        format!("{:.1} KiB", bytes_f64 / KIB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_progress_format_is_bounded_and_readable() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
    }
}
