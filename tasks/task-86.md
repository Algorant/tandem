---
id: task-86
uid: 4f5af325-754e-4c25-9a3e-7880966c7125
type: task
kind: "papercut"
title: "Diagnose and eliminate unnecessary idle TUI I/O"
state: "in-progress"
priority: "medium"
effort: "medium"
relatedFiles: ["tandem/src/tui/mod.rs", "tandem/src/tui/state.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/chrome.rs", "tandem/src/app/background_sync.rs"]
tags: ["tui", "rendering"]
accord:
  status: "claimed"
  acceptance: ["Attribute ten-second process-wide wchar/syscw deltas to actual write destinations and waited-for child accounting; identify the concrete unnecessary idle-I/O cause and board/log-size dependence before source edits.", "Remove the proven unnecessary idle work while preserving input, resize, local reload, background sync/status and chooser refresh; no speculative redraw patch when PTY is already quiet.", "Parent verifies comparable ten-second before/after wchar/syscw plus segmented PTY byte counts on safe small and ~526-log boards, native checks pass and temporary harnesses/fixtures are removed."]
  claimedAt: "2026-10-17T03:25:52Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Algorant requested immediate bug fix via Herdra; no mockup or unrelated layout/feature changes.", "Plan first, explicit Luna/high; isolated Task worktree. Observe baseline before source edits, with no mutation of real .pi records/event history.", "Use release builds and deterministic PTY/output measurements, not debug timing. Measure ten-second wchar/syscw and report byte/s + calls/s; preserve terminal dimensions/theme/input conditions.", "No general rendering framework, permanent tests/fixtures, push or release without separate authorization. Keep Rules task53 mockup lane separate."]
  updatedAt: "2026-10-17T03:30:30Z"
createdAt: "2026-10-17T03:25:22Z"
updatedAt: "2026-10-17T03:30:30Z"
assignee: "worker-task-86-5004cbf6"
---

## Description

Algorant/Herdra measured static desktop .pi Board TUI pid3432147 at~1744 writes/s and~2.9MB/s,118GB over10.5h; five otherTUIs70–580KB/s. OriginalPID has now exited. Diagnose actual redraw/timer/sync/spinner/fingerprint cause; don't assume every draw implies terminal writes because Ratatui should diff unchanged frames. Check size dependence including a board with~526logs. Fix to render only when visible state changes/input, with justified slowclock only.

Use safe disposable Git-backed board snapshots or generated data, never mutate realboard/history; read-only existing process IO sampling is allowed. Trace event loop, redraw invalidation, screen/terminal clearing and reload/backgroundsync. First concrete plan must name suspected path and reproducible baseline measurement, then parent approves code. Verify output while idle and actual visible changes, plus nativefmt/tests/strictclippy. No release/push authorized in this request.


## Measurement correction from Herdra

Original rates were /proc/PID/io totals, NOT per-fd or confirmed pty writes. Herdr rchar across allpanes~67KB/s and itsCPU unchanged whenTUIsSIGSTOPed, so no evidence terminal is churning. write_bytes145MB vswchar118GB. LiveTandem hasstdio→pty, eventpoll andconnectedAF_UNIXsocketpair (bothends owned bysameprocess). Herdra suspects internalwake/channel loop; investigate socketpair purpose, but test attribution rather than assume. LinuxprocessIO may include waited-forchildren, which a staticFDtable misses. Normal-priority performancefix, noturgent. RootWorker remains diagnostic/read-only before concrete cause+minimalimplementationplan approval; no guesses/syncdisable/renderframework.