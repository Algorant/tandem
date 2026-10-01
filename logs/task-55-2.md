---
id: task-55-2
uid: 9cbc3e63-13ed-445f-9db4-acb5d382c0e7
type: task
title: "Migrate active Board research and papercut tags to kinds"
effort: "small"
parentId: "task-55"
tags: ["protocol", "taxonomy"]
accord:
  status: "ready"
  acceptance: ["`tandem migrate` (or an equivalent explicit migration step) sets `kind` from the `research`/`papercut` tag and removes that tag on active Board Tasks only.", "Records it cannot convert unambiguously (both tags, `kind: epic` plus one of the tags, or a papercut-tagged Subtask) are reported and left unchanged. Nothing is guessed.", "Archived Logs are byte-for-byte unchanged by the migration, as shown by tests."]
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-01T23:32:38Z"
createdAt: "2026-10-01T22:20:49Z"
updatedAt: "2026-10-01T23:32:38Z"
archivedAt: "2026-10-01T23:32:38Z"
resolution:
  outcome: "completed"
---

## Description

Board-only migration. Archived Logs keep their legacy tags and are not rewritten. See task-55.
