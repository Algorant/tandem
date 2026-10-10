---
id: task-67
uid: e9d8dcb5-57f9-4b01-ac2f-c691e6b3a549
type: task
kind: "papercut"
title: "Residual stale doc claims after task-66: review-status UI text, install tag pins, decision date field"
state: todo
priority: "low"
references: ["task-66", "task-51"]
tags: ["docs"]
accord:
  status: "ready"
  acceptance: ["Each listed claim is either verified against the current TUI/web/CLI code and kept, or corrected; install snippets either track the latest release or avoid pinning a version."]
  updatedAt: "2026-10-10T14:33:37Z"
createdAt: "2026-10-10T14:33:37Z"
updatedAt: "2026-10-10T14:33:37Z"
---

## Description

Flagged as doubtful by the task-66 Worker and left unchanged because they need checking against TUI/web code, not the CLI:

- docs/web/index.md:38-39 ("pending review, and requested changes"), plus docs/tui/index.md and tandem/README.md, which describe review.status badges and `[badges.review]` keys. Review status is not stored under protocol 0.5.0.
- tandem/README.md:5 describes the Validation queue as "delivered work awaiting accept/rework/complete".
- Install snippets pin stale tags: docs/cli/index.md `tandem-v0.4.2`, docs/quick-start `tandem-v0.4.0`, README.md `tandem-v0.10.1`. README.md:151 says "reports the 0.12.x release".
- docs/guides/decisions.md YAML example shows a manual `date`, which protocol/README.md says does not exist.
