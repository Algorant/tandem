---
id: task-5
type: task
title: "Conduct a collaborative full-product review of the Tandem web interface"
state: todo
priority: "high"
effort: "large"
tags: ["ui", "web", "review"]
accord:
  status: ready
  acceptance: ["A prioritized review record with reproducible findings and bounded follow-up Tasks", "Every view, control, state, empty state, detail panel, responsive layout, and primary workflow is audited with realistic project data", "Defects, polish, product decisions, and parity gaps are distinguished and recorded precisely"]
createdAt: "2026-08-31T20:53:48Z"
updatedAt: "2026-08-31T20:53:48Z"
---

## Description

Continued from task-234 in the pre-0.3.0 workspace (archived under .tandem_old); references to the old workspace were removed in the cutover.

After the current web snapshot-loading regression is fixed, run a collaborative human-and-agent pass through the complete Tandem web interface. Inspect every view, menu, control, state, empty state, detail panel, responsive layout, and primary workflow with realistic project data.

Use the live interface as the review surface. The user will identify visual nits, confusing behavior, unwanted choices, and preferences; the agent should capture each point precisely, investigate implementation constraints where useful, and distinguish defects, polish work, product decisions, missing parity, and future features.

Explicitly audit coverage of current Tandem concepts and commands, including Board hierarchy, Logs, Rules, Decisions, Accord state, validation flows, search/filtering, themes, mouse/keyboard behavior where applicable, and Papercuts as tagged tasks. Identify concepts that are absent, stale, incomplete, or inconsistent with the CLI/TUI and protocol.

Deliver a prioritized review record with reproducible findings, screenshots or Sideshow mockups where they clarify direction, proposed product decisions, quick wins, larger feature opportunities, and clearly bounded follow-up Tasks. Do not fold every finding into one implementation change; preserve user preference questions for collaborative judgment.
