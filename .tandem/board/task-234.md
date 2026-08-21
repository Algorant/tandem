---
id: task-234
type: task
title: "Conduct collaborative full-product review of Tandem web UI"
state: todo
priority: "high"
effort: "large"
blockers: ["task-231"]
references: ["task-231"]
relatedFiles: ["tandem/src/web.rs", "tandem/src/web/index.html", "tandem/src/web/app.js", "tandem/src/web/ui.js", "tandem/src/web/api.js", "tandem/src/web/app.css", "docs/web/index.md", "plan/web-ui-research.md"]
tags: ["ui", "web", "review", "papercuts"]
createdAt: "2026-08-21T03:32:19Z"
updatedAt: "2026-08-21T03:32:19Z"
---

## Description

After the current web snapshot-loading regression is fixed, run a collaborative human-and-agent pass through the complete Tandem web interface. Inspect every view, menu, control, state, empty state, detail panel, responsive layout, and primary workflow with realistic project data.

Use the live interface as the review surface. The user will identify visual nits, confusing behavior, unwanted choices, and preferences; the agent should capture each point precisely, investigate implementation constraints where useful, and distinguish defects, polish work, product decisions, missing parity, and future features.

Explicitly audit coverage of current Tandem concepts and commands, including Board hierarchy, Logs, Rules, Decisions, review and Accord state, validation flows, search/filtering, themes, mouse/keyboard behavior where applicable, and the newly introduced Papercuts inbox. Identify concepts that are absent, stale, incomplete, or inconsistent with the CLI/TUI and protocol.

Deliver a prioritized review record with reproducible findings, screenshots or Sideshow mockups where they clarify direction, proposed product decisions, quick wins, larger feature opportunities, and clearly bounded follow-up Tasks. Do not fold every finding into one implementation change; preserve user preference questions for collaborative judgment.
