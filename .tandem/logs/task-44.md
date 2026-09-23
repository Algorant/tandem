---
id: task-44
type: task
title: "Preserve unrelated dirty checkout state during push-boundary consolidation"
effort: "small"
references: ["task-43"]
relatedFiles: ["tandem/src/project/checkpoint.rs", "tandem/tests/checkpoint_behavior.rs", "protocol/README.md", "plan/task-40-pi-handoff.md"]
tags: ["protocol", "checkpoint", "validation"]
accord:
  status: "accepted"
  acceptance: ["Consolidating flush succeeds with unrelated staged, unstaged, and untracked files and preserves their index entries and worktree bytes exactly; owning .tandem is clean after flush", "No-upstream and linked-worktree refusals retain original commits without history rewrite, while tests explicitly allow and prove a preceding forward-only flush when metadata was pending", "Protocol and handoff describe dirty-target tolerance and refusal/flush boundary"]
  claimedAt: "2026-09-23T02:09:58Z"
  deliveredAt: "2026-09-23T02:12:52Z"
  validation: ["$ cd tandem && cargo test", "$ cd tandem && cargo fmt --check"]
  summary: "Replaced whole-checkout cleanliness requirement with owning `.tandem/` cleanliness before and immediately before the ref move. Private-index replay and equal final tree preserve unrelated dirty target state. Updated protocol/host handoff and refusal demonstrations."
  evidence: ["`cd tandem && cargo test --quiet`: 306 unit tests and all integration groups passed, including 13 checkpoint behavior tests; `cargo fmt --check`, `cargo build --quiet`, and `git diff --check` passed.", "Disposable `consolidation_preserves_unrelated_staged_unstaged_and_untracked_state`: staged modification plus unstaged edit on same tracked source path, staged addition, and untracked file all retained identical index entries, staged diff, status, and worktree bytes after consolidation; owning `.tandem` clean, final tree equal to flushed old HEAD, upstream unchanged.", "Disposable linked-worktree and missing-upstream refusal tests each put pending metadata before the command: command exits with checkpoint envelope; new HEAD is an ordinary forward checkpoint whose parent is the original HEAD; a repeat refusal with clean metadata leaves HEAD unchanged; linked Worker HEAD unchanged."]
  filesChanged: ["tandem/src/project/checkpoint.rs", "tandem/tests/checkpoint_behavior.rs", "protocol/README.md", "plan/task-40-pi-handoff.md"]
  updatedAt: "2026-09-23T02:12:56Z"
createdAt: "2026-09-23T02:09:55Z"
updatedAt: "2026-09-23T02:12:56Z"
assignee: "pi"
archivedAt: "2026-09-23T02:12:56Z"
resolution:
  outcome: "completed"
---

## Description

Pre-release correction to completed task-43, requested after Algorant's independent review. The current consolidation preflight rejects any unrelated source/index dirt, even though private-index tree-preserving rewrite should leave that state untouched. Scope: native checkpoint consolidation, disposable-repo evidence, and protocol/host handoff; no adapter or release work.
