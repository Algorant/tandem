---
id: task-49
type: task
title: "Redesign Logs list pane for readable minimal rows"
priority: "low"
createdAt: "2026-06-28T18:30:49Z"
updatedAt: "2026-06-29T20:30:14Z"
accord:
  status: "accepted"
  deliveredAt: "2026-06-29T20:24:35Z"
  deliverables: ["Commit 48a5e4f on branch herd-logs-redesign-49", "Logs list rows simplified to `task-id  title` only", "Removed date/time grouping and optional cue reservation from default rows to maximize title width", "Preserved detail pane ordering with completion/files before process internals", "Updated focused Rust unit tests for id+title-only rows, title truncation, fallback title behavior, and detail ordering"]
  validation:
    commands: ["cd tandem && cargo fmt --check && cargo test (passed, 87 tests)", "git diff --check (passed)"]
  summary: "Reworked the Logs list again per visual feedback: rows now default to task id + title only, with no reserved completion-time/date column and no file/tag cue. The title receives all remaining pane width and truncates at the available row width. The improved detail-pane prioritization from the previous pass is preserved: completion summary and files changed lead, followed by log reference/process metadata/accord/events/body. Local commit: 48a5e4f Simplify logs list to title rows."
  evidence: ["Example full row now renders as `task-36  Implement Tandem docs site foundation`.", "At constrained width, title truncates near the right edge, e.g. `task-36  Implement Tandem docs…`.", "Rows no longer contain completion time/date, raw ISO timestamps, accord status, file paths, file counts, tags, or completion summaries."]
  filesChanged: ["tandem/src/tui.rs", "tandem/src/tui/logs.rs"]
  reviewer: "tui"
  updatedAt: "2026-06-29T20:30:12Z"
review.decidedAt: "2026-06-29T20:30:12Z"
review.note: "this is also interesting, but needs some work as well. I will attach a screenshot to agent so they can see."
review.reviewer: "tui"
review.status: "accepted"
completedAt: "2026-06-29T20:30:14Z"
completion:
  summary: "Applied accepted Validation sign-off for task-49"
  validation: "Accepted by Validation apply-accepted workflow"
  reviewer: "tui"
---

## Description

Problem: the Logs left selection pane is unreadable because each row crams raw timestamps, accord/review state, files, and summary text into a noisy colored blob. Detail belongs in the right pane.

Desired outcome: make the left pane a minimal selectable list while preserving full detail in the right pane.

Acceptance criteria:
- Left list rows prioritize document/task ID, title or summary, completion timestamp, and only tiny optional metadata if space allows.
- Avoid raw ISO timestamps as the leading visual element.
- Avoid dense accord/review/files/long-summary blobs in each row.
- Selected row is clearly highlighted.
- Right detail pane continues to show full metadata, accord detail, files changed, event timeline, and body.
- Coloring is calmer and more consistent.

Example row direction:
task-36  Implement Tandem docs site foundation        completed 17:34

## Feedback

### 2026-06-29 — Rework requested

Human visual review requested another design pass on the Logs list/detail area.

- In the current vertical split, much of the information is still not visible.
- The area still feels like too much metadata/text squeezed into narrow panes rather than an intentional constrained-width design.
- The design needs to respect the vertical split: fewer fields per row, better truncation/wrapping choices, and a clearer hierarchy between the list and detail pane.
- Avoid dumping metadata in the detail pane as a long label/value wall when width is constrained.
- The list should remain scannable, but rows should not rely on invisible/truncated text for meaning.
- Screenshot context showed filtered Logs (`pi`, 15/44) with row titles truncated and the right detail pane showing many metadata lines where half the useful information was clipped.

### 2026-06-29 — Rework requested again

Human visual review says the Logs pane is improved but still not quite there.

- The list is still too noisy.
- Row descriptions/titles are still too long.
- Timestamps are still getting cut off.
- The current visible fields are still too ambitious for the split pane width.
- Rework should start from a more basic/minimal row design:
  - task number/id,
  - completion timestamp,
  - and only one very short secondary cue if useful.
- The secondary cue might be a brief description, tags, status, or something else, but avoid long clipped descriptions as the default.
- Rows should remain readable at narrow width and timestamps should be reliably visible.

### 2026-06-29 — Rework requested again

Human visual review clarified the core Logs design issue.

- The left pane is still being treated like a mini task table, but it should be more like a navigation index.
- The list is visually dense because every row has task id, timestamp, and a truncated title competing equally.
- The wall of repeated ellipses makes scanning worse.
- Rework should prioritize quick selection:
  - task id,
  - completion time/date,
  - optionally one tiny cue.
- Consider grouping by date so dates are not repeated on every row.
- Put rich title/summary in the detail pane, not every list row.
- The right detail pane is better organized but still leads with too much process metadata.
- Detail should lead with human-useful completion summary and files changed, then push accord/event internals lower or make them visually secondary.

- 2026-06-29T20:10:37Z (tui): this is also interesting, but needs some work as well. I will attach a screenshot to agent so they can see.

### 2026-06-29 — Rework requested again

Human review clarified the desired Logs list row direction further.

- Completion time is not especially useful as a primary row field.
- Try an even simpler default row: task id + title only.
- Use the available pane width for the title and truncate near the actual right border of the pane.
- If a right-side/fill field is needed later, use a more useful optional cue than completion time, but start with just task id and title.
- Keep date grouping if useful, but avoid wasting horizontal space on timestamp columns.
