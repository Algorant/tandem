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
  status: "delivered"
  acceptance: ["TUI Board task detail renders a References row listing document IDs and URL values", "Rendering handles a record with no references without adding an empty row", "just dev-check passes"]
  claimedAt: "2026-09-10T18:00:26Z"
  deliveredAt: "2026-09-10T18:15:11Z"
  validation: ["$ just dev-check"]
  constraints: ["Display-only plain text References row; no OSC-8 work, validation, storage, or lifecycle changes.", "Keep implementation and rendering regression tests in tandem/src/tui/board/mod.rs; Decisions and Papercuts are styling references, not mutation scope."]
  summary: "Delivered the display-only References row plus the requested localized test-fixture correction and formatting. Edits remain confined to tandem/src/tui/board/mod.rs.\n\nPRODUCTION CHANGE (commit 95d8185)\n- `detail_lines_for_doc_with_context` now emits `push_optional_detail_list_line(&mut lines, \"References\", doc.values(\"references\"), theme)` immediately after the Tags row and before the Parent block. Reuses the existing helper so the row has the same label/value spans as every other detail field, joins with \", \" like Papercuts/Decisions, and emits nothing for an empty/missing list. No OSC-8, validation, storage, or lifecycle changes.\n\nTEST-FIXTURE CORRECTION (commit a94016e, per parent request)\n- Removed the hand-built `temp_workspace` fixture (obsolete `.tandem/board`, `protocolVersion: 0.1.0`, `events.jsonl`, empty root/data_dir).\n- Replaced with `initialized_workspace(root)` calling `TandemProject::initialize(root, &crate::protocol::config::default_project_config(\"Reference wrap test\"))` (valid 0.3.0 workspace: tasks/decisions/rules/logs/events) and using its real `tasks_dir`.\n- The wrapping test's task now carries a valid accord (`status: ready` plus an `acceptance` list).\n- The three tests remain: populated ID+URL row with label/value style assertions; absent field and `references: []` both suppress the row; long URL rendered through the production `app.draw_detail` widget, reconstructed across border-stripped wrapped rows, proving the whole URL survives and no single row contains it.\n\nFORMAT\n- `cargo fmt --manifest-path tandem/Cargo.toml` applied; `cargo fmt --manifest-path tandem/Cargo.toml --check` exits 0. Only board/mod.rs was reformatted (the new test list literal).\n\nVALIDATION\n- Targeted `cargo test --manifest-path tandem/Cargo.toml board_detail`: 7 passed / 0 failed, including all three `tui::board::tests::board_detail_*` tests.\n- `just dev-check` exit 0: 265 unit tests passed / 0 failed; integration suites 3+6+9+6 passed / 0 failed; sandbox smoke `PASS: assignment freshness, evidence rejection, milestone batching, and Git boundary checkpoint.` Only the pre-existing `package.readme` Cargo warning appears.\n- `cargo test --no-run` produced no unused/board-module warnings.\n\nSTATE\n- HEAD a94016e on worker-task-30-tui-board-detail-show-a-references-r; working tree clean. No push, merge, or worktree cleanup performed. Parent still owns release rendered-pane verification in the routed preview sandbox."
  evidence: ["TUI Board task detail renders a References row listing document IDs and URL values: detail_lines_for_doc_with_context emits the row via push_optional_detail_list_line; test board_detail_renders_reference_ids_and_urls passes asserting 'References: task-29, https://github.com/Algorant/tandem/issues/30'.", "Rendering handles a record with no references without adding an empty row: Test board_detail_omits_references_row_when_absent passes for both a missing field and `references: []` (no line starts with 'References:').", "just dev-check passes: Ran `just dev-check`; exit 0 with 265 unit tests passed/0 failed, integration suites 3+6+9+6 passed/0 failed, and smoke 'PASS: assignment freshness, evidence rejection, milestone batching, and Git boundary checkpoint.'", "Test fixture no longer depends on obsolete internals: temp_workspace removed; initialized_workspace uses TandemProject::initialize + default_project_config (0.3.0) and project.tasks_dir; wrapping task includes accord status/acceptance. Tests pass against this real project layout.", "Formatting is clean: cargo fmt --manifest-path tandem/Cargo.toml applied; cargo fmt --manifest-path tandem/Cargo.toml --check exits 0 with only board/mod.rs affected.", "Edits confined to board/mod.rs: git status shows only tandem/src/tui/board/mod.rs changed; commits 95d8185 and a94016e touch no other file."]
  filesChanged: ["tandem/src/tui/board/mod.rs"]
  updatedAt: "2026-09-10T18:15:11Z"
createdAt: "2026-09-10T17:54:53Z"
updatedAt: "2026-09-10T18:15:11Z"
assignee: "worker-task-30-256b6a3d"
---

## Description

## Description

Follow-up from task-29 (reference links: accept absolute URLs and scope unresolved warnings to the Board).

The TUI Board task detail shows Type, Kind, Role, State, Priority, Effort, Assignee, Due, Tags, and Parent, but never `references` (`tandem/src/tui/board/mod.rs`). The Decisions view and Papercuts panel already render references; Board tasks do not. Once task-29 makes absolute URLs legitimate reference values, the Board detail should expose them too.

Plain text rendering is acceptable; an OSC-8 hyperlink for URL values is optional and must not regress the existing Decisions/Papercuts styling. This row is display-only and must not change validation, storage, or lifecycle behavior.
