---
id: task-226
type: task
title: "Eliminate Logs TUI idle CPU and interaction lag at project scale"
state: "validation"
priority: "high"
references: ["task-225"]
relatedFiles: ["tandem/src/tui/mod.rs", "tandem/src/tui/state.rs", "tandem/src/tui/logs.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/chrome.rs", "tandem/Justfile", "Justfile", "docs/tui/index.md", "tandem/plan/spec.md", "tandem/plan/todo.md"]
tags: ["tui", "logs", "performance", "validation"]
createdAt: "2026-08-11T02:53:24Z"
updatedAt: "2026-08-11T03:18:28Z"
accord:
  status: "delivered"
  assignee: "worker-task-226-4c2f3a06"
  claimedAt: "2026-08-11T02:55:58Z"
  deliveredAt: "2026-08-11T03:18:28Z"
  deliverables: ["Integrated squash commit 8bd0b7e from Worker commits b28c539 and b28bb89.", "Event/deadline-driven redraw scheduling for input, resize, reload, and transient-status expiry.", "Visible-viewport Logs row projection with absolute selection and mouse indices.", "`just bench-tui-idle` release benchmark using generated temporary workspaces and a drained 150x46 PTY.", "Safe `--prepare-workspace` behavior that refuses existing paths and a built-in refusal check.", "Updated Tandem README, TUI specification, and implementation todo.", "Integrated-main `just dev` route targeting the retained 250-log `/tmp/tandem-task226-preview` fixture."]
  validation:
    commands: ["Orchestrator rerun: cargo fmt --manifest-path tandem/Cargo.toml --check passed.", "Orchestrator rerun: cargo test --manifest-path tandem/Cargo.toml passed with 268 unit tests and 11 CLI integration tests.", "Orchestrator rerun: cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings passed.", "Orchestrator integrated-main benchmark at 250 logs passed: Board 0.50% CPU, Logs 1.00% CPU, input frame 54.9 ms, external reload 299.9 ms.", "Prepare-workspace refusal check passed; an existing-directory probe returned exit 2 and preserved marker content.", "git diff --check passed.", "Worker real-TUI review covered wide, narrow, and short layouts plus rapid navigation, paging, focus, detail scrolling, filtering, mouse selection/wheel, external reload, and four-second status expiry."]
  constraints: ["Keep task-226 in Validation until final human visual and interaction approval.", "Linux `/proc` supplies benchmark CPU assertions; report-only mode documents unsupported/noisy environments.", "Retain the `/tmp/tandem-task226-preview` fixture and Git-local preview route until human validation, then remove both during final cleanup."]
  summary: "Integrated task-226 as squash commit 8bd0b7e. Replaced continuous TUI redraws with event/deadline-driven rendering, bounded Logs row projection and mouse hits to the visible viewport, and added a dependency-free fixed-PTY idle benchmark. The benchmark helper now refuses every existing caller-supplied preview path instead of deleting it. Release idle Logs CPU at 250 generated logs improved from the Worker baseline of 33.9% to about 1.0% and passed the task threshold. Automated validation and Worker real-TUI review passed. Final human visual and interaction acceptance remains required."
  evidence: ["Worker handoff handoff-031e2309-2809-41b4-800f-eb90a7ea4a6a.", "Worker source commits b28c539 and b28bb89 integrated as 8bd0b7e.", "Run `just dev` from /home/ivan/Projects/tandem for visual validation.", "Run `just bench-tui-idle` for the full release benchmark."]
  filesChanged: ["justfile", "scripts/benchmark_tui_idle.py", "tandem/README.md", "tandem/plan/spec.md", "tandem/plan/todo.md", "tandem/src/tui/chrome.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/state.rs"]
  updatedAt: "2026-08-11T03:18:28Z"
assignee: "worker-task-226-4c2f3a06"
---

## Description

## Outcome

Make the Logs TUI responsive and inexpensive while idle in workspaces with hundreds of completed logs. Add a repeatable benchmark that demonstrates the regression before the fix and the improvement after it.

This is a focused performance task. Preserve current Logs behavior and visual design unless a small rendering change is necessary for virtualization or caching.

## Observed regression and baseline

During task-225 visual validation, the `task-225` fixture contained 230 Logs. The Logs page felt slow and the live TUI consumed about 70% of one CPU core while idle.

A separate fixed-size PTY benchmark against the same fixture measured the installed release build as follows:

| Logs | Idle CPU |
| ---: | ---: |
| 10 | 1.3% |
| 50 | 7.3% |
| 100 | 22.7% |
| 230 | 59.3% |

The integrated debug build measured about 69% at 230 Logs. Emptying log bodies and changing records away from task hierarchy did not materially change the result. This indicates repeated list projection/render work and file polling are stronger signals than Markdown body size or hierarchy classification.

## Current suspected hot paths

Investigate and measure rather than assuming every item below must be rewritten:

