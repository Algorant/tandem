---
id: task-86
uid: 4f5af325-754e-4c25-9a3e-7880966c7125
type: task
kind: "papercut"
title: "Diagnose and eliminate unnecessary idle TUI I/O"
state: "in-progress"
priority: "medium"
effort: "medium"
relatedFiles: ["tandem/src/app/background_sync.rs", "tandem/src/project/sync.rs"]
tags: ["tui", "rendering"]
accord:
  status: "rework"
  acceptance: ["Attribute ten-second process-wide wchar/syscw deltas to actual write destinations and waited-for child accounting; identify the concrete unnecessary idle-I/O cause and board/log-size dependence before source edits.", "Remove the proven unnecessary idle work while preserving input, resize, local reload, background sync/status and chooser refresh; no speculative redraw patch when PTY is already quiet.", "Parent verifies comparable ten-second before/after wchar/syscw plus segmented PTY byte counts on safe small and ~526-log boards, native checks pass and temporary harnesses/fixtures are removed."]
  claimedAt: "2026-10-17T03:25:52Z"
  deliveredAt: "2026-10-17T04:10:23Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Algorant requested immediate bug fix via Herdra; no mockup or unrelated layout/feature changes.", "Plan first, explicit Luna/high; isolated Task worktree. Observe baseline before source edits, with no mutation of real .pi records/event history.", "Use release builds and deterministic PTY/output measurements, not debug timing. Measure ten-second wchar/syscw and report byte/s + calls/s; preserve terminal dimensions/theme/input conditions.", "No general rendering framework, permanent tests/fixtures, push or release without separate authorization. Keep Rules task53 mockup lane separate."]
  summary: "Implemented and committed a per-background-worker content fingerprint guard that skips only unchanged Mode::Publish syncs; two-second checks, 30-second refresh, pending/error retry, and existing changed-board sync behavior remain. Root cause was repeated background Git snapshot/tree materialization, not terminal rendering. Commit 02142c7; checkout clean, no push/release."
  evidence: ["Attribute idle process-wide write totals to actual write destinations and waited-for child accounting; identify concrete unnecessary idle-I/O cause and board-size dependence before source edits.: Verified source call path is BackgroundSync 2s publish → Engine::run base load → snapshot temp-index commands → current/pending tree materialization via ls-tree/cat-file before unchanged detection. Remote-backed 526-log wrapped child output was 3,045,307B/10s versus only 41 PTY bytes; large board process counters scaled sharply over small-board baseline.", "Remove proven unnecessary idle work while preserving input, resize, local reload, background sync/status and chooser refresh; no speculative redraw patch when PTY is already quiet.: Added content-sensitive per-instance baseline from existing synced-board file selection; skip only unchanged Mode::Publish. Baseline advances only on successful sync with identical before/after fingerprints; concurrent edits invalidate it. Mode::Refresh remains unconditional at 30s; pending/errors do not advance baseline, fingerprint errors surface and retry. Same-size preserved-mtime edit and edit-during-publish both triggered correct later publish; 30s refresh fetch still ran. No TUI-render code changed.", "Parent verifies comparable ten-second before/after wchar/syscw plus segmented PTY byte counts on safe small and ~526-log boards, native checks pass and temporary harnesses/fixtures are removed.: Remote-backed 526-log before/after figures are in validation: Δwchar 2,581,131→41; Δsyscw 16,313→3; PTY remained 41 bytes/10s. Local-only 526-log after was Δwchar=41, Δsyscw=3, PTY=41 bytes/10s. Git wrapper separated subprocess stdout from PTY and post-change found zero Git calls in the idle 10s window. Native checks passed and temporary tests/wrappers/fixtures were removed."]
  filesChanged: ["tandem/src/app/background_sync.rs", "tandem/src/project/sync.rs"]
  note: "Parent independent verification PASSED: remote small1log baseline10s wchar27,205/syscw175→after0/0; remote526logs3,605,200/16,070→after0/0; PTY0 both before/after after6swarmup. Contentedit same-size+preservedmtime published1.63s; remotepeeredit arrivedonscheduledrefresh; input/resize/quitgood. Native337+76/fmt/strictclippy/diffcheckpass. No logic correction requested."
  updatedAt: "2026-10-17T04:10:24Z"
createdAt: "2026-10-17T03:25:22Z"
updatedAt: "2026-10-17T04:10:24Z"
assignee: "worker-task-86-5004cbf6"
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