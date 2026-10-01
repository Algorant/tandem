---
id: task-56
uid: 1bd31435-ea2b-4099-ba34-a558f0e8d3f3
type: task
title: "Add typed links between records: relates-to, duplicates, fixed-by, supersedes"
state: todo
priority: "low"
effort: "medium"
references: ["task-22", "task-55"]
tags: ["protocol", "relationships"]
accord:
  status: "ready"
  acceptance: ["The protocol defines typed links between records (at least `relates-to`, `duplicates`, `fixed-by`/`fixes`, `supersedes`) and documents their storage shape and how they relate to the existing `references`.", "The CLI can add, remove, show, and filter typed links, and `complete` can record that a Task was resolved as fixed by another record.", "Tests cover link validation (unknown targets, self-links, archived targets) and the fixed-by completion path."]
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-01T22:21:03Z"
createdAt: "2026-10-01T22:21:03Z"
updatedAt: "2026-10-01T22:21:03Z"
---

## Description

## Why
`references` is untyped today. Pi (~/.pi) wants papercuts to stay standalone while connecting to the work they relate to. The main case: a papercut that another Task's work already fixed can be closed as `fixed-by` that Task, citing it, instead of with a manual note.

Independent of task-55. Discussion and mockups: http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8
