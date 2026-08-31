---
id: task-4
type: task
title: "CLI: rules are config-backed with flat ids"
state: todo
priority: "low"
tags: ["papercut"]
accord:
  status: ready
  acceptance: ["Rules live one per Markdown file in .tandem/rules/ with composite ids like always-12 (D46/D49)", "rules list shows composite category ids"]
createdAt: "2026-08-31T20:53:43Z"
updatedAt: "2026-08-31T20:53:43Z"
---

## Description

Migrating surfaced this: 'tandem rules add' writes into the tandem.md rules block (flat numeric ids, listed as rule-N) instead of per-file .tandem/rules/*.md. The migrated workspace keeps rules config-backed until this lands.
