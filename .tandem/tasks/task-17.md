---
id: task-17
type: task
title: "CLI: review accepts a criterion that is not in the accord"
state: todo
priority: "low"
tags: ["protocol", "papercut"]
accord:
  status: "ready"
  acceptance: ["review rejects a criterion string that does not match an existing accord acceptance entry, or the exact-criterion requirement is dropped from protocol/README.md."]
  updatedAt: "2026-09-05T22:07:42Z"
createdAt: "2026-09-05T22:07:42Z"
updatedAt: "2026-09-05T22:07:42Z"
---

## Description

## Description

Preserved from published tandem-v0.12.3 task-14 during reconciliation with local delegation proposals task-13–15. Original creation: 2026-09-04T01:36:35Z. Source commit: 735c4cf738a5f4b78c2c28f043ae64cd5461f90a.

Observed while checking whether Pi could rely on the CLI to catch a mistyped criterion. protocol/README.md says review requires the exact unresolved criterion, but this is not enforced.

Reproduction on 0.12.2: add task alpha with acceptance 'criterion one'; claim task-1 as me; run `tandem review task-1 --criterion 'totally made up' --note n --json`. The command succeeds, enters validation, and stores a criterion absent from accord.acceptance.

Not a Pi dependency. Either enforce the match or drop the protocol claim so specification and behavior agree.
