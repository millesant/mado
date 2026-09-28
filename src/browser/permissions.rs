use relm4::gtk::{self, prelude::*};
use webkit6::{
    PermissionRequest, UserMediaPermissionRequest, WebView,
    prelude::{PermissionRequestExt, WebViewExt},
};

use super::navigation::is_trusted_web_origin;

pub(crate) fn configure_media_permissions(web_view: &WebView) {
    web_view.connect_permission_request(|web_view, request| {
        let Some(media_request) = request.downcast_ref::<UserMediaPermissionRequest>() else {
            return false;
        };

        if !is_trusted_web_origin(web_view.uri().as_deref()) {
            request.deny();
            return true;
        }

        prompt_for_media_permission(web_view, request, media_request);
        true
    });
}

fn prompt_for_media_permission(
    web_view: &WebView,
    request: &PermissionRequest,
    media_request: &UserMediaPermissionRequest,
) {
    let kind = media_capture_kind(
        media_request.is_for_audio_device(),
        media_request.is_for_video_device(),
    );

    let dialog = gtk::AlertDialog::builder()
        .message(format!("Allow ChatGPT to use your {kind}?"))
        .detail("Mado only grants this request after your confirmation.")
        .buttons(["Don't Allow", "Allow"])
        .cancel_button(0)
        .default_button(1)
        .modal(true)
        .build();

    let parent = web_view.root().and_downcast::<gtk::Window>();
    let request = request.clone();
    dialog.choose(
        parent.as_ref(),
        None::<&gtk::gio::Cancellable>,
        move |result| {
            if result.is_ok_and(|choice| choice == 1) {
                request.allow();
            } else {
                request.deny();
            }
        },
    );
}

fn media_capture_kind(audio: bool, video: bool) -> &'static str {
    match (audio, video) {
        (true, true) => "microphone and camera",
        (true, false) => "microphone",
        (false, true) => "camera",
        (false, false) => "media devices",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_capture_kind_describes_requested_devices() {
        assert_eq!(media_capture_kind(true, false), "microphone");
        assert_eq!(media_capture_kind(false, true), "camera");
        assert_eq!(media_capture_kind(true, true), "microphone and camera");
        assert_eq!(media_capture_kind(false, false), "media devices");
    }
}
