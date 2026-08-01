---
id: task-62
type: task
title: "Design docs information architecture and launch content map"
priority: "low"
parentId: "task-59"
references: ["decision-1"]
relatedFiles: ["README.md", "docs/index.md", "docs/concepts/index.md", "docs/guides/index.md", "plan/spec.md", "protocol/README.md", "tandem/README.md"]
tags: ["docs", "ia"]
createdAt: "2026-06-29T20:49:28Z"
updatedAt: "2026-07-04T20:53:58Z"
accord:
  status: "accepted"
  assignee: "pi-docs-ia"
  claimedAt: "2026-07-04T19:47:53Z"
  deliveredAt: "2026-07-04T20:53:41Z"
  deliverables: ["Sideshow v4 review artifact: http://localhost:8228/session/KZcJaf-4LBI/s/gaEKmbO9S_Q", "Approved first-pass IA: Home + real top-level /quick-start/ + TUI-framed Concepts + practical CLI/TUI/Protocol pages + optional Extensions link-out.", "Quickstart scope: CLI/TUI only, brief install lanes for curl/install.sh, Cargo, and AUR binary.", "Concepts scope: TUI/user-workflow POV covering Board states todo/in-progress/validation, accords, epics, rules, decisions, and logs.", "Not relevant for first pass: Why Tandem, daily workflow guide, separate completion/logs guide, separate epics guide, Pi adapter guide, split reference tree, MCP/API/library/schema/template/migration docs."]
  validation:
    commands: ["Worker reported `git status --short --branch` clean after Sideshow v4 update: `## main...origin/main`.", "Parent checked `git status --short --branch`: clean before accepting.", "No repository files changed by the reworked planning deliverable; Sideshow is the review artifact."]
  summary: "Accepted task-62 based on user approval of the revised Sideshow-only docs IA. Approved first-pass direction: Home + real top-level /quick-start/ + TUI-framed Concepts + practical CLI/TUI/Protocol pages + optional Extensions link-out; prior maybe-later/drop items are not relevant for now."
  evidence: ["User approved task-62 after reviewing the Sideshow-only update: \"ok 62 approved\".", "Worker output: Sideshow title/version `Task-62 docs IA first-pass recommendation (revised)`, version 4."]
  reviewer: "Algorant"
  updatedAt: "2026-07-04T20:53:50Z"
completedAt: "2026-07-04T20:53:58Z"
completion:
  summary: "Approved and completed docs IA planning. First-pass docs/site shape is Home + real top-level /quick-start/ with brief curl/install.sh, Cargo, and AUR binary install lanes; CLI/TUI-only quickstart; TUI-framed Concepts covering Board states todo/in-progress/validation, accords, epics, rules, decisions, and logs; practical CLI/TUI/Protocol pages; and optional Extensions link-out only. Prior maybe-later/drop items are not relevant for now: Why Tandem, daily workflow guide, separate completion/logs guide, separate epics guide, Pi adapter guide, split reference tree, MCP/API/library/schema/template/migration docs."
  validation: "Human/product approval from Algorant after reviewing revised Sideshow v4 planning artifact. Worker and parent verified no repository file edits remain; planning artifact URL: http://localhost:8228/session/KZcJaf-4LBI/s/gaEKmbO9S_Q."
  reviewer: "Algorant"
---

## Description

Audit the current docs tree, README, protocol specs, CLI/TUI behavior, and competitor/inspiration sites such as brainfile.md. Produce a concrete docs outline with page purposes, required examples, missing screenshots, and prioritized launch gaps.
