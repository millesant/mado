use std::{error::Error, fmt};

use serde::Deserialize;
use webkit6::{UserContentInjectedFrames, UserContentManager, UserScript, UserScriptInjectionTime};

use super::selectors::CLOUDFLARE_CHALLENGE_SELECTOR;

pub(crate) const BRIDGE_PROTOCOL_VERSION: u16 = 1;
pub(crate) const MAX_MESSAGE_BYTES: usize = 16 * 1024;

const BRIDGE_HANDLER_NAME: &str = "mado";
const BRIDGE_WORLD_NAME: &str = "mado-web-integration-v1";
const BOOTSTRAP_SOURCE: &str = include_str!("scripts/bootstrap.js");
const TRUSTED_SCRIPT_ORIGINS: &[&str] = &[
    "https://chatgpt.com/*",
    "https://*.chatgpt.com/*",
    "https://openai.com/*",
    "https://*.openai.com/*",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PageKind {
    App,
    Auth,
    Challenge,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BridgeMessage {
    Ready {
        page: PageKind,
        observers_allowed: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MessageParseError {
    TooLarge,
    InvalidJson,
    UnsupportedVersion,
    UnknownType,
    InvalidPayload,
}

impl fmt::Display for MessageParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TooLarge => "bridge message exceeded the size limit",
            Self::InvalidJson => "bridge message was not valid JSON",
            Self::UnsupportedVersion => "bridge message used an unsupported protocol version",
            Self::UnknownType => "bridge message type was not recognized",
            Self::InvalidPayload => "bridge message payload did not match its schema",
        };
        formatter.write_str(message)
    }
}

impl Error for MessageParseError {}

#[derive(Debug)]
pub(crate) struct BridgeSetupError;

impl fmt::Display for BridgeSetupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("failed to register the isolated WebKit bridge handler")
    }
}

