# Porting matrix

This document maps important upstream Swift behavior to the intended Linux implementation.

It tracks architectural/behavioral parity only. GitHub Issues and the Project board track work status.

| Swift reference / behavior | Linux target | Phase | Notes |
|---|---|---|---|
| `main.swift` startup | `main.rs` + Relm4 application bootstrap | P0/P1 | Linux-only startup |
| `AppDelegate.swift` | `application.rs` + app actions/services | P1/P2 | Split responsibilities rather than one delegate |
| `BrowserWindowController.swift` | `browser/window.rs` | P0/P1 | WebKitGTK 6 WebView lifecycle |
| navigation policy | `browser/navigation.rs` | P1 | trusted origins, OAuth, external links |
| WebKit failure recovery | `browser/recovery.rs` | P1 | native bounded load-failure state + one-shot WebProcess reload; no network retry loop or prompt resend |
| `BrowserSupport.swift` | browser/profile/privacy modules | P1/P2 | split by responsibility |
| `WKUserContentController` / script-message handlers | `web/bridge.rs` + `web/scripts/` | P2 | isolated WebKit script world, trusted top-frame injection, versioned/bounded typed messages; no generic native commands |
| ChatGPT DOM selectors / challenge guards | `web/selectors.rs` + feature scripts | P2 | volatile DOM knowledge remains inside web-integration boundary; auth/Cloudflare pages suppress observers |
| isolated website data stores | `profiles/storage.rs` + profile-scoped `NetworkSession` | P1 | XDG-backed per-profile data/cache/cookies; no cross-profile fallback |
| cookie-consent defaults | `privacy/` + web scripts | P2 | must not damage login/session |
| draft preservation/recovery | `web/scripts/draft.js` + Rust store | P2 | no prompt auto-send |
| completion detection | `web/scripts/completion.js` | P2 | DOM-dependent and isolated |
| native completion notifications | `notifications/` | P2 | only when appropriate/unfocused |
| normal downloads | `downloads/` | P0/P1 | profile `NetworkSession` download stream, safe `.part` → final promotion, native status |
| `blob:` / `data:` generated downloads | navigation policy + `downloads/` | P0/P1 | trusted user-gesture policy decisions are promoted into native WebKit downloads; no JS/base64 bridge unless a future compatibility gap requires bounded chunks |
| OAuth/login popup handling | browser navigation/window policy | P0/P1 | must preserve login context |
| microphone/camera permissions | `browser/permissions.rs` | P0/P1 | trusted-origin policy |
| window frame persistence | `browser/state.rs` | P1 | profile-scoped size/maximized state only; no Wayland absolute positioning |
| single-instance lock | application activation | P1 | Relm4/GTK GApplication uniqueness; second launch presents existing window |
| Settings window | `settings/` | P2 | GTK/Relm4 native UI |
| diagnostics window | `diagnostics/` | P2 | sanitized by default |
| profile switcher | `profiles/` + GTK UI | P2 | preserve strict isolation |
| window title/profile identity | shell/settings | P2 | privacy-conscious defaults |
| Apple Notes bridge | not directly ported | Later | replace only through generic Linux context-provider design if justified |
| Quick Window | deferred | Later | evaluate after core parity |
| Sparkle update flow | removed | P3 | use Linux packaging/update channels |
| DMG/codesign/notarization | removed | P3 | Flatpak/RPM/DEB/release signing instead |
| macOS notification APIs | Linux desktop notification path | P2 | prefer standard desktop integration |
| macOS privacy prompts | WebKitGTK + Linux portal/native permissions | P0/P1 | Wayland/PipeWire must be tested |

## Parity policy

A feature is not considered ported merely because an analogous API exists. Parity requires observable behavior to match the useful upstream contract and to pass the Linux acceptance criteria defined by its issue.

When upstream Swift behavior is macOS-specific, preserve the user-facing intent rather than copying the mechanism.
