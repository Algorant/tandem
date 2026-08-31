---
id: task-2
type: task
title: "CLI: update rejects decision documents"
state: todo
priority: "low"
tags: ["papercut"]
accord:
  status: ready
  acceptance: ["tandem update decision-N --status accepted works and writes decidedAt", "Decision update validates per type (D34)"]
createdAt: "2026-08-31T20:53:43Z"
updatedAt: "2026-08-31T20:53:43Z"
---

## Description

Migrating to protocol 0.3.0 surfaced this: 'tandem update decision-8 --status accepted' fails with 'only task documents can be updated in v0'. Contract D34 requires type-aware update; decision status/deciders/supersedes must be editable through common update. Migration workaround: frontmatter authoring.
