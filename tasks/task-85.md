---
id: task-85
uid: 7595ee64-8a20-4cc1-8c33-e9a8827f0160
type: task
kind: "papercut"
title: "Document the coordinated 0.17.1 shared-board upgrade"
state: "in-progress"
priority: "low"
effort: "small"
references: ["task-50", "task-83"]
relatedFiles: ["docs/cli/index.md", "RELEASES.md"]
accord:
  status: "claimed"
  acceptance: ["CLI docs and the published 0.17.1 release explicitly tell every machine and agent sharing a board to upgrade to 0.17.1 together because 0.17.0 readers skip new data-only audit events.", "Guidance tells operators to verify the actual agent binary and restart old clients; no migration, event-file edits, version bump, or 0.17.2 release is introduced.", "Site build/link checks pass and the guidance is verified in the live docs and GitHub Release body."]
  claimedAt: "2026-10-16T16:45:50Z"
  validation: ["$ just site-build", "$ cd site && bun run check:links"]
  updatedAt: "2026-10-16T16:45:50Z"
createdAt: "2026-10-16T16:45:40Z"
updatedAt: "2026-10-16T16:45:50Z"
assignee: "tandem"
---

## Description

Herdra approved a documentation-only fix now after read-only confirmation: on a disposable copy of the six data-only event lines, 0.17.0 emits six missing-summary warnings and 0.17.1 emits none. Add short coordinated-upgrade guidance to the CLI page, canonical RELEASES.md 0.17.1 section and published GitHub Release body, preserving generated install/download sections. Every user is Algorant's own machines; algomarchy/cart-lab own their mise upgrades. Do not run machine upgrades here or modify event history; no 0.17.2, tag rewrite, or binary/assets change. Push/deploy this authorized small docs correction and verify live. Direct parent implementation is bounded and tightly coupled.
