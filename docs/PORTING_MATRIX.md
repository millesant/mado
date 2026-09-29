# Porting matrix

This document records architectural and behavioral parity decisions between the upstream Swift implementation and Mado.

Only Swift files still useful to upcoming parity work remain under `reference/swift/`. Completed historical references removed from the working tree remain available through Git history.

This file does not track task status. GitHub Issues and the Project board remain authoritative for work state.

| Upstream behavior / retained reference | Linux implementation | Notes |
| --- | --- | --- |
| application startup and lifecycle | `main.rs` + `application.rs` | Relm4/GTK GApplication; single-instance activation; Wayland-safe window state |
| `reference/swift/ChatGPTSwiftWeb/AppDelegate.swift` | application/profile/settings services | retained only for remaining P2 integration behavior |
| `reference/swift/ChatGPTSwiftWeb/BrowserWindowController.swift` | `browser/` + feature modules | WebKitGTK lifecycle and remaining integration context; do not translate AppKit mechanics |
| navigation / OAuth policy | `browser/navigation.rs` + `browser/window.rs` | trusted ChatGPT/OpenAI origins, related OAuth child views, third-party browser handoff |
| WebKit failure recovery | `browser/recovery.rs` | bounded native error state + one-shot WebProcess recovery; never resend prompts |
| isolated website data stores | `profiles/` + profile-scoped `NetworkSession` | XDG-backed data/cache/cookies with strict profile boundaries |
| `reference/swift/ChatGPTSwiftWeb/BrowserDataBoundary.swift` + `ProfileDataStoreInventory.swift` | profile deletion/data controls | retain for #18/#19 privacy and destructive-data behavior |
| `reference/swift/ChatGPTSwiftWeb/ConsentPreferenceSettings.swift` | future privacy/settings work | preserve safe user-facing intent only; never damage authentication/session state |
| web/native message handlers | `web/bridge.rs` + `web/scripts/` | isolated WebKit script world; trusted top-frame injection; typed/versioned/bounded events |
| ChatGPT DOM selectors / challenge guards | `web/selectors.rs` + feature scripts | volatile DOM knowledge stays inside the web-integration boundary |
| draft preservation/recovery | `web/scripts/draft.js` + `drafts/` | profile/page scoped; 350 ms debounce; explicit restore; existing composer text wins; never auto-send |
| normal and generated downloads | navigation policy + `downloads/` | native WebKit download pipeline; safe `.part` staging; no-clobber finalization; no base64 bridge |
| microphone/camera permissions | `browser/permissions.rs` | trusted top-level origin plus explicit native confirmation |
| `reference/swift/ChatGPTSwiftWeb/BrowserCompletionObserver.swift` | `web/scripts/completion.js` | generating -> idle detection must remain isolated and deduplicated |
| `reference/swift/ChatGPTSwiftWeb/CompletionNotificationService.swift` | native Linux notifications | notify only when appropriate/unfocused; failures must not affect chat |
| `reference/swift/ChatGPTSwiftWeb/AppSettingsWindowController.swift` | native Relm4/GTK settings/profile UI | preserve user-facing settings intent, not AppKit layout/mechanisms |
| `reference/swift/ChatGPTSwiftWeb/DiagnosticsWindowController.swift` + `reference/swift/ChatGPTSwiftWebCore/DiagnosticRedactor.swift` | sanitized diagnostics/privacy controls | no cookies, tokens, prompts, responses, raw HTML, or sensitive environment values |

## Parity policy

A feature is not considered ported merely because an analogous Linux API exists. Parity requires the useful observable behavior to survive the platform change and pass the acceptance criteria in its GitHub issue.

When upstream behavior is platform-specific, preserve the user-facing intent rather than copying the macOS mechanism. Features with no justified Linux-native equivalent are omitted instead of carried as dead compatibility baggage.
