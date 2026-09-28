# Security model

Mado embeds a privileged authenticated web session. Browser/native integration is therefore security-sensitive even though Mado is a desktop shell.

## Trust boundaries

### Web content
Treat page content, DOM values, URLs, downloads, and JavaScript-originated bridge messages as untrusted input.

### Native shell
Native code may access filesystem, desktop services, notifications, and application state. Never expose unrestricted native commands to page JavaScript.

### Profiles
Profiles are isolation boundaries. Cookies, local storage, IndexedDB, service workers, caches, and related WebKit data must not leak across profiles.

## Authentication and cookies

Prefer WebKit-managed persistent sessions. Each P1 profile owns a distinct WebKit `NetworkSession`, profile-specific data/cache directories, and a SQLite cookie store at `$XDG_DATA_HOME/mado/profiles/<id>/cookies.sqlite` (with the usual XDG fallback). Native code configures these paths but does not read cookie contents.

Profile IDs are validated as canonical path-safe identifiers before filesystem use. Do not extract, serialize, log, or manually shuttle authentication cookies unless a narrowly scoped feature explicitly requires it and receives a separate security review.

Deleting a profile removes only that profile's config/data/state/cache directories. The legacy P0 root-level WebKit store is not automatically copied into a P1 profile because an old WebKit helper may still hold it open during startup.

## Navigation

Maintain an explicit trusted-origin policy for ChatGPT/OpenAI authentication flows.

WebKit new-window requests may create an in-app child view only when the opener is an HTTPS ChatGPT/OpenAI origin and the initial target is `about:blank`, a trusted ChatGPT/OpenAI destination, or an explicitly allowed authentication provider (Google, Apple, or Microsoft). Other third-party HTTPS new-window requests are handed to the Linux default browser. The child must use WebKit's related-view relationship so authentication state remains WebKit-owned and shared with its opener.

User-initiated third-party HTTPS navigations from a trusted ChatGPT/OpenAI page are handed to the Linux default browser through GIO. Non-user HTTP(S) redirects remain embedded so authentication flows can continue, and trusted ChatGPT/OpenAI destinations stay in-app.

Unknown/custom schemes are denied rather than blindly launched.

## JavaScript/native bridge

Bridge messages require:
- an explicit message schema;
- origin/context validation where the platform allows it;
- size limits;
- strict command allow-listing;
- no arbitrary filesystem paths or shell execution.

DOM selectors and injected scripts live in the dedicated web-integration layer.

## Uploads and downloads

File chooser requests are allowed only from an HTTPS ChatGPT/OpenAI page. Trusted requests use WebKitGTK's native GTK chooser; requests from other current page origins are cancelled.

Treat filenames, MIME types, sizes, and bytes as untrusted.

WebKit download suggestions never become filesystem paths directly. Mado removes path/control syntax, bounds filename length by UTF-8 bytes, reserves a unique destination under the user's Downloads directory, and downloads into a hidden same-directory `.part` file first. A failed/cancelled transfer removes the partial file; only successful completion renames it to the final filename.

The tested WebKitGTK runtime streams HTTP(S) downloads through the native download API. Trusted ChatGPT/OpenAI pages may also convert a real user-initiated `blob:` or `data:` navigation policy decision into a WebKit download; the same schemes are denied without both the trusted source and user gesture. Mado therefore does not base64-serialize synthetic downloads. If a future fallback bridge becomes necessary, it must be bounded/chunked and stay behind the web-integration boundary.

Successful finalization is no-clobber: if another process creates the reserved final path during the transfer, Mado fails safely rather than overwriting that file. Downloaded files are never auto-executed or automatically launched.

## Permissions

Microphone/camera requests are handled only while the current WebView is on an explicitly trusted HTTPS ChatGPT/OpenAI origin. Every WebKit user-media request still requires an explicit native GTK confirmation; Mado does not blanket-grant capture access.

WebKitGTK 6's public `UserMediaPermissionRequest` API reports whether audio and/or video was requested but does not expose the exact requesting subframe origin. The P0 policy therefore validates the top-level WebView origin and keeps the user confirmation as a second boundary. If a future WebKitGTK API exposes the initiating security origin, prefer that more precise check.

Prefer desktop portals for host integration where appropriate, especially under Flatpak/Wayland.

## Diagnostics and logs

Default diagnostics must not include:
- cookies/session tokens;
- authorization headers;
- prompt/response text;
- raw localStorage/IndexedDB;
- arbitrary page HTML;
- secrets from environment variables.

Logs must be useful without exposing authenticated session material.

## Dependency and release hygiene

Keep Rust dependencies minimal and auditable.
Pin/review security-sensitive browser/bridge dependencies.
CI artifacts must come from the repository revision being released.
Release instructions must not claim signing or verification that was not actually performed.
