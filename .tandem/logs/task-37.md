---
id: task-37
type: task
title: "Fold Tandem metadata into the last unpushed commit"
priority: "high"
effort: "medium"
relatedFiles: ["tandem/src/project/checkpoint.rs", "tandem/tests/checkpoint_behavior.rs", "protocol/README.md"]
tags: ["protocol", "git"]
accord:
  status: "accepted"
  acceptance: ["An assignment boundary amends any unpushed HEAD with the .tandem pathspec and keeps a non-chore commit message; it never rewrites a pushed commit or a merge.", "Unpushed chore(tandem) commits sitting next to an unpushed real commit are folded into that real commit so git log does not show those chores.", "Unrelated staged, unstaged, and untracked files stay untouched. Real-Git tests prove the new history shape.", "Pi/adapters are not modified."]
  claimedAt: "2026-09-17T13:49:00Z"
  deliveredAt: "2026-09-17T13:52:11Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml --test checkpoint_behavior", "$ cargo test --manifest-path tandem/Cargo.toml", "$ git diff --check"]
  constraints: ["Do not rewrite origin or other remote-tracking commits.", "Do not fold into merge commits.", "No Pi adapter or git hook the user has to run.", "Tests may mutate disposable repositories only."]
  summary: "Assignment boundaries now amend any unpushed non-merge HEAD with .tandem/ and keep a real commit message. Leftover unpushed chore commits next to a real commit fold into it. Pushed commits are not rewritten. Pi was not changed."
  evidence: ["cargo test --manifest-path tandem/Cargo.toml --test checkpoint_behavior: 12 passed", "cargo test --manifest-path tandem/Cargo.toml: full suite passed", "git diff --check: clean", "Unpushed ordinary commits keep their subject and absorb .tandem/; leftover sandwich folds to the source commit; dirty unrelated files stay untouched; pushed HEAD still gets a new checkpoint."]
  filesChanged: ["tandem/src/project/checkpoint.rs", "tandem/tests/checkpoint_behavior.rs", "protocol/README.md"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-09-17T13:52:14Z"
createdAt: "2026-09-17T13:48:56Z"
updatedAt: "2026-09-17T13:52:14Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-17T13:52:14Z"
resolution:
  outcome: "completed"
  reviewer: "pi-orchestrator"
---

## Description

Going forward, Tandem must not leave chore(tandem): checkpoint metadata commits next to real work. If HEAD is unpushed, put .tandem/ into that commit and keep its message. Then fold leftover unpushed tandem-only commits into the neighboring real commit. Pushed history is not rewritten. Pi stays a client.
