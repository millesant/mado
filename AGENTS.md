# Mado agent instructions

Mado is a Linux-only native ChatGPT web client.

## Fixed product direction

Use:
- Rust
- Relm4
- GTK4
- WebKitGTK 6

Do not introduce Electron, Chromium, Tauri, Qt WebEngine, or another bundled browser runtime unless the user explicitly changes the product direction.

The upstream `swift/` implementation is the primary behavioral reference. Reimplement behavior idiomatically for Linux; do not mechanically translate Swift.

Until an explicit cleanup issue says otherwise, keep `swift/` and `tauri/` as reference material.

## Sources of truth

Authority order:
1. Current user instruction.
2. Accepted repository requirements and architecture.
3. The active GitHub issue.
4. This file and scoped repository instructions.
5. Existing implementation and tests.
6. Upstream Swift behavior.
7. External documentation/research.

GitHub Issues are the work-unit source of truth. The GitHub Project board is the workflow-state source of truth. Repository docs are the durable architecture/decision source of truth.

Do not create secondary task trackers, progress JSON, mirrored TODO lists, hidden queues, or agent-state files.

## Work model

Work on one implementation issue at a time unless broader planning is explicitly requested.

For implementation work:
1. Identify the active issue and exact repository revision.
2. Read only files/docs materially relevant to that issue.
3. Inspect the corresponding Swift code when parity matters.
4. Implement the smallest coherent solution.
5. Add or update tests.
6. Update docs when behavior, architecture, interfaces, security boundaries, or non-obvious invariants change.
7. Run the smallest sufficient validation.
8. Commit/push according to the active workflow.
9. Update the issue with durable handoff information.
10. Stop at a clean boundary.

Do not expand scope because adjacent work is discovered; create/reference another issue instead.

## Context and tool discipline

Do not read the whole repository by default.
Prefer targeted file reads/searches over repository-wide dumps.
Batch independent reads where practical.
Do not dump large raw connector responses, CI logs, generated JSON, or dependency trees into reasoning context when a bounded subset is enough.

The chat is not the project database. Durable state belongs in GitHub.

## Blocking and CI

Do not remain indefinitely in research, thinking, debugging, CI polling, or retry loops.

If blocked after one reasonable bounded investigation:
- preserve completed work;
- record the concrete blocker and smallest useful evidence in the issue;
- mark/recommend Blocked;
- stop.

After pushing, inspect CI once when useful. If still running, record the commit SHA and stop. Do not poll repeatedly. Never claim CI is green unless it completed successfully for the relevant revision.

## Porting rules

Maintain `docs/PORTING_MATRIX.md` for architectural and behavioral parity only. It must not duplicate issue/board status.

All ChatGPT DOM assumptions and injected JavaScript belong behind a dedicated web-integration boundary. Treat DOM structure as volatile.

Authentication cookies, session tokens, credentials, and sensitive WebKit data should remain owned by WebKit/session storage wherever practical. Never log secrets.

Profiles must be isolated. Navigation, OAuth, permissions, downloads, microphone/camera access, custom schemes, and JavaScript/native bridges are security boundaries.

Use XDG locations correctly. Prefer Linux desktop standards and portals where appropriate. Avoid speculative abstraction and dependencies that save only trivial code.
