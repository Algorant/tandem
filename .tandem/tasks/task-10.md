---
id: task-10
type: task
title: "Validation escalation prompt hides the criterion field"
state: todo
priority: "high"
effort: "small"
relatedFiles: ["tandem/src/tui/state.rs", "tandem/src/tui/validation.rs", "tandem/src/tui/chrome.rs"]
tags: ["tui", "validation"]
accord:
  status: ready
  acceptance: ["The escalation prompt renders the criterion field and its prefilled value", "Typed characters go to the field the prompt shows as focused", "A human can complete a validation escalation from the TUI without guessing which buffer has focus"]
createdAt: "2026-09-01T04:50:53Z"
updatedAt: "2026-09-01T04:50:53Z"
---

## Description

## Reproduction

Verified in a rendered Herdr pane against a claimed task with one acceptance criterion.

1. Press `v`, choose Request human validation, press Enter.
2. The popup titled "Request rework" shows only "Feedback to append durably: <type feedback>".
3. Type any text. Nothing appears.
4. Press Enter. Status reads "Validation request requires a note."

The escalation is unfinishable from the TUI.

## Cause

`start_validation_request` (`tui/validation.rs:36`) opens `ValidationPrompt::Rework` with `request: true` and `editing_criterion: true`, and prefills `criterion` from `accord.acceptance`.

`validation_prompt_lines` (`tui/state.rs:1287`) destructures only `feedback` and renders only the feedback field. It ignores `criterion`, `request`, and `editing_criterion` entirely. Keyboard input in criterion mode appends to `criterion` (`tui/validation.rs:247`), which is never drawn, so the visible field stays empty and Enter rejects the empty note.

## Scope note

Found while verifying task-9-1. The prefill itself works and is covered by a unit test. This is the rendering and focus defect, not the data path.
