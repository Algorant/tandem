---
id: task-233
type: task
title: "Research remaining Logs TUI slowdown and optimization opportunities"
state: todo
priority: "high"
effort: "medium"
references: ["task-226", "task-242"]
relatedFiles: ["tandem/src/tui/logs.rs", "tandem/src/tui/state.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/mod.rs"]
tags: ["tui", "logs", "performance", "research"]
createdAt: "2026-08-21T03:29:46Z"
updatedAt: "2026-08-25T12:57:04Z"
---

## Description

Investigate why the Logs screen still feels substantially slower than expected after task-226 eliminated idle busy rendering and introduced viewport projection. This is a measurement-first research task, not an implementation task.

Reproduce the slowdown with both a realistic Tandem workspace and generated fixtures at multiple log counts. Profile startup/view-switch latency, rapid keyboard and mouse navigation, filtering, detail rendering and scrolling, external-change checks, and steady-state CPU. Compare debug and release builds. Determine where time and allocation are spent across log loading/parsing, event loading, hierarchy construction, filtering, viewport projection, detail-line generation, reload fingerprinting, Ratatui diff/render work, and terminal output.

Review task-226's benchmark and assumptions. Identify gaps between its approximately 55 ms synthetic interaction result and the current poor real-world experience, including whether log body size, event volume, relationship depth, terminal dimensions, themes, debug builds, filesystem scale, or repeated projection work change the result.

Deliver a concise evidence-backed report with reproducible commands, baseline measurements, ranked bottlenecks, low-risk quick wins, larger architectural options only where justified, expected impact, regression risks, and recommended implementation task boundaries. Preserve current behavior and do not optimize speculatively.

## Benchmark harness was removed (task-242)

The harness this task was written against no longer exists. `scripts/benchmark_tui_idle.py` and the `just bench-tui-idle` recipe were deleted in task-242, which removed Python from the repository. The `tandem/README.md` benchmark section now points here.

Recover it from git before starting:

    git show 6247192~1:scripts/benchmark_tui_idle.py

It was 297 lines of dependency-free Python 3. It drove the release binary through a real PTY fixed at 46x150 using `pty`, `fcntl`, `termios`, and `select`, sampled CPU from Linux `/proc`, generated throwaway workspaces at 10/50/100/250 Log counts, and asserted at most 5% idle CPU at the largest count, selection redraw within 250 ms, and external-change render within one second. It also supported `--report-only`, `--binary` for debug-build comparison, `--prepare-workspace <new-path>` for a persistent visual fixture that refused to overwrite an existing path, and `--check-prepare-refusal` to validate that boundary.

Deciding what replaces it is part of this task, and it should be settled before the profiling work rather than reconstructed ad hoc. The harness was run once, at creation in task-226, so its 55 ms interaction figure has no corroborating second measurement. Since this task already questions whether that synthetic result reflects real usage, treat the old design as input, not as the baseline to reproduce.

Whatever replaces it must not be Python. Rust with a PTY dev-dependency is the expected direction. Record the choice as a decision if it adds a dependency.
