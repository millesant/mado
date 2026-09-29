<p align="center">
  <strong>English</strong> · <a href="./README.pt-BR.md">Português (Brasil)</a>
</p>

<h1 align="center">Mado</h1>

<p align="center">
  <strong>A native Linux desktop client for ChatGPT Web.</strong><br>
  Rust · Relm4 · GTK 4 · WebKitGTK 6
</p>

> [!IMPORTANT]
> Mado is under active development. There is no packaged stable release yet; current use is from source.

## What is Mado?

Mado keeps the real ChatGPT web experience inside a small Linux-native application instead of rebuilding ChatGPT as an API client or shipping a bundled Chromium runtime.

- **Rust + Relm4**
- **GTK 4**
- **WebKitGTK 6**
- **Wayland first**, with X11/XWayland compatibility where practical.
- **XDG-native storage** for application and profile state.

Mado does **not** use Electron, Tauri, Qt WebEngine, or a bundled Chromium runtime.

## Current status

| Area | Status |
| --- | --- |
| Native GTK/Relm4 shell | Working |
| Login + persistent WebKit session | Working |
| Isolated persistent profiles | Engine complete; UI planned |
| Trusted navigation + external-browser handoff | Working |
| Uploads + native normal/generated downloads | Working |
| Single instance + window-state persistence | Working |
| Bounded WebKit recovery | Working |
| Local unsent-draft recovery | Working |
| Completion notifications | Planned in [#17](https://github.com/millesant/mado/issues/17) |
| Profile/settings UI | Planned in [#18](https://github.com/millesant/mado/issues/18) |
| Sanitized diagnostics/privacy controls | Planned in [#19](https://github.com/millesant/mado/issues/19) |
| Full ChatGPT Voice on WebKitGTK 2.54 | Runtime limitation; [#26](https://github.com/millesant/mado/issues/26) |

See [WebKitGTK compatibility](./docs/WEBKIT_COMPAT.md) for tested runtime details.

## Build and run

The validated development baseline is Fedora Linux 44.

    sudo dnf install rust cargo pkgconf-pkg-config gtk4-devel webkitgtk6.0-devel

    git clone https://github.com/millesant/mado.git
    cd mado
    cargo run

More details: [docs/BUILDING.md](./docs/BUILDING.md).

## Privacy and security

Mado treats the authenticated WebKit session as a security boundary.

- Authentication cookies and sensitive browsing state remain owned by WebKit wherever practical.
- Profiles use separate XDG-backed WebKit data/cache/session storage.
- Third-party links follow an explicit native navigation policy.
- JavaScript/native messages are typed, versioned, isolated, and bounded.
- Downloads use sanitized names, partial files, and no-clobber finalization.
- Raw cookies, tokens, prompt contents, and page HTML are excluded from the diagnostics design.

Local draft recovery stores an additional profile-scoped copy of unsent composer text in Mado’s XDG state. Restore is explicit and never auto-sends. A user-facing toggle belongs to the upcoming settings work.

Read [docs/SECURITY.md](./docs/SECURITY.md).

## Repository layout

    src/                Linux application code
    docs/               Architecture, security, build, and compatibility notes
    scripts/            Repository validation helpers
    reference/swift/    Read-only upstream Swift references for remaining parity work
    .github/workflows/  Linux CI

The reference/swift directory is not a macOS product or build target. It contains only behavioral references still useful for #17–#19.

## Documentation

- [Architecture](./docs/ARCHITECTURE.md)
- [Building](./docs/BUILDING.md)
- [Security model](./docs/SECURITY.md)
- [WebKitGTK compatibility](./docs/WEBKIT_COMPAT.md)
- [Porting matrix](./docs/PORTING_MATRIX.md)

## Project status

Mado is in **P2 parity work**. Packaging/distribution is not finalized yet; the future direction is standard Linux packaging without a bundled browser runtime.

## Disclaimer

Mado is an independent open-source project and is not affiliated with, endorsed by, or sponsored by OpenAI.
