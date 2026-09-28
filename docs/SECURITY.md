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

Prefer WebKit-managed persistent sessions.
Do not extract, serialize, log, or manually shuttle authentication cookies unless a narrowly scoped feature explicitly requires it and receives a separate security review.

Deleting a profile must delete only that profile's owned data.

## Navigation

Maintain an explicit trusted-origin policy for ChatGPT/OpenAI authentication flows.

WebKit new-window requests may create an in-app child view only when the opener is an HTTPS ChatGPT/OpenAI origin. The initial child target must be HTTPS (or `about:blank` while WebKit bootstraps the popup), and the child must use WebKit's related-view relationship so authentication state remains WebKit-owned and shared with its opener.

Ordinary external links should open through the system browser unless an issue deliberately defines in-app handling.

Unknown custom schemes must not be blindly launched.

## JavaScript/native bridge

Bridge messages require:
- an explicit message schema;
- origin/context validation where the platform allows it;
- size limits;
- strict command allow-listing;
- no arbitrary filesystem paths or shell execution.

DOM selectors and injected scripts live in the dedicated web-integration layer.

## Downloads

Treat filenames, MIME types, sizes, and bytes as untrusted.
Normalize/sanitize suggested filenames.
Prevent path traversal.
Use bounded buffering/chunking.
Write through temporary files and finalize atomically where practical.
Do not automatically execute/open downloaded files.

## Permissions

Microphone/camera requests are allowed only for explicitly trusted ChatGPT/OpenAI origins and require user-visible permission behavior.

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
