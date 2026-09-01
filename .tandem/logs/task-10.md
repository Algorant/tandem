---
id: task-10
type: task
title: "Remove TUI validation escalation and unify accord field shapes"
priority: "high"
effort: "small"
relatedFiles: ["tandem/src/tui/state.rs", "tandem/src/tui/validation.rs", "tandem/src/tui/chrome.rs"]
tags: ["tui", "validation"]
accord:
  status: "accepted"
  acceptance: ["The TUI validation picker no longer offers Request human validation, and its criterion/request plumbing is deleted rather than disabled", "v on an active task reports that validation actions apply to delivered work", "accord claim writes top-level assignee and release clears it; accord.assignee is removed from the record, renderer, and all readers", "tandem list --assignee returns claimed tasks", "accord.validation is one flat list matching acceptance and constraints; the nested commands form and the accord.validations fallback are removed", "add and every accord transition write the same accord block shape"]
  claimedAt: "2026-09-01T17:52:19Z"
  deliveredAt: "2026-09-01T17:57:34Z"
  summary: "Escalation removed; assignee and validation shapes unified"
  evidence: ["244 tests, clippy and fmt clean", "rendered pane: active task offers no action, validation accept/request-changes work end to end"]
  updatedAt: "2026-09-01T17:57:34Z"
createdAt: "2026-09-01T04:50:53Z"
updatedAt: "2026-09-01T17:57:34Z"
archivedAt: "2026-09-01T17:57:34Z"
resolution:
  outcome: "completed"
---
## Why

Three separate divergences, all cleanup rather than new behavior.

### 1. TUI validation escalation is the wrong feature for the surface

`v` on an active task offers only "Request human validation", which has never worked: `validation_prompt_lines` (`tui/state.rs:1287`) renders only the feedback field while input in escalation mode appends to an unrendered `criterion` buffer.

It is not worth fixing. Escalation is an agent asking a human for judgment (D15, rule always-12), and `tandem review <id> --criterion --note` is that path. A human in the TUI escalating to themselves has no meaning. Delete the entry and its plumbing; keep Accept and archive and Request changes, which act on work already waiting for a human.

### 2. assignee contradicts D23 and breaks a filter

D23: "Keep only top-level `assignee`. Claim sets it; release clears it. Remove `accord.assignee`." The implementation does the opposite. `list --assignee` filters top-level `assignee` (`app/queries.rs:361`), which claim never writes, so filtering by assignee silently returns nothing for claimed work.

### 3. Accord block shape flips on first transition

`add` writes `validation: [...]` flat (`app/tasks.rs:258`) while `render_accord_block` writes `validation:\n  commands: [...]`. `AccordRecord::from_document` tolerates both plus a third `accord.validations` spelling. Now that `show --json` exposes the accord to agents, one shape must win.

Canonical is the flat list, matching `acceptance` and `constraints` and the repeated `--validation` flags. Remove the nested form and both fallbacks. No live workspace data uses the nested shape.

## Direction

Route `add` through the same accord rendering path as transitions so the shape cannot drift again.