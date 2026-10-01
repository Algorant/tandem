---
id: task-49-5
uid: 462bcd0a-7625-4ade-a6dd-a9429e6c0573
type: task
title: "Step 5: semantic merge and conflict resolution"
priority: "high"
effort: "large"
parentId: "task-49"
tags: ["protocol", "sync"]
accord:
  status: "accepted"
  acceptance: ["Three-way merge by uid implements the design's field, list, body, lifecycle, archive, updatedAt, config, and event-ledger rules, and validates the merged graph.", "A contradictory record is held with base, local, and remote preserved while all other changes publish; tandem sync resolve <id> --keep local|remote|edited resolves it; mutations to a held record are refused with a clear message.", "Archive on one machine plus edit on another asks which to keep. Invalid direct edits are held locally with file, line, and reason. No conflict markers are ever written into a record."]
  claimedAt: "2026-09-30T04:51:40Z"
  deliveredAt: "2026-09-30T04:51:40Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml"]
  summary: "Record-level three-way merge by uid (tandem/src/protocol/merge.rs plus sync.rs merge_entry): field and nested-field merge, set-like lists, later updatedAt, three-way body merge without markers, ledger extension only, archive-vs-edit conflict, merged-board validation with held culprits; conflicts preserved and resolved with tandem sync resolve --keep local|remote|edited; invalid hand edits held."
  evidence: ["`cargo test --manifest-path tandem/Cargo.toml`: 372 passed, 0 failed (318 unit + 54 integration, including 11 two-clone sync scenarios in tandem/tests/sync_behavior.rs).", "Unit: combines_independent_fields_sets_and_timestamps, same_field_changed_differently_is_a_conflict, bodies_use_the_text_merge_and_never_emit_markers, ledgers_merge_only_by_extension, archive_versus_edit_is_a_conflict_that_keeps_the_remote.", "Integration: compatible_edits_to_one_record_combine, contradictory_edits_stop_only_that_record_until_resolved (mutation refused until resolved; unrelated change still synced), completing_on_one_machine_while_editing_on_another_asks, direct_markdown_edits_sync_and_broken_ones_are_held."]
  filesChanged: ["tandem/src/protocol/merge.rs", "tandem/src/project/sync.rs", "tandem/src/cli/commands.rs"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-09-30T04:51:40Z"
createdAt: "2026-09-30T03:54:17Z"
updatedAt: "2026-09-30T04:51:40Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-30T04:51:40Z"
resolution:
  outcome: "completed"
  reviewer: "pi-orchestrator"
---

