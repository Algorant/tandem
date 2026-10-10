---
id: task-62
uid: 838c6f00-4180-4205-b18a-6154027f96a6
type: task
title: "A validation-invalid local edit blocks every command and has no supported repair"
priority: "medium"
effort: "medium"
relatedFiles: ["tandem/src/project/sync.rs", "tandem/src/cli/commands.rs", "tandem/tests/sync_behavior.rs"]
tags: ["protocol", "validation"]
accord:
  status: "accepted"
  acceptance: ["With one validation-invalid record held on a Git-backed board, reads list the other records with a warning that names the held file, and unrelated mutations still succeed (task-49-10 parity).", "`tandem sync status` lists validation-held edits with their reason, matching what `tandem sync` reports.", "A supported native command (e.g. `sync resolve <id> --keep remote`) restores a validation-held record to the shared version without hand-editing `.tandem/`.", "Regression tests in tandem/tests/sync_behavior.rs cover the three behaviors above."]
  claimedAt: "2026-10-10T14:09:47Z"
  deliveredAt: "2026-10-10T14:24:53Z"
  validation: ["$ just dev-check"]
  summary: "The branch is rebased onto main 550467b and the held-record set is now per-instance state owned by `TandemProject`. The `static HELD_RECORDS`, `set_held_records` and `held_record_error` are gone, with no global fallback. Behavior and messages are unchanged, and the previous ready report's \"Behavior for task-66 to document\" section still applies verbatim. `just dev-check` exits 0 and clippy is clean. Two commits sit on 550467b: a435238 (the fix, rebased) and 902fac1 (the ownership refactor).\n\n## What changed in 902fac1\n- **`project/write.rs`:**\n  - New `HeldRecords`, a `Default + Clone` wrapper around `Arc<Mutex<BTreeMap<path, (id, reason)>>>`. Clones of a project share one set.\n  - Its `replace`, `error_for_path` and `error_for_id` replace the global functions.\n  - `read_file_snapshot(project, path)` now takes the project and refuses held paths through `project.held`.\n  - The two unit tests in `write.rs` pass a project.\n- **`project/mod.rs`:** `TandemProject` gains `held: write::HeldRecords`. `read_effective` fills it with `self.held.replace(..)`, and `refuse_held` consults `self.held.error_for_id`. `with_paths` initializes it.\n- **Call sites:** all ten `read_file_snapshot(&doc.path)` calls now pass the project. These are accord (2), decisions (2), links (1), review (1) and tasks (4).\n- **Struct literals:** the test-only `TandemProject { .. }` literals in `app/review_contract_tests.rs`, `tui/mod.rs` and `tui/theme.rs` got `held: Default::default()`.\n\nMutations call `hierarchy_from_project(project)`, which fills the same instance's set. Each guarded write then runs `read_file_snapshot` on that instance. The guard stays in the shared project layer, so the CLI, TUI and web app ops all get it.\n\n## Behavior (unchanged)\nThe behavior contract for task-66 is as stated in the previous ready report. Summary:\n- **`sync status`:** lists validation holds in the status text as `Held edit: <path>: would make the shared board invalid: <msg>` plus `  resolve: tandem sync resolve <id> --keep remote`. JSON `held[]` entries are `{path, reason, id}`.\n- **`sync resolve <id> --keep remote`:** restores a validation-held record's shared version, or removes a record that was never shared. `--keep local|edited` fail with the guidance message.\n- **Mutating a held record:** refused with `<id> has a local edit held from sync (<reason>). Restore the shared version with `tandem sync resolve <id> --keep remote` before changing it.`\n\n## Validation\n- **Rebase base:** 550467b (\"Guard update --clear parent against canonical role changes\", task-61).\n- **`just dev-check`:** exit 0. Unit tests: 337 passed. All integration files ok, including 19 in `sync_behavior`, which still contains the five new task-62 tests. The `test_dev.sh` smoke PASS lines appear.\n- **`cargo clippy --all-targets`:** clean. `cargo fmt` applied.\n- **Static check:** `grep` for `HELD_RECORDS`, `set_held_records` and `held_record_error` over `tandem/src` returns 0 hits."
  evidence: ["With one validation-invalid record held on a Git-backed board, reads list the other records with a warning that names the held file, and unrelated mutations still succeed (task-49-10 parity).: a_validation_invalid_edit_is_held_and_other_work_continues still passes on the per-instance guard: `list` and `show --json` warn with the file name, `add` and `update` of other records succeed, and `update`, `complete` and `accord claim` on the held record are refused with the resolve guidance and leave the file bytes unchanged.", "`tandem sync status` lists validation-held edits with their reason, matching what `tandem sync` reports.: sync_status_lists_validation_holds_like_sync_does passes: `sync status` JSON `held` equals the `held` from `sync` (path, reason, id), and the status text prints the Held edit line and the resolve line.", "A supported native command (e.g. `sync resolve <id> --keep remote`) restores a validation-held record to the shared version without hand-editing `.tandem/`.: sync_resolve_keep_remote_restores_a_validation_held_record and a_held_new_record_is_skipped_and_resolve_removes_it pass: `--keep remote` restores the shared bytes (or removes a record that was never shared), and `--keep local|edited` fail with the guidance.", "Regression tests in tandem/tests/sync_behavior.rs cover the three behaviors above.: The five new tests (including held parent with children, and held new record) are unchanged and pass. The file has 19 passing tests.", "Ownership correction: held state lives on TandemProject, no global: `TandemProject.held` is a `write::HeldRecords` instance field, filled by `read_effective` and consulted by `read_file_snapshot(project, path)` and `refuse_held`. The static, `set_held_records` and `held_record_error` are removed (grep returns no matches). All ten mutation call sites pass the project."]
  filesChanged: ["tandem/src/project/write.rs", "tandem/src/project/mod.rs", "tandem/src/project/sync.rs", "tandem/src/cli/commands.rs", "tandem/src/app/accord.rs", "tandem/src/app/decisions.rs", "tandem/src/app/links.rs", "tandem/src/app/review.rs", "tandem/src/app/tasks.rs", "tandem/src/app/review_contract_tests.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/theme.rs", "tandem/tests/sync_behavior.rs"]
  updatedAt: "2026-10-10T14:24:54Z"
createdAt: "2026-10-03T03:11:28Z"
updatedAt: "2026-10-10T14:24:54Z"
assignee: "worker-task-62-b417d4a2"
archivedAt: "2026-10-10T14:24:54Z"
resolution:
  outcome: "completed"
---

## Description


## Description

Found with task-337 on the ~/.pi board, and reproduced on 0.16.2 on 2026-10-03.

When a local record parses but fails board validation (for example a subtask whose parent was cleared), sync holds it with "would make the shared board invalid: …". Unlike the unparsable-file case fixed by task-49-10, every later command (`show`, unrelated `update`, and even the update that would make the record valid again) fails on that validation error. `tandem sync resolve <id> --keep remote` refuses with "no open sync conflict for <id>", so the only repair is restoring the file by hand from the `tandem` branch, which agents may not do.

Related inconsistency: `tandem sync` reports the held edit and says "see `tandem sync status`", yet `sync status` shows `held: []`. `project/sync.rs` `status()` returns only the snapshot's unparsable-file holds, not the validation holds that merge adds as `extra_held`.

