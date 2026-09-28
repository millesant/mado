# WebKitGTK compatibility gate

The Linux port does not proceed to broad feature work until the WebKitGTK 6 path is proven viable for current ChatGPT Web behavior.

This document records durable compatibility findings. GitHub Issues track active work.

## Test environments

### Fedora 44 host smoke — issue #5

- Fedora Linux 44, x86_64 host build
- GTK 4.22.5
- WebKitGTK 2.54.0
- KDE on Wayland
- Manual evidence: native Mado window opened, `https://chatgpt.com/` rendered to the signed-out ChatGPT UI, and closing the window returned to the shell with no lingering Mado/WebKit helper process.
- One credential-socket `Broken pipe` warning was observed during shutdown on this host; after commit `1de52c2`, the repeated WebKit `internallyFailedLoadTimerFired()` shutdown errors no longer occurred.

## Gate matrix

| Capability | Required for P0 exit | Current state | Evidence |
|---|---:|---|---|
| launch native GTK/Relm4 shell | yes | Verified | Fedora 44 host smoke above; clean process exit after `1de52c2` |
| load `https://chatgpt.com` | yes | Verified | Fedora 44 host smoke above; current signed-out ChatGPT UI rendered in WebKitGTK 6 |
| interactive login | yes | Verified | Fedora 44 smoke; Google sign-in completed |
| session persists after restart | yes | Verified | Fedora 44 smoke; signed-in state remained across multiple restarts |
| Google/Apple/Microsoft OAuth/new-window flow | yes | Verified | Google flow completed in an app-owned related child WebView |
| Cloudflare challenge can complete without loops | yes | Verified | Challenge completed successfully before Google sign-in |
| basic chat send/stream/render | yes | Unverified | |
| file upload | yes | Unverified | |
| normal file download | yes | Unverified | |
| generated `blob:`/`data:` download feasibility | yes | Unverified | |
| microphone permission | yes | Unverified | |
| voice input / relevant WebRTC path | yes | Unverified | |
| camera permission where ChatGPT requests it | no | Unverified | |
| external links can hand off to system browser | yes | Unverified | |
| Wayland smoke | yes | Unverified | |
| X11 smoke | yes | Unverified | |
| page recovery after WebKit/load failure | no | Unverified | |

## P0 exit rule

P0 may exit when every required row is either:
- verified working with reproducible evidence; or
- documented as a known limitation with an accepted product decision and an issue describing the fallback/mitigation.

Do not mark a row verified from API availability alone. It requires an actual build/test result on Linux.
