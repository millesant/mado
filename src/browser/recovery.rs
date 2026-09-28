use std::{cell::RefCell, rc::Rc};

use relm4::gtk::{self, prelude::*};
use webkit6::{LoadEvent, NetworkError, WebView, prelude::WebViewExt};

use super::window::CHATGPT_URL;

const MAX_AUTOMATIC_PROCESS_RECOVERIES: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailureKind {
    Navigation,
    WebProcess,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RecoveryState {
    Healthy,
    Recovering { target: String },
    Failed { kind: FailureKind, target: String },
}

#[derive(Debug)]
struct RecoveryTracker {
    state: RecoveryState,
    automatic_process_recoveries: u8,
    last_committed_target: String,
}

impl Default for RecoveryTracker {
    fn default() -> Self {
        Self {
            state: RecoveryState::Healthy,
            automatic_process_recoveries: 0,
            last_committed_target: CHATGPT_URL.to_owned(),
        }
    }
}

impl RecoveryTracker {
    fn navigation_committed(&mut self, uri: Option<&str>) {
        self.last_committed_target = safe_recovery_target(uri);
    }

    fn navigation_failed(&mut self) {
        self.state = RecoveryState::Failed {
            kind: FailureKind::Navigation,
            target: self.last_committed_target.clone(),
        };
    }

    fn web_process_terminated(&mut self) -> Option<String> {
        let target = self.last_committed_target.clone();
        if self.automatic_process_recoveries < MAX_AUTOMATIC_PROCESS_RECOVERIES {
            self.automatic_process_recoveries += 1;
            self.state = RecoveryState::Recovering {
                target: target.clone(),
            };
            Some(target)
        } else {
            self.state = RecoveryState::Failed {
                kind: FailureKind::WebProcess,
                target,
            };
            None
        }
    }

    fn manual_retry(&mut self) -> Option<String> {
        let target = match &self.state {
            RecoveryState::Failed { target, .. } => target.clone(),
            RecoveryState::Healthy | RecoveryState::Recovering { .. } => return None,
        };

        self.state = RecoveryState::Recovering {
            target: target.clone(),
        };
        Some(target)
    }

    fn load_finished(&mut self) {
        if matches!(self.state, RecoveryState::Recovering { .. }) {
            self.state = RecoveryState::Healthy;
            self.automatic_process_recoveries = 0;
        }
    }
}

pub(crate) fn wrap_with_recovery(web_view: &WebView) -> gtk::Overlay {
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(web_view));

    let recovery_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    recovery_box.set_halign(gtk::Align::Center);
    recovery_box.set_valign(gtk::Align::Start);
    recovery_box.set_margin_top(24);
    recovery_box.set_margin_start(24);
    recovery_box.set_margin_end(24);
    recovery_box.set_size_request(420, -1);
    recovery_box.add_css_class("card");

    let message = gtk::Label::new(None);
    message.set_wrap(true);
    message.set_justify(gtk::Justification::Center);

    let retry = gtk::Button::with_label("Reload ChatGPT");
    retry.set_halign(gtk::Align::Center);

    recovery_box.append(&message);
    recovery_box.append(&retry);
    recovery_box.set_visible(false);
    overlay.add_overlay(&recovery_box);

    let tracker = Rc::new(RefCell::new(RecoveryTracker::default()));

    let retry_tracker = tracker.clone();
    let retry_web_view = web_view.clone();
    let retry_box = recovery_box.clone();
    let retry_message = message.clone();
    let retry_button = retry.clone();
    retry.connect_clicked(move |_| {
        let Some(target) = retry_tracker.borrow_mut().manual_retry() else {
            return;
        };

        show_recovering(&retry_box, &retry_message, &retry_button);
        retry_web_view.stop_loading();
        retry_web_view.load_uri(&target);
    });

    let failure_tracker = tracker.clone();
    let failure_box = recovery_box.clone();
    let failure_message = message.clone();
    let failure_button = retry.clone();
    web_view.connect_load_failed(move |_, _, _failing_uri, error| {
        if error.matches(NetworkError::Cancelled) {
            return true;
        }

        failure_tracker.borrow_mut().navigation_failed();
        show_failed(
            &failure_box,
            &failure_message,
            &failure_button,
            "ChatGPT could not load this page. Check your connection, then reload.",
        );
        true
    });

    let changed_tracker = tracker.clone();
    let changed_box = recovery_box.clone();
    web_view.connect_load_changed(move |web_view, event| {
        let mut tracker = changed_tracker.borrow_mut();
        match event {
            LoadEvent::Committed => {
                tracker.navigation_committed(web_view.uri().as_deref());
            }
            LoadEvent::Finished => {
                tracker.load_finished();
                if matches!(tracker.state, RecoveryState::Healthy) {
                    changed_box.set_visible(false);
                }
            }
            _ => {}
        }
    });

    let terminated_tracker = tracker.clone();
    let terminated_box = recovery_box.clone();
    let terminated_message = message.clone();
    let terminated_button = retry.clone();
    web_view.connect_web_process_terminated(move |web_view, _reason| {
        let automatic_target = terminated_tracker.borrow_mut().web_process_terminated();

        if let Some(target) = automatic_target {
            show_recovering(&terminated_box, &terminated_message, &terminated_button);
            web_view.stop_loading();
            web_view.load_uri(&target);
        } else {
            show_failed(
                &terminated_box,
                &terminated_message,
                &terminated_button,
                "The WebKit web process stopped repeatedly. Reload when you are ready.",
            );
        }
    });

    overlay
}

