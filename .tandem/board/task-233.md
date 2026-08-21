---
id: task-233
type: task
title: "Research remaining Logs TUI slowdown and optimization opportunities"
state: todo
priority: "high"
effort: "medium"
references: ["task-226"]
relatedFiles: ["scripts/benchmark_tui_idle.py", "tandem/src/tui/logs.rs", "tandem/src/tui/state.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/mod.rs"]
tags: ["tui", "logs", "performance", "research"]
createdAt: "2026-08-21T03:29:46Z"
updatedAt: "2026-08-21T03:29:46Z"
---

## Description

Investigate why the Logs screen still feels substantially slower than expected after task-226 eliminated idle busy rendering and introduced viewport projection. This is a measurement-first research task, not an implementation task.

Reproduce the slowdown with both a realistic Tandem workspace and generated fixtures at multiple log counts. Profile startup/view-switch latency, rapid keyboard and mouse navigation, filtering, detail rendering and scrolling, external-change checks, and steady-state CPU. Compare debug and release builds. Determine where time and allocation are spent across log loading/parsing, event loading, hierarchy construction, filtering, viewport projection, detail-line generation, reload fingerprinting, Ratatui diff/render work, and terminal output.

Review task-226's benchmark and assumptions. Identify gaps between its approximately 55 ms synthetic interaction result and the current poor real-world experience, including whether log body size, event volume, relationship depth, terminal dimensions, themes, debug builds, filesystem scale, or repeated projection work change the result.

Deliver a concise evidence-backed report with reproducible commands, baseline measurements, ranked bottlenecks, low-risk quick wins, larger architectural options only where justified, expected impact, regression risks, and recommended implementation task boundaries. Preserve current behavior and do not optimize speculatively.
