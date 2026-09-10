---
id: task-30
type: task
title: "TUI Board detail: show a References row"
state: "in-progress"
priority: "low"
effort: "small"
references: ["task-29"]
relatedFiles: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/papercuts.rs"]
tags: ["tui"]
accord:
  status: "rework"
  acceptance: ["TUI Board task detail renders a References row listing document IDs and URL values", "Rendering handles a record with no references without adding an empty row", "just dev-check passes"]
  claimedAt: "2026-09-10T18:00:26Z"
  deliveredAt: "2026-09-10T18:12:35Z"
  validation: ["$ just dev-check"]
  constraints: ["Display-only plain text References row; no OSC-8 work, validation, storage, or lifecycle changes.", "Keep implementation and rendering regression tests in tandem/src/tui/board/mod.rs; Decisions and Papercuts are styling references, not mutation scope."]
  summary: "Implemented the display-only References row in the TUI Board task detail and committed it.\n\nCHANGE (only tandem/src/tui/board/mod.rs, +158 lines)\n1. In `detail_lines_for_doc_with_context`, immediately after the `Tags` row and before the Parent block:\n   `push_optional_detail_list_line(&mut lines, \"References\", doc.values(\"references\"), theme);`\n   This reuses the existing helper (same label/value spans as every other detail row), joins values with \", \" like Papercuts/Decisions, and emits nothing when the value list is empty. No OSC-8, no validation/storage/lifecycle change.\n2. Added `#[cfg(test)] mod tests` at the end of board/mod.rs (local `line_text`, temp-workspace, and border-stripping row helpers) with three regression tests:\n   - `board_detail_renders_reference_ids_and_urls`: asserts the exact row `References: task-29, https://github.com/Algorant/tandem/issues/30` and that span 0/secondary styles equal `theme.label_style()` / `theme.text_style()` (no style regression).\n   - `board_detail_omits_references_row_when_absent`: covers both a missing `references` field and `references: []`, asserting no line starts with `References:`.\n   - `board_detail_wraps_long_reference_url`: loads a temp workspace, renders the real detail widget via `app.draw_detail(frame, frame.area())` at 46x24, strips the pane border, reconstructs the full 111-char URL across wrapped rows (whitespace removed) and asserts the whole URL survives, while also asserting no single row contains it (proving wrap rather than truncation).\n\nRESULTS\n- Targeted: `cargo test --manifest-path tandem/Cargo.toml board_detail` -> 7 passed, 0 failed (includes all 3 new tests).\n- Native validation `just dev-check` -> exit 0: 265 unit tests passed / 0 failed, integration suites 3+6+9+6 passed / 0 failed, and the sandbox smoke check printed `PASS: assignment freshness, evidence rejection, milestone batching, and Git boundary checkpoint.` Only pre-existing `package.readme` Cargo warning appeared.\n\nREMAINING GAPS / NOTES\n- Release rendered-pane verification is parent-owned; `/tmp/tandem-reference-preview.fPvxkd` was already seeded and routed, and task-29 semantics are not required for this display path.\n- Committed as 95d8185 on worker-task-30-tui-board-detail-show-a-references-r; working tree clean. No push, merge, or worktree cleanup performed."
  evidence: ["TUI Board task detail renders a References row listing document IDs and URL values: detail_lines_for_doc_with_context now calls push_optional_detail_list_line(\"References\", doc.values(\"references\")). Test board_detail_renders_reference_ids_and_urls passes and asserts the exact line 'References: task-29, https://github.com/Algorant/tandem/issues/30'.", "Rendering handles a record with no references without adding an empty row: push_optional_detail_list_line only pushes non-empty lists; Document::values returns [] for missing field and `[]`. Test board_detail_omits_references_row_when_absent passes for both cases (no line starts with 'References:').", "just dev-check passes: Ran `just dev-check`; exit code 0. Output: 265 unit tests passed/0 failed, integration suites 3+6+9+6 passed/0 failed, and 'PASS: assignment freshness, evidence rejection, milestone batching, and Git boundary checkpoint.'", "Regression test covers wrapping of a long URL reference: Test board_detail_wraps_long_reference_url passes: renders via app.draw_detail at 46x24, reconstructs the full URL from border-stripped rows, and asserts the whole URL survives while no single row contains it.", "No references-related style regression: Test asserts references.spans[0].style == theme.label_style() and references.spans[1].style == theme.text_style(), identical to detail_field_line used by other rows.", "Only tandem/src/tui/board/mod.rs mutated: git status showed a single modified file before commit; commit 95d8185 contains only tandem/src/tui/board/mod.rs (+158 lines)."]
  filesChanged: ["tandem/src/tui/board/mod.rs"]
  note: "Production one-line change and behavior tests are sound; parent independently ran just dev-check successfully. One localized test-fixture correction before landing: new temp_workspace copies obsolete .tandem/board + protocolVersion 0.1.0 + events.jsonl and constructs TandemProject with empty root/data_dir. Do not add new fixtures depending on obsolete internals. Replace with TandemProject::initialize(&root, valid 0.3.0 config), use its tasks_dir, and give the test task a valid accord acceptance/status. Exis"
  updatedAt: "2026-09-10T18:12:35Z"
createdAt: "2026-09-10T17:54:53Z"
updatedAt: "2026-09-10T18:12:35Z"
assignee: "worker-task-30-256b6a3d"
---

## Description

## Description

Follow-up from task-29 (reference links: accept absolute URLs and scope unresolved warnings to the Board).

The TUI Board task detail shows Type, Kind, Role, State, Priority, Effort, Assignee, Due, Tags, and Parent, but never `references` (`tandem/src/tui/board/mod.rs`). The Decisions view and Papercuts panel already render references; Board tasks do not. Once task-29 makes absolute URLs legitimate reference values, the Board detail should expose them too.

Plain text rendering is acceptable; an OSC-8 hyperlink for URL values is optional and must not regress the existing Decisions/Papercuts styling. This row is display-only and must not change validation, storage, or lifecycle behavior.
