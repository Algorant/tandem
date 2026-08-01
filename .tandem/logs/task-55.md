---
id: task-55
type: task
title: "Refine Validation review actions and badge semantics"
priority: "medium"
relatedFiles: ["tandem/src/tui.rs", "tandem/src/main.rs", "tandem/plan/spec.md"]
tags: ["tui", "validation", "review", "ux", "badges"]
createdAt: "2026-06-29T01:31:46Z"
updatedAt: "2026-06-29T20:11:14Z"
accord:
  status: "accepted"
  assignee: "validation-review-55"
  claimedAt: "2026-06-29T03:11:49Z"
  deliveredAt: "2026-06-29T18:09:29Z"
  deliverables: ["Rework prompt no longer treats `n` as cancel while entering feedback; underlying hotkeys remain intercepted while the modal is open.", "Regression test covers typing `n`, `a`, `e`, and `/` into Validation rework feedback without quick-add/editor/search/view side effects."]
  validation:
    commands: ["cd tandem && cargo fmt --check && cargo test"]
  summary: "Implemented Validation modal input isolation rework: feedback dialogs now keep printable hotkey characters as text, Esc is the cancellation path, Enter submits, Ctrl-U/backspace remain scoped to the prompt, and regression coverage verifies n/a/e/slash do not trigger underlying Board actions."
  evidence: ["Commit 66e77b4 on branch herd-validation-review-55", "Validation passed: cd tandem && cargo fmt --check && cargo test (76 tests)"]
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "tui"
  updatedAt: "2026-06-29T20:11:03Z"
review.decidedAt: "2026-06-29T20:11:03Z"
review.reviewer: "tui"
review.status: "accepted"
completedAt: "2026-06-29T20:11:14Z"
completion:
  summary: "Applied accepted Validation sign-off for task-55"
  validation: "Accepted by Validation apply-accepted workflow"
  reviewer: "tui"
---

## Description

Validation should be a human sign-off queue, not a generic delivered-work list. Refine the TUI Validation/Board experience so review actions are explicit and badges are useful but visually calm.

Scope:
- Replace noisy/repetitive delivered badges in Validation with more useful human-review signals such as visual, accepted, or rework where applicable.
- Tone down badge/chip styling for minimalist themes; badges should aid scanning without being jarring or visually dominant.
- Pressing A on a validation item should open an explicit confirmation dialog and then mark the work accepted/sign-off with a calm accepted indication.
- Pressing R should open a Request rework/Feedback dialog with a text input area. On confirm, durably append the feedback to the task body (for v0, a Feedback section in the task markdown is sufficient), update accord/review state to rework, and move the task back to an actionable state.
- Remove or de-emphasize C from Validation as a primary action; completion/logging should generally happen after acceptance or for non-human-verification work outside the visual validation queue.

Design notes:
- Keep workflow state, review metadata, and accord status distinct.
- Do not attempt realtime routing to the original herd/orchestrator in v0; durable task-file feedback is enough.
- Feedback entries should include timestamp/source if practical.
- Dialogs should show the target task id/title and have clear confirm/cancel behavior.

Acceptance:
- Validation rows no longer show a redundant delivered badge for every delivered item.
- Human review needs are visually clear with calmer/minimal badges.
- A confirmation flow prevents accidental acceptance.
- R captures feedback and appends it durably to the task markdown without overwriting existing content.
- Rework action updates lifecycle state consistently and removes the item from the human sign-off queue.
- Tests cover accept confirmation, rework feedback append/cancel behavior, and badge rendering semantics.

## Feedback

### 2026-06-29 — Rework requested

Human review found a modal input isolation bug in the Validation rework feedback dialog.

- When pressing `R` to give feedback, the popup appears to still allow underlying TUI hotkeys to fire while typing.
- For example, typing a normal letter such as `n` in the feedback message can trigger the underlying `n`/new behavior instead of just inserting text.
- When any modal popup/text-entry dialog is open, normal global/view hotkeys must be disabled or intercepted.
- The modal should own input:
  - printable characters edit the text field,
  - Enter submits/advances,
  - Esc cancels,
  - Ctrl-U clears where supported,
  - unrelated hotkeys no-op.
- Add regression coverage for typing hotkey characters like `n`, `a`, `e`, and `/` into rework feedback without triggering underlying pane actions.
