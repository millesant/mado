# Building Mado on Linux

Mado's initial supported development baseline is Fedora Linux 44 on x86_64.

## Toolchain

- Rust 1.93 or newer
- Cargo
- `pkg-config`
- GTK 4.10 or newer development files
- WebKitGTK 6.0 development files

On Fedora 44:

```bash
sudo dnf install rust cargo pkgconf-pkg-config gtk4-devel webkitgtk6.0-devel
```

## Build

From the repository root:

```bash
cargo check
cargo test
```

To launch the shell:

```bash
cargo run
```

A successful manual smoke test opens a native GTK4 window with the current ChatGPT page rendered by WebKitGTK 6.

## Development profile selection

Until the native profile UI is implemented, select a profile for manual isolation testing with `MADO_PROFILE`:

```bash
MADO_PROFILE=default cargo run
MADO_PROFILE=profile-b cargo run
```

Profile IDs are lowercase ASCII path-safe identifiers using letters, digits, `-`, and `_`, with a maximum length of 64 characters. If `MADO_PROFILE` is unset, Mado uses `default`.

P1 profile storage starts in new per-profile XDG directories. The earlier P0 test store at the Mado data/cache root is deliberately not copied automatically, so the first P1 run of `default` may require signing in again. The legacy files are left untouched rather than copied while a WebKit helper might still hold them open.

Other Linux distributions may work, but this phase only claims the Fedora baseline after it is actually validated.

## GTK binding baseline

Mado enables the gtk4-rs `v4_10` feature explicitly. The current `webkit6 0.6.x`
bindings reference GTK accessibility APIs that gtk4-rs gates behind GTK 4.10,
so lowering this feature requires re-validating the WebKitGTK binding combination.
