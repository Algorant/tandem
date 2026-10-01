---
id: task-55
uid: 73825776-f51e-40df-a7b1-9ee17ae8bc2e
type: task
title: "Add research and papercut as first-class Task kinds"
priority: "medium"
effort: "large"
references: ["task-22", "decision-5"]
relatedFiles: ["protocol/README.md", "tandem/src/protocol/document.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/papercuts.rs"]
tags: ["protocol", "taxonomy"]
accord:
  status: "accepted"
  acceptance: ["`research` and `papercut` are valid Task kinds across protocol, CLI, TUI, and docs, with the agreed required fields, default priority, and placement rule enforced by Tandem.", "Active Board records tagged `research` or `papercut` migrate to the matching kind. Archived Logs are untouched.", "No Tandem code path classifies research or papercut work by tag after this change."]
  claimedAt: "2026-10-01T23:02:50Z"
  deliveredAt: "2026-10-01T23:32:56Z"
  validation: ["$ just dev-check"]
  summary: "task-55-2 and task-55-3 are done and committed as e785283 on top of main, which already includes 24e4184 (task-56). Board tabs are now ALL (first, default), each workflow state, RESEARCH, PAPERCUTS. One `BoardView {All, State(String), Kind(&'static str)}` replaces the `__papercuts` synthetic state, `PAPERCUTS_SUBVIEW`, `is_papercut_doc`, `papercut_lens` and `PapercutsState`. ALL and state tabs keep the tree. Kind tabs are flat, cross-state lists read from `kind` only, each row showing a state chip, a `↳ <parent role>: <title>` context chip and a kind badge. Reload restores view and record by id in every tab. Badges (`RESEARCH`, `PAPERCUT`) and the `f` filter (\"Kind …\", \"Clear kind\") come from `kind`. The `research` tag chip is gone. `tui/papercuts.rs` and the `i` inbox are deleted, including its HitActions, header indicator, key, mouse, footer, help section and reload count. Landing text now says 0.5.0. task-55-2 needed no new code (see evidence). Remaining gap: the 'Selected …' header text after a tab change was not separately captured; selection was verified through row contents and tests."
  evidence: ["task-55-2: tag→kind migration covers acceptance: Reviewed rather than added. The migrate.rs unit tests cover in-place conversion and removal of the tag. They also cover ambiguous records, which are reported and left byte-identical: both tags, an Epic plus a tag, and a papercut Subtask. Logs, decisions and rules stay byte-identical. The sync_behavior migrate test asserts the skipped reasons, an unchanged archived log file, and an identical converted file on a second machine. Migration was not run on the live board.", "task-55-3: tab layout ALL · states · RESEARCH · PAPERCUTS: Rendered tab bar from the pane: `ALL 11 │ TODO 9 │ IN PROGRESS 2 │ VALIDATION 0 │ RESEARCH 2 │ PAPERCUTS 3`. ALL is highlighted on launch. Unit tests `board_view_tabs_are_all_first_then_states_then_kinds` and `all_view_is_the_whole_tree_across_states` cover this.", "Kind tabs are flat cross-state lists with parent context: Pane rows show the state chip and `↳ Task of Epic: Explore assignm…` for children of the epic. Root rows have no context. Unit tests cover cross-state children, Task/Subtask roles, and that `references` never appear as parent context.", "One BoardView replaces __papercuts; selection survives reload in every tab: `BoardView` is the only view type. Tests `selection_survives_reload_by_id_in_every_board_view` (ALL, a state, RESEARCH, PAPERCUTS) and `kind_tab_survives_manual_and_auto_reload_without_jumping_tabs` pass. A live external add in the pane kept RESEARCH and #9 selected.", "Badges and f filter from kind; research tag chip removed: Pane shows PAPERCUT and RESEARCH badges. The `f` picker lists `Kind  epic`. Tests `kind_badges_come_from_kind_and_honor_the_opt_out`, `kind_filter_matches_the_kind_field_and_is_offered_by_the_picker` and `kinds_come_from_the_kind_field_never_from_tags` pass. `is_builtin_board_tag(\"research\")` is false.", "Inbox removed: `git rm tandem/src/tui/papercuts.rs`. Test `the_papercut_inbox_and_its_header_indicator_are_gone` asserts no `Papercuts ` text in any view and that `i` is unbound. In the pane, `i` left the status unchanged.", "No tag fallback in code: Grep of non-test, non-migrate code for papercut/research tags found only `kind`-keyed code. The only reader of the tags is the migrate converter."]
  filesChanged: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/board/render.rs", "tandem/src/tui/state.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/input.rs", "tandem/src/tui/pickers.rs", "tandem/src/tui/bindings.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/papercuts.rs (deleted)", "tandem/src/cli/landing.rs", "docs/tui/index.md", "tandem/README.md", "tandem/plan/spec.md", "tandem/plan/todo.md"]
  updatedAt: "2026-10-01T23:32:58Z"
createdAt: "2026-10-01T22:20:35Z"
updatedAt: "2026-10-01T23:32:58Z"
assignee: "worker-task-55-379489ec"
archivedAt: "2026-10-01T23:32:58Z"
resolution:
  outcome: "completed"
---

## Description

## Context
Direction agreed with Algorant on 2026-10-01 in a ~/.pi Pi conversation. This answers task-22's modeling question. Sideshow mockups and the full discussion trail: http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8

## Decided
- `kind` values: `epic`, `research`, `papercut`. A Task with no kind is a standard Task. Tags go back to being topical only.
- No standing Epics or container records. Classification comes from kind alone.
- Required at `add` (deliberately tiny): papercut needs only a title, with acceptance optional; task and research keep title + acceptance.
- `kind: papercut` defaults to `priority: low` (can be overridden).
- Papercut placement: top level or under an Epic, **never a Subtask**. Research can sit anywhere.
- Archived Logs stay as they are and keep their legacy `research`/`papercut` tags. No code reads those tags as kinds.
- No tag fallback: everything keyed on the `research`/`papercut` tags switches to kind in this change.
- The TUI stays static; lanes belong in `tandem web` (separate Task).

## Out of scope (owned by ~/.pi)
- Intake triage rubric and follow-up questions
- The intake verdict (an `## Intake` body section written by Pi)
- Kind-specific Worker/Subagent rules
- Running all Tasks of one kind

