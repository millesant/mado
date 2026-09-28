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

### Fedora 44 transfer smoke — issue #7

- Same Fedora 44 / GTK 4.22.5 / WebKitGTK 2.54.0 / KDE Wayland host.
- A local text file selected through WebKitGTK's native file chooser uploaded successfully to ChatGPT and was readable by the model.
- A ChatGPT-generated text file downloaded successfully to the user's normal Downloads directory using WebKitGTK's built-in download behavior.
- Temporary scheme-only instrumentation confirmed the real ChatGPT-generated download used a `blob:` URL and completed successfully.
- A temporary local compatibility probe confirmed an explicit `data:` download also completed successfully and wrote the expected file contents.
- No native synthetic-download bridge is required for the tested `blob:` / `data:` paths. Final download progress/history UI remains out of scope for this compatibility gate.

### Fedora 44 media smoke — issue #8

- Same Fedora 44 / GTK 4.22.5 / WebKitGTK 2.54.0 / KDE Wayland host.
- ChatGPT microphone capture triggered Mado's native permission prompt and worked after explicit approval; spoken input was transcribed/sent successfully before the experimental WebRTC toggle was tried.
- Full ChatGPT Voice mode failed with `Voice couldn't connect`.
- WebKitGTK 2.54 defaults `enable-media-stream` to true and `enable-webrtc` to false. Explicitly enabling `enable-webrtc` did not make full Voice connect and regressed the working dictation path, so that change was reverted.
- Upstream WebKitGTK 2.54 release notes state that WebRTC support is disabled in 2.54 while the GStreamer backend is being replaced by a LibWebRTC implementation expected in the next release cycle. This explains why forcing the runtime setting cannot make ChatGPT Voice viable on this build. Reference: https://webkitgtk.org/2026/09/16/webkitgtk-2.54-highlights.html
- The host exposes GStreamer WebRTC, Opus, SCTP, DTLS, and PipeWire audio elements. The remaining full-Voice incompatibility is tracked in #26.
- No reachable camera flow was exercised in this smoke test. Media grants are not persisted by Mado in P0, so a later request/restart can prompt again.

### Fedora 44 display smoke — issue #9

- KDE Wayland: normal authenticated ChatGPT use across issues #6–#8 exercised prompt entry, streamed responses, follow-up interactions, uploads, downloads, and microphone dictation without immediate layout/input corruption.
- X11 compatibility was exercised through KDE's XWayland server (`DISPLAY=:0`) by forcing `GDK_BACKEND=x11`.
- The XWayland smoke rendered a 20-item streamed response followed by a second summarization turn with normal typing, clicking, scrolling, and layout.
- An initial process sample during XWayland page startup showed one WebKit web process briefly using about 81% CPU and 768 MiB RSS; no sustained rendering/performance blocker was observed during the completed interaction.

### Fedora 44 external-link smoke — issue #27

- Same Fedora 44 / KDE Wayland host.
- A user-initiated third-party HTTPS link from ChatGPT was handed to the Linux default browser through GIO; the browser reported that it opened the URL in the existing browser session.
- Mado remained on ChatGPT and did not create a child WebView for the third-party link after commit `22a97cf`.
- Trusted ChatGPT/OpenAI destinations remain embedded; known OAuth providers may still use a related child WebView. Unknown/custom schemes are denied.

### Known non-gating runtime defect — issue #24

- Authenticated shutdown can still produce delayed WebKitGTK 2.54 `internallyFailedLoadTimerFired()` stderr output after the native Mado process returns to the shell.
- Multiple bounded Mado-side teardown attempts did not remove the symptom. #24 remains open for upstream/runtime diagnosis and is not treated as a P0 viability blocker because launch, authenticated use, persistence, and process exit are otherwise functional.

## Gate matrix

| Capability | Required for P0 exit | Current state | Evidence |
|---|---:|---|---|
| launch native GTK/Relm4 shell | yes | Verified | Fedora 44 host smoke above; clean process exit after `1de52c2` |
| load `https://chatgpt.com` | yes | Verified | Fedora 44 host smoke above; current signed-out ChatGPT UI rendered in WebKitGTK 6 |
| interactive login | yes | Verified | Fedora 44 smoke; Google sign-in completed |
| session persists after restart | yes | Verified | Fedora 44 smoke; signed-in state remained across multiple restarts |
| Google/Apple/Microsoft OAuth/new-window flow | yes | Verified | Google flow completed in an app-owned related child WebView |
| Cloudflare challenge can complete without loops | yes | Verified | Challenge completed successfully before Google sign-in |
| basic chat send/stream/render | yes | Verified | Fedora 44 display smoke; multi-turn streamed responses rendered correctly on Wayland and XWayland |
| file upload | yes | Verified | Fedora 44 transfer smoke; local text file uploaded and was readable by ChatGPT |
| normal file download | yes | Verified | Fedora 44 transfer smoke; generated text file saved to the normal Downloads directory |
| generated `blob:`/`data:` download feasibility | yes | Verified | Real ChatGPT `blob:` download and explicit `data:` probe both completed directly through WebKitGTK |
| microphone permission | yes | Verified | Fedora 44 media smoke; native Mado prompt appeared and approved microphone capture worked |
| voice input / relevant WebRTC path | yes | Known limitation | Basic dictation worked; full ChatGPT Voice mode could not connect on WebKitGTK 2.54.0 and is tracked in #26 |
| camera permission where ChatGPT requests it | no | Not exercised | No reachable camera flow was exposed during the issue #8 smoke test |
| external links can hand off to system browser | yes | Verified | Fedora 44 external-link smoke; third-party HTTPS opened in the existing system browser session while Mado stayed on ChatGPT |
| Wayland smoke | yes | Verified | Fedora 44 KDE Wayland host used throughout P0 interactive testing without immediate chat layout/input corruption |
| X11 smoke | yes | Verified | Fedora 44 KDE XWayland (`GDK_BACKEND=x11`, `DISPLAY=:0`) multi-turn chat smoke passed |
| page recovery after WebKit/load failure | no | Unverified | |

## P0 exit rule

P0 may exit when every required row is either:
- verified working with reproducible evidence; or
- documented as a known limitation with an accepted product decision and an issue describing the fallback/mitigation.

Do not mark a row verified from API availability alone. It requires an actual build/test result on Linux.
