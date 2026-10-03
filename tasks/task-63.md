---
id: task-63
uid: 701b0825-d104-4961-9bdb-804d826d3524
type: task
kind: "papercut"
title: "Concurrent add can return its provisional task-new ID after a parallel sync already numbered it"
state: todo
priority: "low"
effort: "small"
relatedFiles: ["tandem/src/cli/commands.rs", "tandem/src/project/sync.rs"]
tags: ["protocol"]
accord:
  status: "ready"
  acceptance: ["Two concurrent `tandem add` calls on a Git-backed board each return their final published `task-N` ID in `data.id` whenever `sync.status` is `synced`."]
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-03T03:11:29Z"
createdAt: "2026-10-03T03:11:29Z"
updatedAt: "2026-10-03T03:11:29Z"
---

## Description


## Description

Reported from the ~/.pi board (task-327). Reproduced on 0.16.2 on 2026-10-03, 3 times out of 3, with two parallel `tandem add task … --json` calls in a Git-backed workspace with a local bare remote. One call returns its final `task-N` with a `renamed` map that holds both mappings. The other returns its provisional `data.id` (for example `task-new-5fe72d46`) with `sync.status: synced` and an empty `renamed` map. The board ends consistent. Agents must cross-map IDs by hand, and the result is misleading because it claims `synced` while reporting a provisional ID.

The CLI maps the outcome ID through its own sync report only (`report.renamed(&outcome.id)` in `tandem/src/cli/commands.rs`). It never resolves the record's current published ID by uid when another process already numbered it.

