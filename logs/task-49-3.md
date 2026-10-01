---
id: task-49-3
uid: db66f124-790d-4100-8d99-2d85ae0545de
type: task
title: "Step 3: record identity (hidden UUIDs, temporary labels, numbering at first sync)"
priority: "high"
effort: "medium"
parentId: "task-49"
tags: ["protocol", "sync"]
accord:
  status: "accepted"
  acceptance: ["Every new Task, Subtask, Epic, Decision, and Rule gets a permanent uid; uid and published IDs cannot be changed by update or direct edit without a held validation error.", "Unsynced records use provisional handles (task-new-<uid prefix>) that work in every command; ambiguous prefixes are reported, never guessed.", "Publishing assigns the next sequential numbers against the fetched shared board (parents before Subtasks), renames files, and rewrites every structured reference and exact prose occurrence."]
  claimedAt: "2026-09-30T04:51:40Z"
  deliveredAt: "2026-09-30T04:51:40Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml"]
  summary: "Every record gets a permanent uid; boards with a remote create provisional <prefix>-new-<hex> IDs that publication numbers against the shared board (parents before Subtasks), renaming files and rewriting references and event ledgers; outdated provisional IDs resolve through the uid prefix."
  evidence: ["`cargo test --manifest-path tandem/Cargo.toml`: 372 passed, 0 failed (318 unit + 54 integration, including 11 two-clone sync scenarios in tandem/tests/sync_behavior.rs).", "Unit tests: provisional_ids_round_trip_and_reject_lookalikes, numbering_continues_sequences_and_numbers_parents_before_subtasks, token_replacement_respects_id_boundaries, provisional_records_are_numbered_and_references_rewritten.", "Integration: offline_work_is_saved_with_a_temporary_id_and_numbered_on_reconnect (task-new-* -> task-2, child -> task-2-1, old handle still resolves); rules_and_subtasks_from_two_machines_get_distinct_numbers (always-2, task-1-2)."]
  filesChanged: ["tandem/src/protocol/ids.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/app/support.rs", "tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/app/rules.rs", "tandem/src/project/rules.rs", "tandem/src/project/mod.rs"]
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

