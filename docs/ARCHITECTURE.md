# Mado architecture

## Goal

Mado is a Linux-only native desktop shell for `https://chatgpt.com` that preserves web feature parity without Electron/Chromium/Tauri.

Primary stack:
- Rust
- Relm4
- GTK4
- WebKitGTK 6

Wayland is first-class. X11 compatibility is maintained where reasonable.

## Architectural principles

1. Keep the ChatGPT web surface inside WebKitGTK rather than rebuilding ChatGPT as a native API client.
2. Keep Linux-native behavior outside the page: application/window lifecycle, profiles, permissions, downloads, notifications, diagnostics, settings, and packaging.
3. Isolate volatile ChatGPT DOM knowledge behind one web-integration layer.
4. Keep authentication/session state under WebKit ownership wherever practical.
5. Treat profiles as security/privacy boundaries.
6. Prefer explicit small modules over framework-heavy abstraction.
7. Do not duplicate workflow state in repository files.

## Target layout

```text
src/
├── main.rs
├── application.rs
├── browser/
│   ├── mod.rs
│   ├── window.rs
│   ├── navigation.rs
│   ├── permissions.rs
│   ├── recovery.rs
│   └── state.rs
├── web/
│   ├── mod.rs
│   ├── bridge.rs
│   ├── selectors.rs
│   └── scripts/
├── profiles/
│   ├── mod.rs
│   ├── model.rs
│   └── storage.rs
├── downloads/
├── notifications/
├── settings/
├── diagnostics/
├── privacy/
└── platform/
    ├── xdg.rs
    └── portals.rs

data/
├── io.github.Millesant.Mado.desktop
├── io.github.Millesant.Mado.metainfo.xml
└── icons/

packaging/
├── flatpak/
├── rpm/
└── deb/

tests/
```

The exact module split may evolve, but the boundaries above should remain recognizable.

## Runtime boundaries

### GTK/Relm4 shell
Owns application activation, native windows, menus/actions, settings surfaces, dialogs, profile switcher, and desktop integration.

### WebKitGTK
Owns page rendering, cookies, local storage, IndexedDB, service workers, media/WebRTC, and normal browser networking.

### Web integration
`src/web/` owns all injected JavaScript, DOM selectors, page-state observation, draft/completion hooks, and browser/native message contracts. No other subsystem should directly depend on ChatGPT DOM structure.

The P2 bridge uses one WebKit isolated script world (`mado-web-integration-v1`) and one named message handler. The bootstrap script is injected at document start into the top frame only, and WebKit's URI allow-list restricts injection to HTTPS ChatGPT/OpenAI origins. Related OAuth child WebViews share the parent's `UserContentManager`; third-party provider pages do not match the injection allow-list.

The bridge protocol is explicitly versioned. JavaScript sends JSON strings with `v`, `type`, and a type-specific `payload`. Rust rejects messages larger than 16 KiB before JSON parsing, rejects unknown versions/types, and deserializes each payload into a closed schema. The bridge is an event/data channel, not a generic native command dispatcher.

`bootstrap.js` exposes only isolated-world helpers for later feature scripts: `pageKind()`, `observersAllowed()`, and bounded `post()`. DOM-dependent observers must check `observersAllowed()`; it becomes false on Cloudflare challenge pages and auth-like pages. The Cloudflare selector is centralized in `web/selectors.rs`.

Injected scripts live as separate files under `web/scripts/` so their source and protocol version can be checked independently from native code.

### Profiles
Each profile receives an isolated persistent WebKit data/cache boundary. Switching profiles must never merge cookie/local-storage state.

### Downloads
`downloads/` owns the native download model and status surface. Each profile's WebKit `NetworkSession` feeds its `download-started` events into that controller.

On the tested WebKitGTK 2.54 baseline, normal HTTP(S) transfers use WebKit's native `Download` stream. Real ChatGPT generated files appear first as trusted, user-initiated `blob:` navigation policy decisions; Mado explicitly converts those decisions (and equivalent trusted `data:` decisions) into WebKit downloads. This keeps bytes inside WebKit's streaming path and avoids a JavaScript/base64 synthetic-download bridge. If a future WebKitGTK/ChatGPT combination requires such a bridge, it must live behind the web-integration boundary and use bounded/chunked messages rather than one giant payload.

