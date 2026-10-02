---
id: task-59
uid: b5a832b0-4012-47c4-a9c5-50a2e02fcb19
type: task
kind: papercut
title: "just dev-check skips clippy and the docs audit, so the release gate catches them late"
state: todo
priority: "low"
tags: ["config", "validation"]
accord:
  status: "ready"
  acceptance: ["`just dev-check` fails on the same clippy lints that the `just release` gate rejects."]
  updatedAt: "2026-10-01T23:46:59Z"
createdAt: "2026-10-01T23:46:59Z"
updatedAt: "2026-10-01T23:46:59Z"
---

## Description

During the 0.16.0 release, `just release` failed twice after Workers had delivered green `just dev-check` runs. First, two `cargo clippy -D warnings` lints from task-55 and task-56. Second, a `bun audit --audit-level=high` advisory for devalue in site/. Both were fixed before tagging (8b4ab9a, 3144198). Running the strict clippy gate (and possibly the docs audit) in dev-check would catch these at Worker delivery time.
