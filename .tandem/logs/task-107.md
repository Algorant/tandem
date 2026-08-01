---
id: task-107
type: task
title: "Fix tandem update/pi-tandem mismatch for unsupported task fields"
priority: "medium"
relatedFiles: ["tandem/src/main.rs", "extensions/pi-tandem/index.ts", "tandem/plan/spec.md"]
tags: ["pi-tandem", "update", "bugfix"]
createdAt: "2026-07-05T16:39:39Z"
updatedAt: "2026-07-05T17:28:36Z"
subtasks:
  - id: task-107-1
    title: "Confirm intended update surface"
    completed: false
  - id: task-107-2
    title: "Add regression coverage"
    completed: false
  - id: task-107-3
    title: "Implement CLI support or wrapper rejection"
    completed: false
  - id: task-107-4
    title: "Update tool guidance/help"
    completed: false
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-05T16:43:24Z"
  deliveredAt: "2026-07-05T17:15:45Z"
  deliverables: ["Verified tandem update supported fields against parser/help/spec: CLI update supports title, kind, priority, assignee, dueDate, tags, blockers, references, and relatedFiles; description/subtask/accord/review remain unsupported by design.", "Added pre-invocation pi-tandem validation for action=update when description, subtasks, accord, or review are supplied, with clear recommended alternatives.", "Updated schema/tool/prompt guidance and tandem plan spec documentation for unsupported update fields."]
  validation:
    commands: ["bun extensions/pi-tandem/tests/smoke.ts: passed", "cargo test --manifest-path tandem/Cargo.toml update: passed (4 passed)", "git diff --check: passed"]
  summary: "Accepted task-107: pi-tandem now explicitly rejects unsupported tandem_task update fields (description, subtasks, accord, review), updates schema/guidance/spec wording, and includes smoke coverage preventing silent no-op behavior. Commit 5935e7a reviewed; working tree clean; validations passed."
  evidence: ["Commit 5935e7a (Fix pi-tandem unsupported update fields)", "git status --short before commit showed only intended files modified"]
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "tandem/plan/spec.md"]
  updatedAt: "2026-07-05T17:28:32Z"
completedAt: "2026-07-05T17:28:36Z"
completion:
  summary: "Fixed pi-tandem unsupported update field handling with explicit rejection, guidance/spec updates, and regression smoke coverage."
  validation: "Reviewed commit 5935e7a. Ran `bun extensions/pi-tandem/tests/smoke.ts`, `cargo test --manifest-path tandem/Cargo.toml update`, and `git diff --check`; all passed."
  reviewer: "pi"
---

## Description

Investigation: the pi-tandem tandem_task schema exposes description, subtasks, accord, and review as generic task params, but buildTaskArgs only maps update to title, priority, assignee, dueDate, tags, blockers, references, and relatedFiles. The tandem CLI update parser and UpdateOptions likewise do not support --description, --subtask, --accord, or --review; direct CLI calls fail with unknown update flag. This creates a mismatch where agents may attempt update-style edits for fields that are only supported on add or via separate commands, and in some wrapper/version combinations this can appear as a misleading no-op/"No changes" result.

Bugfix goal: make update behavior explicit and non-misleading. Decide whether Tandem CLI should support updating description/subtasks/accord/review, or whether pi-tandem should reject/warn before invoking tandem update when update-only unsupported params are supplied. Prefer preserving thin-adapter behavior: if CLI support is not added, pi-tandem schema/tool validation and guidance should make unsupported update fields obvious.

Acceptance criteria:
- Verify current tandem update supported fields against CLI parser/help/spec.
- Add tests covering tandem_task action=update with unsupported fields so the behavior cannot silently no-op.
- Either implement CLI update support for selected fields with validation/events, or make pi-tandem reject/warn clearly for description, subtasks, accord, and review on update.
- Update relevant prompt/schema/help text so agents use tandem add for creation fields, tandem accord for accord lifecycle, and only supported metadata fields for update.
- Document any intentionally unsupported fields and the recommended command path.
