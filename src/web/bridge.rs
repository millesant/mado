use std::{error::Error, fmt};

use serde::Deserialize;
use webkit6::{UserContentInjectedFrames, UserContentManager, UserScript, UserScriptInjectionTime};

use crate::drafts::store::MAX_DRAFT_CHARACTERS;

use super::selectors::{CLOUDFLARE_CHALLENGE_SELECTOR, PROMPT_COMPOSER_SELECTOR};

pub(crate) const BRIDGE_PROTOCOL_VERSION: u16 = 1;
pub(crate) const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
pub(crate) const BRIDGE_WORLD_NAME: &str = "mado-web-integration-v1";

const BRIDGE_HANDLER_NAME: &str = "mado";
const BOOTSTRAP_SOURCE: &str = include_str!("scripts/bootstrap.js");
const DRAFT_SOURCE: &str = include_str!("scripts/draft.js");
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DraftRestoreReason {
    Restored,
    Blocked,
    PageChanged,
    InvalidDraft,
    ComposerMissing,
    ComposerNotEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BridgeMessage {
    Ready {
        page: PageKind,
        observers_allowed: bool,
    },
    DraftReady {
        path: String,
        composer_empty: bool,
    },
    DraftChanged {
        path: String,
        text: String,
    },
    DraftRestoreResult {
        path: String,
        restored: bool,
        reason: DraftRestoreReason,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftReadyPayload {
    path: String,
    composer_empty: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftChangedPayload {
    path: String,
    text: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftRestoreResultPayload {
    path: String,
    restored: bool,
    reason: DraftRestoreReason,
}

pub(crate) fn create_user_content_manager<F>(
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

    for (source, injection_time) in [
        (render_bootstrap_script(), UserScriptInjectionTime::Start),
        (render_draft_script(), UserScriptInjectionTime::End),
    ] {
        let script = UserScript::for_world(
            &source,
            UserContentInjectedFrames::TopFrame,
            injection_time,
            BRIDGE_WORLD_NAME,
            TRUSTED_SCRIPT_ORIGINS,
            &[],
        );
        manager.add_script(&script);
    }

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
            let payload: BridgeReadyPayload = parse_payload(envelope.payload)?;
            Ok(BridgeMessage::Ready {
                page: payload.page,
                observers_allowed: payload.observers_allowed,
            })
        }
        "draft_ready" => {
            let payload: DraftReadyPayload = parse_payload(envelope.payload)?;
            Ok(BridgeMessage::DraftReady {
                path: payload.path,
                composer_empty: payload.composer_empty,
            })
        }
        "draft_changed" => {
            let payload: DraftChangedPayload = parse_payload(envelope.payload)?;
            Ok(BridgeMessage::DraftChanged {
                path: payload.path,
                text: payload.text,
            })
        }
        "draft_restore_result" => {
            let payload: DraftRestoreResultPayload = parse_payload(envelope.payload)?;
            Ok(BridgeMessage::DraftRestoreResult {
                path: payload.path,
                restored: payload.restored,
                reason: payload.reason,
            })
        }
        _ => Err(MessageParseError::UnknownType),
    }
}

fn parse_payload<T: for<'de> Deserialize<'de>>(
    payload: serde_json::Value,
) -> Result<T, MessageParseError> {
    serde_json::from_value(payload).map_err(|_| MessageParseError::InvalidPayload)
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

fn render_draft_script() -> String {
    let selector = serde_json::to_string(PROMPT_COMPOSER_SELECTOR)
        .expect("static prompt composer selector should serialize");

    DRAFT_SOURCE
        .replace("__MADO_COMPOSER_SELECTOR__", &selector)
        .replace(
            "__MADO_MAX_DRAFT_CHARACTERS__",
            &MAX_DRAFT_CHARACTERS.to_string(),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_versioned_messages() {
        assert_eq!(
            parse_message(
                r#"{"v":1,"type":"bridge_ready","payload":{"page":"app","observers_allowed":true}}"#
            ),
            Ok(BridgeMessage::Ready {
                page: PageKind::App,
                observers_allowed: true,
            })
        );
        assert_eq!(
            parse_message(
                r#"{"v":1,"type":"draft_ready","payload":{"path":"/c/example","composer_empty":true}}"#
            ),
            Ok(BridgeMessage::DraftReady {
                path: "/c/example".into(),
                composer_empty: true,
            })
        );
        assert_eq!(
            parse_message(
                r#"{"v":1,"type":"draft_changed","payload":{"path":"/c/example","text":"hello"}}"#
            ),
            Ok(BridgeMessage::DraftChanged {
                path: "/c/example".into(),
                text: "hello".into(),
            })
        );
        assert_eq!(
            parse_message(
                r#"{"v":1,"type":"draft_restore_result","payload":{"path":"/c/example","restored":false,"reason":"composer_not_empty"}}"#
            ),
            Ok(BridgeMessage::DraftRestoreResult {
                path: "/c/example".into(),
                restored: false,
                reason: DraftRestoreReason::ComposerNotEmpty,
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
                r#"{"v":1,"type":"draft_changed","payload":{"path":"/","text":"hi","command":"shell"}}"#
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
    fn rejects_malformed_payloads_and_restore_reasons() {
        assert_eq!(
            parse_message(
                r#"{"v":1,"type":"bridge_ready","payload":{"page":"unknown","observers_allowed":true}}"#
            ),
            Err(MessageParseError::InvalidPayload)
        );
        assert_eq!(
            parse_message(
                r#"{"v":1,"type":"draft_restore_result","payload":{"path":"/","restored":false,"reason":"run_shell"}}"#
            ),
            Err(MessageParseError::InvalidPayload)
        );
    }

    #[test]
    fn rendered_scripts_are_versioned_bounded_and_centralize_selectors() {
        let bootstrap = render_bootstrap_script();
        let draft = render_draft_script();

        assert!(!bootstrap.contains("__MADO_BRIDGE_VERSION__"));
        assert!(!bootstrap.contains("__MADO_MAX_MESSAGE_BYTES__"));
        assert!(!bootstrap.contains("__MADO_CLOUDFLARE_SELECTOR__"));
        assert!(bootstrap.contains(&format!("const VERSION = {BRIDGE_PROTOCOL_VERSION};")));
        assert!(bootstrap.contains(&MAX_MESSAGE_BYTES.to_string()));
        assert!(bootstrap.contains("observersAllowed"));
        assert!(bootstrap.contains("challengeActive"));
        assert!(bootstrap.contains("bridge_ready"));

        assert!(!draft.contains("__MADO_COMPOSER_SELECTOR__"));
        assert!(!draft.contains("__MADO_MAX_DRAFT_CHARACTERS__"));
        assert!(draft.contains("#prompt-textarea"));
        assert!(draft.contains(&MAX_DRAFT_CHARACTERS.to_string()));
        assert!(draft.contains("draft_changed"));
        assert!(draft.contains("draft_restore_result"));

        for selector_piece in [
            ".cf-turnstile",
            "#challenge-stage",
            "challenges.cloudflare.com",
        ] {
            assert!(bootstrap.contains(selector_piece));
        }
    }
}
