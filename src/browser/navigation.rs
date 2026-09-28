use glib::prelude::Cast;
use relm4::gtk::{gio, glib};
use webkit6::{
    NavigationPolicyDecision, PolicyDecisionType, WebView,
    prelude::{PolicyDecisionExt, WebViewExt},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavigationDisposition {
    Embedded,
    External,
    Deny,
}

pub(crate) fn configure_navigation_policy(web_view: &WebView) {
    web_view.connect_decide_policy(|web_view, decision, decision_type| {
        if decision_type != PolicyDecisionType::NavigationAction {
            return false;
        }

        let Some(navigation) = decision.downcast_ref::<NavigationPolicyDecision>() else {
            return false;
        };
        let Some(action) = navigation.navigation_action() else {
            return false;
        };
        let Some(target) = action.request().and_then(|request| request.uri()) else {
            return false;
        };

        match classify_navigation(
            web_view.uri().as_deref(),
            target.as_str(),
            action.is_user_gesture(),
        ) {
            NavigationDisposition::Embedded => false,
            NavigationDisposition::External => {
                decision.ignore();
                if let Err(error) = gio::AppInfo::launch_default_for_uri(
                    target.as_str(),
                    None::<&gio::AppLaunchContext>,
                ) {
                    eprintln!("failed to open external link: {error}");
                }
                true
            }
            NavigationDisposition::Deny => {
                decision.ignore();
                true
            }
        }
    });
}

pub(crate) fn is_trusted_web_origin(uri: Option<&str>) -> bool {
    let Some(uri) = uri else {
        return false;
    };
    let Ok(uri) = glib::Uri::parse(uri, glib::UriFlags::NONE) else {
        return false;
    };
    if uri.scheme().as_str() != "https" {
        return false;
    }
    let Some(host) = uri.host() else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    host == "chatgpt.com"
        || host.ends_with(".chatgpt.com")
        || host == "openai.com"
        || host.ends_with(".openai.com")
}

pub(crate) fn is_allowed_popup_target(uri: &str) -> bool {
    if uri == "about:blank" {
        return true;
    }
    glib::Uri::parse(uri, glib::UriFlags::NONE)
        .is_ok_and(|parsed| parsed.scheme().as_str() == "https")
}

fn classify_navigation(
    source_uri: Option<&str>,
    target_uri: &str,
    is_user_gesture: bool,
) -> NavigationDisposition {
    if target_uri == "about:blank" || is_trusted_web_origin(Some(target_uri)) {
        return NavigationDisposition::Embedded;
    }

    let Ok(target) = glib::Uri::parse(target_uri, glib::UriFlags::NONE) else {
        return NavigationDisposition::Deny;
    };
    let scheme = target.scheme();

    if is_trusted_web_origin(source_uri) && is_user_gesture {
        return if scheme.as_str() == "https" {
            NavigationDisposition::External
        } else {
            NavigationDisposition::Deny
        };
    }

    if matches!(scheme.as_str(), "http" | "https") {
        NavigationDisposition::Embedded
    } else {
        NavigationDisposition::Deny
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_web_origins_are_limited_to_chatgpt_and_openai_https_hosts() {
        assert!(is_trusted_web_origin(Some("https://chatgpt.com/")));
        assert!(is_trusted_web_origin(Some("https://auth.openai.com/")));
        assert!(!is_trusted_web_origin(Some("http://chatgpt.com/")));
        assert!(!is_trusted_web_origin(Some(
            "https://chatgpt.com.example.org/"
        )));
        assert!(!is_trusted_web_origin(Some("https://example.org/")));
        assert!(!is_trusted_web_origin(None));
    }

    #[test]
    fn popup_targets_allow_https_and_initial_blank_document_only() {
        assert!(is_allowed_popup_target("https://accounts.google.com/"));
        assert!(is_allowed_popup_target("about:blank"));
        assert!(!is_allowed_popup_target("http://accounts.google.com/"));
        assert!(!is_allowed_popup_target("javascript:alert(1)"));
        assert!(!is_allowed_popup_target("not a uri"));
    }

    #[test]
    fn trusted_page_user_links_handoff_only_third_party_https() {
        let source = Some("https://chatgpt.com/");

        assert_eq!(
            classify_navigation(source, "https://example.com/", true),
            NavigationDisposition::External
        );
        assert_eq!(
            classify_navigation(source, "https://help.openai.com/", true),
            NavigationDisposition::Embedded
        );
        assert_eq!(
            classify_navigation(source, "mailto:test@example.com", true),
            NavigationDisposition::Deny
        );
    }

    #[test]
    fn non_user_redirects_and_oauth_pages_stay_embedded_for_http_https() {
        assert_eq!(
            classify_navigation(
                Some("https://chatgpt.com/"),
                "https://example.com/redirect",
                false,
            ),
            NavigationDisposition::Embedded
        );
        assert_eq!(
            classify_navigation(
                Some("https://accounts.google.com/"),
                "https://accounts.google.com/signin",
                true,
            ),
            NavigationDisposition::Embedded
        );
    }
}