fn show_recovering(container: &gtk::Box, message: &gtk::Label, retry: &gtk::Button) {
    message.set_label("The WebKit web process restarted. Recovering the current page…");
    retry.set_visible(false);
    container.set_visible(true);
}

fn show_failed(container: &gtk::Box, message: &gtk::Label, retry: &gtk::Button, text: &str) {
    message.set_label(text);
    retry.set_visible(true);
    container.set_visible(true);
}

fn safe_recovery_target(uri: Option<&str>) -> String {
    match uri {
        Some(uri) if uri.starts_with("https://") || uri.starts_with("http://") => uri.to_owned(),
        _ => CHATGPT_URL.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_failure_survives_followup_finished_event() {
        let mut tracker = RecoveryTracker::default();
        tracker.navigation_committed(Some("https://chatgpt.com/"));
        tracker.navigation_failed();

        tracker.load_finished();

        assert!(matches!(
            tracker.state,
            RecoveryState::Failed {
                kind: FailureKind::Navigation,
                ..
            }
        ));
    }

    #[test]
    fn failed_provisional_navigation_retries_last_committed_page() {
        let mut tracker = RecoveryTracker::default();
        tracker.navigation_committed(Some("https://chatgpt.com/c/good"));
        tracker.navigation_failed();

        assert_eq!(
            tracker.manual_retry().as_deref(),
            Some("https://chatgpt.com/c/good")
        );
    }

    #[test]
    fn web_process_recovery_is_automatic_once_then_bounded() {
        let mut tracker = RecoveryTracker::default();
        let target = "https://chatgpt.com/c/example";

        tracker.navigation_committed(Some(target));
        assert_eq!(tracker.web_process_terminated().as_deref(), Some(target));
        assert!(matches!(tracker.state, RecoveryState::Recovering { .. }));

        assert_eq!(tracker.web_process_terminated(), None);
        assert!(matches!(
            tracker.state,
            RecoveryState::Failed {
                kind: FailureKind::WebProcess,
                ..
            }
        ));
    }

    #[test]
    fn successful_recovery_resets_process_retry_budget() {
        let mut tracker = RecoveryTracker::default();
        let target = "https://chatgpt.com/";

        tracker.navigation_committed(Some(target));
        assert!(tracker.web_process_terminated().is_some());
        tracker.load_finished();

        assert_eq!(tracker.state, RecoveryState::Healthy);
        assert!(tracker.web_process_terminated().is_some());
    }

    #[test]
    fn manual_retry_only_runs_from_failed_state() {
        let mut tracker = RecoveryTracker::default();
        assert_eq!(tracker.manual_retry(), None);

        tracker.navigation_committed(Some("https://chatgpt.com/"));
        tracker.navigation_failed();
        assert_eq!(
            tracker.manual_retry().as_deref(),
            Some("https://chatgpt.com/")
        );
        assert!(matches!(tracker.state, RecoveryState::Recovering { .. }));
    }

    #[test]
    fn recovery_target_rejects_non_http_schemes() {
        assert_eq!(
            safe_recovery_target(Some("javascript:alert(1)")),
            CHATGPT_URL
        );
        assert_eq!(
            safe_recovery_target(Some("https://chatgpt.com/c/123")),
            "https://chatgpt.com/c/123"
        );
    }
}
