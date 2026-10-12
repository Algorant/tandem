---
id: task-69
uid: 10aea0ea-acfb-4e18-b718-0cc7bbdf0231
type: task
kind: "papercut"
title: "blockers accept only board IDs, so dependencies outside the board can only go in prose"
state: todo
priority: "low"
references: ["task-42"]
tags: ["protocol"]
accord:
  status: "ready"
  acceptance: ["A decision is recorded on whether `blockers` accept an external reference (another workspace's Task or free text) or external dependencies stay in prose plus `accord block`.", "The chosen answer is documented in protocol/README.md and the user docs."]
  updatedAt: "2026-10-12T12:35:08Z"
createdAt: "2026-10-12T12:35:08Z"
updatedAt: "2026-10-12T12:35:08Z"
---

## Description

Reported by the p40 Ceph via Herdra. p40 task-6 was blocked on work outside its own board (the ~/.pi `dbs` tooling, the codeparse pilot). `blockers` must resolve to a document on the same board (an unresolved blocker is a validation error), so the Task could only record those dependencies in its body. Nothing shows the Task as blocked, and nothing clears when the external work lands.

Options to decide between:
1. Leave it as prose. External blockers stay in the body, and `accord block --note` marks the Task blocked.
2. Accept an external reference in `blockers`: another workspace's Task (for example `<workspace>:task-N`) or a free-text blocker, recorded but not resolved or validated.
