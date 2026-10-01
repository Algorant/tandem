---
id: task-49-8
type: task
title: "Step 8: end-to-end verification, documentation, and release"
state: "in-progress"
priority: "high"
effort: "medium"
parentId: "task-49"
tags: ["protocol", "sync", "docs"]
accord:
  status: "claimed"
  acceptance: ["The native two-clone matrix from the design's verification plan passes, asserting exact record content, references, event integrity, and unchanged source HEAD, index, and working-tree bytes.", "CLI, workspace, and agent docs describe independent sync and state that old code checkouts no longer show historical boards.", "The release is published per the release rules (tag plus GitHub Release) and installable through mise."]
  claimedAt: "2026-10-01T06:30:00Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml", "$ git diff --check"]
  updatedAt: "2026-10-01T06:30:00Z"
createdAt: "2026-09-30T03:54:18Z"
updatedAt: "2026-10-01T06:30:00Z"
assignee: "pi-orchestrator"
---

