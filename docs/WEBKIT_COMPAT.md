# WebKitGTK compatibility gate

The Linux port does not proceed to broad feature work until the WebKitGTK 6 path is proven viable for current ChatGPT Web behavior.

This document records durable compatibility findings. GitHub Issues track active work.

## Test environments

Record:
- distribution/version;
- GTK version;
- WebKitGTK version;
- display backend (Wayland/X11);
- desktop environment;
- Flatpak vs host build when relevant.

## Gate matrix

| Capability | Required for P0 exit | Current state | Evidence |
|---|---:|---|---|
| launch native GTK/Relm4 shell | yes | Unverified | |
| load `https://chatgpt.com` | yes | Unverified | |
| interactive login | yes | Unverified | |
| session persists after restart | yes | Unverified | |
| Google/Apple/Microsoft OAuth/new-window flow | yes | Unverified | |
| Cloudflare challenge can complete without loops | yes | Unverified | |
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
