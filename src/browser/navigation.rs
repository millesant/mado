use relm4::gtk::glib;

pub(crate) fn is_trusted_popup_source(uri: Option<&str>) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_sources_are_limited_to_chatgpt_and_openai_https_origins() {
        assert!(is_trusted_popup_source(Some("https://chatgpt.com/")));
        assert!(is_trusted_popup_source(Some("https://auth.openai.com/")));
        assert!(!is_trusted_popup_source(Some("http://chatgpt.com/")));
        assert!(!is_trusted_popup_source(Some(
            "https://chatgpt.com.example.org/"
        )));
        assert!(!is_trusted_popup_source(Some("https://example.org/")));
        assert!(!is_trusted_popup_source(None));
    }

    #[test]
    fn popup_targets_allow_https_and_initial_blank_document_only() {
        assert!(is_allowed_popup_target("https://accounts.google.com/"));
        assert!(is_allowed_popup_target("about:blank"));
        assert!(!is_allowed_popup_target("http://accounts.google.com/"));
        assert!(!is_allowed_popup_target("javascript:alert(1)"));
        assert!(!is_allowed_popup_target("not a uri"));
    }
}