Server-suggested filenames are treated as untrusted. Mado sanitizes them, reserves unique final destinations in the user's Downloads directory, and gives WebKit a hidden sibling `.part` destination. Only WebKit's successful `finished` path promotes the partial file to its final name; failures discard the partial file. Finalization uses a no-clobber same-directory promotion so a file created by another process during the download is never overwritten. Download progress/completion/failure are reported through a small native GTK bottom status bar that does not cover the WebView. Downloaded files are never automatically executed or opened.

### Diagnostics
Diagnostics are read-only and sanitized by default. Never include cookies, session tokens, prompt text, or other sensitive page contents unless an explicit future feature defines safe opt-in behavior.

## XDG storage

Use the XDG base directory specification:
- config: `$XDG_CONFIG_HOME/mado` or `~/.config/mado`
- data: `$XDG_DATA_HOME/mado` or `~/.local/share/mado`
- state: `$XDG_STATE_HOME/mado` or `~/.local/state/mado`
- cache: `$XDG_CACHE_HOME/mado` or `~/.cache/mado`

Per-profile WebKit data/cache must live beneath profile-specific directories.

The P1 profile layout is:

```text
$XDG_CONFIG_HOME/mado/profiles/<id>/
$XDG_DATA_HOME/mado/profiles/<id>/
├── cookies.sqlite
└── webkit/
$XDG_STATE_HOME/mado/profiles/<id>/
$XDG_CACHE_HOME/mado/profiles/<id>/
└── webkit/
```

Each profile receives its own persistent WebKit `NetworkSession` using that profile's data/cache roots. Profile IDs are validated before they are used in paths, and profile deletion removes only the four directories owned by that profile.

## Application lifecycle and window state

Mado uses GTK/GApplication uniqueness through Relm4 rather than a secondary lockfile. A second launch activates and presents the existing application window.

Main-window state is profile-scoped at `$XDG_STATE_HOME/mado/profiles/<id>/window-state`. Persist only width, height, and maximized state. Do not persist or restore absolute window coordinates: Wayland compositors own placement and do not provide a portable absolute-position contract.

## WebKit recovery

`browser/recovery.rs` owns native recovery state for main and related child WebViews.

Recovery is intentionally bounded:
- cancelled loads are ignored and do not surface stock WebKit error pages;
- an ordinary navigation/load failure shows a native recovery banner and waits for an explicit user reload;
- recovery targets the last successfully committed HTTP(S) page; a provisional URI that failed before commit is never promoted to the retry target;
- an abnormal WebProcess termination gets at most one automatic reload of that last committed page (falling back to the ChatGPT home URL);
- if that recovery does not finish successfully before another process termination/failure, automatic recovery stops and the user must choose Reload;
- a successful recovery resets the one-shot WebProcess retry budget;
- Mado does not watch for network restoration and does not automatically loop retries;
- recovery only reloads page URLs. It never injects, replays, or submits prompt text.

WebKitGTK emits `load-changed(Finished)` immediately after `load-failed`; the state machine therefore preserves a failed state across that terminal event instead of treating it as success.

## Threading

GTK/Relm4/WebKit objects stay on the GLib main context unless their API explicitly supports otherwise.

Blocking filesystem or expensive CPU work must not stall the UI thread. Use bounded background work and return results through message channels/main-context dispatch.

## Compatibility strategy

Before polishing the shell, prove that WebKitGTK can reliably support:
- ChatGPT page load
- login/session persistence
- OAuth/new-window flows
- Cloudflare challenges
- file upload
- generated downloads
- microphone/camera/voice
- Wayland and X11

The compatibility gate is tracked in `docs/WEBKIT_COMPAT.md`.

## Distribution

Primary distribution target: Flatpak.
Additional targets: Fedora/RPM and Debian/DEB.

Packaging must not silently bundle Chromium or switch browser engines.
