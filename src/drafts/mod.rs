pub(crate) mod store;

use std::{cell::RefCell, rc::Rc};

use relm4::gtk::{self, prelude::*};
use webkit6::{WebView, prelude::WebViewExt};

use crate::web::bridge::{BRIDGE_WORLD_NAME, BridgeMessage, DraftRestoreReason};

use store::{DraftPageKey, DraftStore, MAX_DRAFT_CHARACTERS};

#[derive(Clone)]
pub(crate) struct DraftController {
    inner: Rc<DraftControllerInner>,
}

struct DraftControllerInner {
    store: RefCell<DraftStore>,
    current_page: RefCell<Option<DraftPageKey>>,
    web_view: RefCell<Option<WebView>>,
    bar: gtk::Box,
    label: gtk::Label,
    restore_button: gtk::Button,
}

impl DraftController {
    pub(crate) fn new(store_path: std::path::PathBuf) -> Self {
        let store = DraftStore::load(store_path.clone()).unwrap_or_else(|error| {
            eprintln!("failed to load saved drafts: {error}");
            DraftStore::empty(store_path)
        });

        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        bar.set_margin_top(8);
        bar.set_margin_bottom(8);
        bar.set_margin_start(12);
        bar.set_margin_end(12);
        bar.set_visible(false);

        let label = gtk::Label::new(Some("A saved draft is available for this page."));
        label.set_halign(gtk::Align::Start);
        label.set_hexpand(true);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);

        let restore_button = gtk::Button::with_label("Restore draft");

        bar.append(&label);
        bar.append(&restore_button);

        let controller = Self {
            inner: Rc::new(DraftControllerInner {
                store: RefCell::new(store),
                current_page: RefCell::new(None),
                web_view: RefCell::new(None),
                bar,
                label,
                restore_button,
            }),
        };

        let click_controller = controller.clone();
        controller
            .inner
            .restore_button
            .connect_clicked(move |_| click_controller.restore_current_draft());

        controller
    }

    pub(crate) fn widget(&self) -> gtk::Box {
        self.inner.bar.clone()
    }

    pub(crate) fn attach_web_view(&self, web_view: &WebView) {
        *self.inner.web_view.borrow_mut() = Some(web_view.clone());
    }

    pub(crate) fn handle_bridge_message(&self, message: BridgeMessage) {
        match message {
            BridgeMessage::Ready { .. } => {}
            BridgeMessage::DraftReady {
                path,
                composer_empty,
            } => self.handle_draft_ready(&path, composer_empty),
            BridgeMessage::DraftChanged { path, text } => {
                self.handle_draft_changed(&path, &text);
            }
            BridgeMessage::DraftRestoreResult {
                path,
                restored,
                reason,
            } => self.handle_restore_result(&path, restored, reason),
        }
    }

    fn handle_draft_ready(&self, raw_path: &str, composer_empty: bool) {
        let Some(page) = DraftPageKey::parse(raw_path) else {
            return;
        };

        *self.inner.current_page.borrow_mut() = Some(page.clone());

        let has_saved_draft = self.inner.store.borrow().draft(&page).is_some();
        if composer_empty && has_saved_draft {
            self.inner
                .label
                .set_label("A saved draft is available for this page.");
            self.inner.restore_button.set_sensitive(true);
            self.inner.restore_button.set_visible(true);
            self.inner.bar.set_visible(true);
        } else {
            self.inner.bar.set_visible(false);
        }
    }

    fn handle_draft_changed(&self, raw_path: &str, text: &str) {
        let Some(page) = DraftPageKey::parse(raw_path) else {
            return;
        };
        if text.trim().is_empty() || text.chars().count() > MAX_DRAFT_CHARACTERS {
            return;
        }

        if let Err(error) = self.inner.store.borrow_mut().save_draft(&page, text) {
            eprintln!("failed to save local draft: {error}");
        }

        if self
            .inner
            .current_page
            .borrow()
            .as_ref()
            .is_some_and(|current| current == &page)
        {
            self.inner.bar.set_visible(false);
        }
    }

    fn restore_current_draft(&self) {
        let Some(page) = self.inner.current_page.borrow().clone() else {
            return;
        };
        let Some(text) = self.inner.store.borrow().draft(&page).map(str::to_owned) else {
            self.inner.bar.set_visible(false);
            return;
        };
        let Some(web_view) = self.inner.web_view.borrow().clone() else {
            return;
        };

        let text_literal =
            serde_json::to_string(&text).expect("draft text should serialize as JSON string");
        let path_literal = serde_json::to_string(page.as_str())
            .expect("draft path should serialize as JSON string");
        let script = format!("globalThis.__madoDraft?.restore({text_literal}, {path_literal});");

        self.inner.label.set_label("Restoring saved draft…");
        self.inner.restore_button.set_sensitive(false);

        let result_controller = self.clone();
        let result_page = page.clone();
        web_view.evaluate_javascript(
            &script,
            Some(BRIDGE_WORLD_NAME),
            None,
            None::<&gtk::gio::Cancellable>,
            move |result| {
                if result.is_err() {
                    result_controller.show_restore_failure(
                        &result_page,
                        "Draft restore is temporarily unavailable; the saved draft was kept.",
                        true,
                    );
                }
            },
        );
    }

    fn handle_restore_result(&self, raw_path: &str, restored: bool, reason: DraftRestoreReason) {
        let Some(page) = DraftPageKey::parse(raw_path) else {
            return;
        };
        if !self
            .inner
            .current_page
            .borrow()
            .as_ref()
            .is_some_and(|current| current == &page)
        {
            return;
        }

        if restored && reason == DraftRestoreReason::Restored {
            self.inner.bar.set_visible(false);
            return;
        }

        match reason {
            DraftRestoreReason::ComposerNotEmpty => self.show_restore_failure(
                &page,
                "Composer already has text; it was kept and the saved draft was not changed.",
                false,
            ),
            DraftRestoreReason::PageChanged => self.show_restore_failure(
                &page,
                "The page changed before restore; the saved draft was kept.",
                false,
            ),
            DraftRestoreReason::Blocked => self.show_restore_failure(
                &page,
                "Draft restore is paused on this page; the saved draft was kept.",
                false,
            ),
            DraftRestoreReason::InvalidDraft | DraftRestoreReason::ComposerMissing => self
                .show_restore_failure(
                    &page,
                    "Draft restore is temporarily unavailable; the saved draft was kept.",
                    true,
                ),
            DraftRestoreReason::Restored => self.show_restore_failure(
                &page,
                "Draft restore did not complete; the saved draft was kept.",
                true,
            ),
        }
    }

    fn show_restore_failure(&self, page: &DraftPageKey, text: &str, allow_retry: bool) {
        if !self
            .inner
            .current_page
            .borrow()
            .as_ref()
            .is_some_and(|current| current == page)
        {
            return;
        }

        self.inner.label.set_label(text);
        self.inner.restore_button.set_visible(allow_retry);
        self.inner.restore_button.set_sensitive(allow_retry);
        self.inner.bar.set_visible(true);
    }
}
