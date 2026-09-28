use webkit6::{WebView, prelude::WebViewExt};

pub const CHATGPT_URL: &str = "https://chatgpt.com/";

pub fn chatgpt_web_view() -> WebView {
    let web_view = WebView::new();
    web_view.load_uri(CHATGPT_URL);
    web_view
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_url_is_chatgpt() {
        assert_eq!(CHATGPT_URL, "https://chatgpt.com/");
    }
}
