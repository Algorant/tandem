---
id: task-86
uid: 4f5af325-754e-4c25-9a3e-7880966c7125
type: task
kind: "papercut"
title: "Diagnose and eliminate unnecessary idle TUI I/O"
priority: "medium"
effort: "medium"
relatedFiles: ["tandem/src/app/background_sync.rs", "tandem/src/project/sync.rs"]
tags: ["tui", "rendering"]
accord:
  status: "accepted"
  acceptance: ["Attribute ten-second process-wide wchar/syscw deltas to actual write destinations and waited-for child accounting; identify the concrete unnecessary idle-I/O cause and board/log-size dependence before source edits.", "Remove the proven unnecessary idle work while preserving input, resize, local reload, background sync/status and chooser refresh; no speculative redraw patch when PTY is already quiet.", "Parent verifies comparable ten-second before/after wchar/syscw plus segmented PTY byte counts on safe small and ~526-log boards, native checks pass and temporary harnesses/fixtures are removed."]
  claimedAt: "2026-10-17T03:25:52Z"
  deliveredAt: "2026-10-17T04:34:01Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Algorant requested immediate bug fix via Herdra; no mockup or unrelated layout/feature changes.", "Plan first, explicit Luna/high; isolated Task worktree. Observe baseline before source edits, with no mutation of real .pi records/event history.", "Use release builds and deterministic PTY/output measurements, not debug timing. Measure ten-second wchar/syscw and report byte/s + calls/s; preserve terminal dimensions/theme/input conditions.", "No general rendering framework, permanent tests/fixtures, push or release without separate authorization. Keep Rules task53 mockup lane separate."]
  summary: "Moved the synced-path doc comment back onto `is_board_path` and documented `board_fingerprint` accurately. No logic changed. Commit 3164830 (on top of implementation commit 02142c7); checkout clean."
  evidence: ["Correctly document fingerprint and path policy without further implementation changes: `board_fingerprint` now has the content-digest description; the original synced-board-path comment is directly above `is_board_path`. Only comments changed.", "Deliver clean committed Worker checkout: Comment fix committed as 3164830; checkout has no remaining changes."]
  filesChanged: ["tandem/src/project/sync.rs"]
  updatedAt: "2026-10-17T04:34:02Z"
createdAt: "2026-10-17T03:25:22Z"
updatedAt: "2026-10-17T04:34:02Z"
assignee: "worker-task-86-5004cbf6"
archivedAt: "2026-10-17T04:34:02Z"
resolution:
  outcome: "completed"
---

## Description

Algorant/Herdra measured static desktop .pi Board TUI pid3432147 at~1744 writes/s and~2.9MB/s,118GB over10.5h; five otherTUIs70–580KB/s. OriginalPID has now exited. Diagnose actual redraw/timer/sync/spinner/fingerprint cause; don't assume every draw implies terminal writes because Ratatui should diff unchanged frames. Check size dependence including a board with~526logs. Fix to render only when visible state changes/input, with justified slowclock only.

Use safe disposable Git-backed board snapshots or generated data, never mutate realboard/history; read-only existing process IO sampling is allowed. Trace event loop, redraw invalidation, screen/terminal clearing and reload/backgroundsync. First concrete plan must name suspected path and reproducible baseline measurement, then parent approves code. Verify output while idle and actual visible changes, plus nativefmt/tests/strictclippy. No release/push authorized in this request.


## Measurement correction from Herdra

Original rates were /proc/PID/io totals, NOT per-fd or confirmed pty writes. Herdr rchar across allpanes~67KB/s and itsCPU unchanged whenTUIsSIGSTOPed, so no evidence terminal is churning. write_bytes145MB vswchar118GB. LiveTandem hasstdio→pty, eventpoll andconnectedAF_UNIXsocketpair (bothends owned bysameprocess). Herdra suspects internalwake/channel loop; investigate socketpair purpose, but test attribution rather than assume. LinuxprocessIO may include waited-forchildren, which a staticFDtable misses. Normal-priority performancefix, noturgent. RootWorker remains diagnostic/read-only before concrete cause+minimalimplementationplan approval; no guesses/syncdisable/renderframework.

## Remote-backed cause verified; guarded implementation approved

Remote-backed526-log release baseline10.010s: PTY41B, processwchar2,581,131/syscw16,313. Git-wrapper attribution3,045,307stdoutB/10s, chiefly13cat-file+19ls-tree calls. BackgroundSync calls fullPublish every2s; Engine loads base/current/pending trees BEFORE unchanged detection. No redraw/socket bug proved; do not change TUI rendering.

Approve a narrow background-only unchanged-publish guard, both local-only/remote-backed, preserving2s local checks and30s remoteRefresh. Use a CONTENT-SENSITIVE fingerprint of existing authoritative synced-board file selection, not length/mtime-only (same-size edits/atomic replacement with preservedtimestamps must be detected). Prefer existing read_board_dir/is_board_path authority, no new registry/cacheframework/dependency. Do not advance successfulbaseline forpending/error. Tie baseline to observedsync input/output: changes DURING sync cannot be markedalready-published by blindlytakinga postsyncfingerprint; conservatively invalidate when before/after differ. Refresh never suppressed. File errors surface through existing warning/retry, not silently swallowed.

Temporary tests: nochange/same-size+preservedmtime/atomicreplace/add/delete,2s localpublish,30s remotechanges,concurrent localedit duringpublish, held/conflict and safetycopy/recovery. Measure uninstrumented matching10sPTY/proc beforeafter and separateGit-wrapper attribution; startupstatussettled andsamegeometry. Nativechecks, cleanup, no permanenttests/fixtures, no push/release. Parentreview beforeintegration.