- `TuiApp::run` redraws every loop before the 200 ms event poll, even when no visible state changed.
- `draw_log_list` rebuilds the filtered collection and display items on every frame.
- `draw_log_detail` reconstructs selected-detail lines on every frame.
- `filtered_logs` repeatedly allocates a `Vec<&Document>` across selection, status, and rendering paths.
- reload detection fingerprints every Board, Log, Papercut, and event file every 250 ms.
- row hit geometry and selection currently depend on the complete filtered count and must remain correct if rendering becomes viewport-based.

## Required investigation

1. Establish a reproducible baseline against the current code before optimizing it.
2. Separate idle redraw cost, Logs row/detail projection cost, and reload-fingerprint polling cost sufficiently to justify the selected fix.
3. Confirm behavior with approximately 10, 50, 100, and 250 logs.
4. Check both release and debug builds where practical. Release behavior is the primary product metric.
5. Record machine/environment information needed to interpret CPU percentages.

## Benchmark requirement

Add one documented, repeatable local benchmark command, preferably a `just` recipe backed by a dependency-free script or existing Rust test support. Do not add Criterion or a new crate/package solely for this benchmark.

The benchmark must:

- create or use a temporary generated Tandem workspace; do not commit a large fixture;
- exercise a fixed terminal size and the real `tandem tui` Logs page through a PTY or an equivalently representative harness;
- drain terminal output so the child cannot block on a full PTY buffer;
- report results for multiple log counts, including approximately 250;
- measure a stable idle interval after startup and view switching;
- terminate and clean temporary processes/files reliably on success or failure;
- print enough information to compare before and after results;
- document platform limits if process CPU sampling depends on Linux `/proc`;
- return nonzero when a supported benchmark assertion is materially exceeded, while allowing an explicit report-only mode for noisy environments.

Also add deterministic tests around the optimization's correctness and invalidation boundaries. Timing-only unit tests must not be the sole regression protection.

## Implementation direction

Choose the smallest coherent design supported by measurements. Likely options include:

- redraw only after input, resize, data reload, or an explicit timer deadline such as transient-status expiry;
- avoid rebuilding unchanged Logs row projections and detail lines on each frame;
- render or project only the visible Logs viewport while preserving correct selection and mouse hit targets;
- avoid repeated filtered-vector construction for the same query and snapshot;
- reduce or restructure fingerprint polling while retaining reliable external-change detection.

Do not introduce background threads, async runtime dependencies, a new persistence cache, or a generalized UI framework unless measurements prove they are necessary.

## Behavioral requirements

- Keyboard and mouse navigation must remain correct across the full filtered result set.
- Selection, scroll position, list/detail focus, search, and detail scrolling must remain stable.
- `/` filtering must update immediately and invalidate stale projections.
- Theme, terminal resize, reload, external file changes, and selected-log changes must invalidate the correct cached/rendered data.
- Hot reload must remain responsive. Target detection within one second under ordinary local filesystem conditions.
- Four-second transient footer expiry from task-225 must still occur without continuous redraw.
- Papercuts, Board, Rules, Decisions, help, pickers, prompts, and confirmation input must not regress.
- The TUI must not busy-loop while idle.

## Performance acceptance criteria

Use the new benchmark and record before/after tables in the delivery evidence.

On the development machine used for the baseline, with a release build, fixed 150×46 terminal, and approximately 250 generated logs:

1. Idle Logs CPU is at most 5% of one core over the documented stable sample interval, or improves by at least 85% from the same-run baseline if host noise prevents the absolute threshold.
2. CPU growth from 50 to 250 idle Logs no longer resembles the current near-linear busy redraw curve.
3. A normal Logs selection or focus input produces the updated frame without a noticeable multi-frame stall; include a repeatable interaction measurement or bounded work-count assertion where practical.
4. Board idle CPU does not regress materially relative to the same benchmark environment.
5. External file changes remain visible within one second.
6. Transient status clears at approximately four seconds without continuous rendering.

If a threshold must change after better instrumentation, document the measured reason and obtain orchestrator approval before delivery.

## Automated and manual validation

- Add focused tests for redraw scheduling, timer deadlines, cache/projection invalidation, selection, filters, resize, theme/reload changes, and mouse hit mapping as applicable.
- Run `cargo fmt --check`.
- Run the full Rust test suite.
- Run strict Clippy for all targets and features.
- Run the benchmark in release mode and preserve its output in the handoff.
- Run `git diff --check`.
- Perform a real-TUI review with the 250-log generated workspace in wide, narrow, and short layouts. Exercise rapid `j/k`, page movement, list/detail focus, detail scrolling, search entry/clear, mouse selection, wheel scrolling, reload, and idle status expiry.

## Documentation

- Document the benchmark command and interpretation near developer validation guidance.
- Update `docs/tui/index.md` only if user-visible behavior changes.
- Update `tandem/plan/spec.md` and `tandem/plan/todo.md` with the implemented scheduling/caching behavior and performance validation.
- Do not claim a portable CPU threshold on unsupported platforms.

## Excluded

- Redesigning Logs information architecture or visual styling.
- Changing Log protocol/storage format.
- Deleting or compacting historical logs.
- General application profiling infrastructure.
- Web interface performance.
- pi-tandem or other adapter changes.
- A general async/background-task rewrite.

