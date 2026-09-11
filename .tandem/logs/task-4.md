---
id: task-4
type: task
title: "CLI: rules are config-backed with flat ids"
priority: "low"
tags: ["papercut"]
accord:
  status: "accepted"
  acceptance: ["Rules live one per Markdown file in .tandem/rules/ with composite ids like always-12 (D46/D49)", "rules list shows composite category ids"]
  claimedAt: "2026-09-11T14:22:41Z"
  deliveredAt: "2026-09-11T14:23:03Z"
  summary: "Reconciled per-file rule storage and composite-ID support already shipped in 0.12.1. Algorant approved closeout; toggles and legacy diagnostic presentation remain separate tasks."
  evidence: ["Current 0.13.1 native CLI scratch probe ran rules add always, asserted .tandem/rules/always-1.md exists, confirmed rules list --json returns id always-1, and confirmed rule text is absent from tandem.md.", "Commits79c375c and5860955 are ancestors of tandem-v0.12.1; RELEASES.md documents per-rule files and composite rule identifiers.", "Rules toggle semantics remain future task-20; legacy embedded-rule detection is completed task-12 and remaining rendered warning visibility belongs to task-32."]
  filesChanged: ["tandem/src/project/rules.rs", "tandem/src/app/rules.rs", "tandem/src/cli/commands.rs"]
  updatedAt: "2026-09-11T14:23:14Z"
createdAt: "2026-08-31T20:53:43Z"
updatedAt: "2026-09-11T14:23:14Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-11T14:23:14Z"
resolution:
  outcome: "completed"
---

## Description

Migrating surfaced this: 'tandem rules add' writes into the tandem.md rules block (flat numeric ids, listed as rule-N) instead of per-file .tandem/rules/*.md. The migrated workspace keeps rules config-backed until this lands.
