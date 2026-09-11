---
id: task-4
type: task
title: "CLI: rules are config-backed with flat ids"
state: "in-progress"
priority: "low"
tags: ["papercut"]
accord:
  status: "claimed"
  acceptance: ["Rules live one per Markdown file in .tandem/rules/ with composite ids like always-12 (D46/D49)", "rules list shows composite category ids"]
  claimedAt: "2026-09-11T14:22:41Z"
  updatedAt: "2026-09-11T14:22:41Z"
createdAt: "2026-08-31T20:53:43Z"
updatedAt: "2026-09-11T14:22:41Z"
assignee: "pi-orchestrator"
---

## Description

Migrating surfaced this: 'tandem rules add' writes into the tandem.md rules block (flat numeric ids, listed as rule-N) instead of per-file .tandem/rules/*.md. The migrated workspace keeps rules config-backed until this lands.
