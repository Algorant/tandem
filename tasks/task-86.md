---
id: task-86
uid: 4f5af325-754e-4c25-9a3e-7880966c7125
type: task
kind: "papercut"
title: "Stop continuous terminal writes from an idle TUI"
state: todo
priority: "high"
effort: "medium"
relatedFiles: ["tandem/src/tui/mod.rs", "tandem/src/tui/state.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/chrome.rs", "tandem/src/app/background_sync.rs"]
tags: ["tui", "rendering"]
accord:
  status: "ready"
  acceptance: ["Identify the concrete idle output cause and whether ten-second wchar/syscw deltas scale with board/log size; report actual before/after rates using release builds.", "A static idle TUI writes approximately nothing, rendering on visible state/input changes and only a justified slow clock tick if needed; no full-frame output or spinner churn when unchanged.", "Input/navigation, resize, local-file reload, active-board sync/status/error changes and visible chooser summary refresh remain correct and responsive.", "Parent independently verifies /proc/<pid>/io wchar and syscw deltas over ten seconds on safe representative boards before/after, native checks pass, and temporary harnesses/fixtures are removed."]
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Algorant requested immediate bug fix via Herdra; no mockup or unrelated layout/feature changes.", "Plan first, explicit Luna/high; isolated Task worktree. Observe baseline before source edits, with no mutation of real .pi records/event history.", "Use release builds and deterministic PTY/output measurements, not debug timing. Measure ten-second wchar/syscw and report byte/s + calls/s; preserve terminal dimensions/theme/input conditions.", "No general rendering framework, permanent tests/fixtures, push or release without separate authorization. Keep Rules task53 mockup lane separate."]
  updatedAt: "2026-10-17T03:25:22Z"
createdAt: "2026-10-17T03:25:22Z"
updatedAt: "2026-10-17T03:25:22Z"
---

## Description

Algorant/Herdra measured static desktop .pi Board TUI pid3432147 at~1744 writes/s and~2.9MB/s,118GB over10.5h; five otherTUIs70–580KB/s. OriginalPID has now exited. Diagnose actual redraw/timer/sync/spinner/fingerprint cause; don't assume every draw implies terminal writes because Ratatui should diff unchanged frames. Check size dependence including a board with~526logs. Fix to render only when visible state changes/input, with justified slowclock only.

Use safe disposable Git-backed board snapshots or generated data, never mutate realboard/history; read-only existing process IO sampling is allowed. Trace event loop, redraw invalidation, screen/terminal clearing and reload/backgroundsync. First concrete plan must name suspected path and reproducible baseline measurement, then parent approves code. Verify output while idle and actual visible changes, plus nativefmt/tests/strictclippy. No release/push authorized in this request.
