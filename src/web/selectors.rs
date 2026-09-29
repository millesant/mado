pub(crate) const CLOUDFLARE_CHALLENGE_SELECTOR: &str = r#"iframe[src*="challenges.cloudflare.com"],.cf-turnstile,#cf-challenge-running,#challenge-stage,[data-cf-challenge]"#;

pub(crate) const PROMPT_COMPOSER_SELECTOR: &str = r#"#prompt-textarea,textarea[data-testid="prompt-textarea"],[contenteditable="true"][data-testid="prompt-textarea"]"#;
