use relm4::gtk::{self, glib, prelude::*};
use webkit6::{WebView, prelude::*};

use super::{
    navigation::{
        PopupDisposition, classify_popup_target, configure_navigation_policy,
        is_trusted_web_origin, open_external_uri,
    },
    permissions::configure_media_permissions,
    recovery::wrap_with_recovery,
    session::create_profile_network_session,
    transfers::configure_file_chooser,
};

pub const CHATGPT_URL: &str = "https://chatgpt.com/";

pub fn chatgpt_web_view(
    paths: &crate::profiles::storage::ProfilePaths,
    downloads: &crate::downloads::DownloadController,
) -> WebView {
    let session = create_profile_network_session(paths)
        .expect("failed to initialize persistent WebKit profile storage");
    downloads.attach(&session);

    let user_content_manager = crate::web::bridge::create_user_content_manager()
        .expect("failed to initialize isolated web-integration bridge");
    let web_view = WebView::builder()
        .network_session(&session)
        .user_content_manager(&user_content_manager)
        .build();
    configure_file_chooser(&web_view);
    configure_media_permissions(&web_view);
    configure_navigation_policy(&web_view);
    configure_popup_handling(&web_view);
    web_view.load_uri(CHATGPT_URL);
    web_view
}

pub fn stop_app_web_views(application: &gtk::Application) {
    for window in application.windows() {
        if let Some(child) = window.child() {
            stop_web_views_in_widget(&child);
        }
    }
}

fn stop_web_views_in_widget(widget: &gtk::Widget) {
    if let Some(web_view) = widget.downcast_ref::<WebView>() {
        web_view.stop_loading();
        return;
    }

    let mut child = widget.first_child();
    while let Some(current) = child {
        let next = current.next_sibling();
        stop_web_views_in_widget(&current);
        child = next;
    }
}

fn configure_popup_handling(web_view: &WebView) {
    web_view.connect_create(|parent, action| {
        if !is_trusted_web_origin(parent.uri().as_deref()) {
            return None;
        }
        let Some(target) = action.request().and_then(|request| request.uri()) else {
            return None;
        };

        match classify_popup_target(target.as_str()) {
            PopupDisposition::ChildWebView => {}
            PopupDisposition::External => {
                open_external_uri(target.as_str());
                return None;
            }
            PopupDisposition::Deny => return None,
        }

        let mut popup_builder = WebView::builder().related_view(parent);
        if let Some(user_content_manager) = parent.user_content_manager() {
            popup_builder = popup_builder.user_content_manager(&user_content_manager);
        }
        let popup = popup_builder.build();
        configure_file_chooser(&popup);
        configure_media_permissions(&popup);
        configure_navigation_policy(&popup);
        configure_popup_handling(&popup);

        let window = gtk::ApplicationWindow::builder()
            .application(&relm4::main_application())
            .title("Mado — Sign in")
            .default_width(900)
            .default_height(700)
            .build();
        let popup_widget = wrap_with_recovery(&popup);
        window.set_child(Some(&popup_widget));

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
