use relm4::gtk::{self, glib, prelude::*};
use webkit6::{WebView, prelude::*};

use super::{
    navigation::{is_allowed_popup_target, is_trusted_web_origin},
    session::configure_cookie_persistence,
    transfers::configure_file_chooser,
};

pub const CHATGPT_URL: &str = "https://chatgpt.com/";

pub fn chatgpt_web_view() -> WebView {
    configure_cookie_persistence().expect("failed to configure persistent WebKit cookies");
    let web_view = WebView::new();
    configure_file_chooser(&web_view);
    configure_popup_handling(&web_view);
    web_view.load_uri(CHATGPT_URL);
    web_view
}

pub fn stop_app_web_views(application: &gtk::Application) {
    for window in application.windows() {
        if let Some(child) = window.child()
            && let Ok(web_view) = child.downcast::<WebView>()
        {
            web_view.stop_loading();
        }
    }
}

fn configure_popup_handling(web_view: &WebView) {
    web_view.connect_create(|parent, action| {
        if !is_trusted_web_origin(parent.uri().as_deref()) {
            return None;
        }
        let target = action.request().and_then(|request| request.uri());
        if !target.as_deref().is_some_and(is_allowed_popup_target) {
            return None;
        }

        let popup = WebView::builder().related_view(parent).build();
        configure_file_chooser(&popup);
        configure_popup_handling(&popup);

        let window = gtk::ApplicationWindow::builder()
            .application(&relm4::main_application())
            .title("Mado — Sign in")
            .default_width(900)
            .default_height(700)
            .build();
        window.set_child(Some(&popup));

        let ready_window = window.clone();
        popup.connect_ready_to_show(move |_| ready_window.present());

        let close_window = window.clone();
        popup.connect_close(move |_| close_window.close());

        let closing_popup = popup.clone();
        window.connect_close_request(move |_| {
            closing_popup.stop_loading();
            glib::Propagation::Proceed
        });

        Some(popup.upcast())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_url_is_chatgpt() {
        assert_eq!(CHATGPT_URL, "https://chatgpt.com/");
    }
}
