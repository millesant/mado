use webkit6::{WebView, prelude::WebViewExt};

use super::navigation::is_trusted_web_origin;

pub(crate) fn configure_file_chooser(web_view: &WebView) {
    web_view.connect_run_file_chooser(|web_view, request| {
        if is_trusted_web_origin(web_view.uri().as_deref()) {
            // Let WebKitGTK's default handler present the native GTK file chooser.
            false
        } else {
            request.cancel();
            true
        }
    });
}
