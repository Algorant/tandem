---
id: task-95
uid: e62d583b-ccae-4978-a5be-6247194e0731
type: task
kind: "papercut"
title: "rules add returns provisional ID without reporting final rename"
state: todo
priority: "low"
tags: ["rules"]
accord:
  status: "ready"
  acceptance: ["A successful synced `tandem rules add --json` returns the final rule ID, or reports the provisional-to-final mapping in `sync.renamed`."]
  updatedAt: "2026-10-29T03:16:00Z"
createdAt: "2026-10-29T03:16:00Z"
updatedAt: "2026-10-29T03:16:00Z"
---

## Description

On 0.17.3, `rules add --json` with category prefer returned `prefer-new-ff7c64e3` and sync data reported `status: synced` with an empty `renamed` map, yet `rules list` immediately showed the rule as `prefer-7`. Callers can't learn the final ID from the mutation response.