impl Error for BridgeSetupError {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnvelope {
    v: u16,
    #[serde(rename = "type")]
    kind: String,
    payload: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BridgeReadyPayload {
    page: PageKind,
    observers_allowed: bool,
}

pub(crate) fn create_user_content_manager() -> Result<UserContentManager, BridgeSetupError> {
    create_user_content_manager_with_handler(|message| match message {
        BridgeMessage::Ready {
            page,
            observers_allowed,
        } => {
            let _ = (page, observers_allowed);
        }
    })
}

fn create_user_content_manager_with_handler<F>(
    on_message: F,
) -> Result<UserContentManager, BridgeSetupError>
where
    F: Fn(BridgeMessage) + 'static,
{
    let manager = UserContentManager::new();

    if !manager.register_script_message_handler(BRIDGE_HANDLER_NAME, Some(BRIDGE_WORLD_NAME)) {
        return Err(BridgeSetupError);
    }

    manager.connect_script_message_received(Some(BRIDGE_HANDLER_NAME), move |_, value| {
        if !value.is_string() {
            return;
        }

        let raw = value.to_str();
        if let Ok(message) = parse_message(raw.as_str()) {
            on_message(message);
        }
    });

    let source = render_bootstrap_script();
    let script = UserScript::for_world(
        &source,
        UserContentInjectedFrames::TopFrame,
        UserScriptInjectionTime::Start,
        BRIDGE_WORLD_NAME,
        TRUSTED_SCRIPT_ORIGINS,
        &[],
    );
    manager.add_script(&script);

    Ok(manager)
}

pub(crate) fn parse_message(raw: &str) -> Result<BridgeMessage, MessageParseError> {
    if raw.len() > MAX_MESSAGE_BYTES {
        return Err(MessageParseError::TooLarge);
    }

    let envelope: RawEnvelope =
        serde_json::from_str(raw).map_err(|_| MessageParseError::InvalidJson)?;

    if envelope.v != BRIDGE_PROTOCOL_VERSION {
        return Err(MessageParseError::UnsupportedVersion);
    }

    match envelope.kind.as_str() {
        "bridge_ready" => {
            let payload: BridgeReadyPayload = serde_json::from_value(envelope.payload)
                .map_err(|_| MessageParseError::InvalidPayload)?;
            Ok(BridgeMessage::Ready {
                page: payload.page,
                observers_allowed: payload.observers_allowed,
            })
        }
        _ => Err(MessageParseError::UnknownType),
    }
}

fn render_bootstrap_script() -> String {
    let selector = serde_json::to_string(CLOUDFLARE_CHALLENGE_SELECTOR)
        .expect("static Cloudflare selector should serialize");

    BOOTSTRAP_SOURCE
        .replace(
            "__MADO_BRIDGE_VERSION__",
            &BRIDGE_PROTOCOL_VERSION.to_string(),
        )
        .replace("__MADO_MAX_MESSAGE_BYTES__", &MAX_MESSAGE_BYTES.to_string())
        .replace("__MADO_CLOUDFLARE_SELECTOR__", &selector)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_versioned_message() {
        assert_eq!(
            parse_message(
                r#"{"v":1,"type":"bridge_ready","payload":{"page":"app","observers_allowed":true}}"#
            ),
            Ok(BridgeMessage::Ready {
                page: PageKind::App,
                observers_allowed: true,
            })
        );
    }

    #[test]
    fn rejects_unknown_versions_types_and_extra_fields() {
        assert_eq!(
            parse_message(
                r#"{"v":2,"type":"bridge_ready","payload":{"page":"app","observers_allowed":true}}"#
            ),
            Err(MessageParseError::UnsupportedVersion)
        );
        assert_eq!(
            parse_message(r#"{"v":1,"type":"native_eval","payload":{}}"#),
            Err(MessageParseError::UnknownType)
        );
        assert_eq!(
            parse_message(
                r#"{"v":1,"type":"bridge_ready","payload":{"page":"app","observers_allowed":true,"command":"shell"}}"#
            ),
            Err(MessageParseError::InvalidPayload)
        );
        assert_eq!(
            parse_message(
                r#"{"v":1,"type":"bridge_ready","payload":{"page":"app","observers_allowed":true},"extra":1}"#
            ),
            Err(MessageParseError::InvalidJson)
        );
    }

    #[test]
    fn enforces_message_size_before_json_parsing() {
        let oversized = "x".repeat(MAX_MESSAGE_BYTES + 1);
        assert_eq!(parse_message(&oversized), Err(MessageParseError::TooLarge));
    }

    #[test]
    fn rejects_malformed_payloads_and_page_kinds() {
        assert_eq!(
            parse_message(
                r#"{"v":1,"type":"bridge_ready","payload":{"page":"unknown","observers_allowed":true}}"#
            ),
            Err(MessageParseError::InvalidPayload)
        );
        assert_eq!(
            parse_message(
                r#"{"v":1,"type":"bridge_ready","payload":{"page":"app","observers_allowed":"yes"}}"#
            ),
            Err(MessageParseError::InvalidPayload)
        );
    }

    #[test]
    fn rendered_bootstrap_is_versioned_bounded_and_centralizes_challenge_selector() {
        let source = render_bootstrap_script();

        assert!(!source.contains("__MADO_BRIDGE_VERSION__"));
        assert!(!source.contains("__MADO_MAX_MESSAGE_BYTES__"));
        assert!(!source.contains("__MADO_CLOUDFLARE_SELECTOR__"));
        assert!(source.contains(&format!("const VERSION = {BRIDGE_PROTOCOL_VERSION};")));
        assert!(source.contains(&MAX_MESSAGE_BYTES.to_string()));
        assert!(source.contains("observersAllowed"));
        assert!(source.contains("challengeActive"));
        assert!(source.contains("bridge_ready"));

        for selector_piece in [
            ".cf-turnstile",
            "#challenge-stage",
            "challenges.cloudflare.com",
        ] {
            assert!(source.contains(selector_piece));
        }
    }
}
