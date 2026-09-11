---
id: task-2
type: task
title: "CLI: update rejects decision documents"
state: "in-progress"
priority: "low"
tags: ["protocol", "papercut", "decisions"]
accord:
  status: "claimed"
  acceptance: ["Common update supports active Decision documents alongside existing Tasks; status, deciders, supersedes and common title/body/tags/references/relatedFiles metadata are editable with correct type-specific validation.", "Entering accepted/rejected decision status maintains automatic decidedAt plus updatedAt according to a documented plan-approved rule; invalid statuses or inappropriate type-specific flags fail without writes.", "Repeated list flags replace lists, supported --clear operations work, unknown metadata and body bytes are preserved when unrelated, and no-op updates do not write events or timestamps.", "Decision URL/document-ID references retain task-29 semantics; repo paths can be stored as relatedFiles through decision updates without being treated as document references.", "CLI success reports actual changed fields and failures retain standard envelope/exit behavior. Regression tests exercise real native decision update, timestamp rules, lists/clears, invalid/no-op requests and existing Task update behavior."]
  claimedAt: "2026-09-11T14:27:36Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml --check"]
  constraints: ["Wait for task-16 to integrate before launch; it currently owns cli/commands.rs. No concurrent edits to that file.", "Protocol meaning first; app owns shared mutations, CLI dispatches by resolved document type rather than ID shape.", "Decision update only: do not expand common update to Rules or add new types/compatibility paths; do not refactor unrelated dead decision code unless directly replaced by the authoritative update path.", "Normative protocol0.3.0 and current task-29 reference semantics are authoritative; existing broader CLI docs contain stale command shapes, do not rewrite the whole docs suite.", "Submit detailed plan and timestamp/clear/type-validation design before editing, and ask about ambiguity instead of silently choosing a divergent interpretation.", "No adapter or TUI changes."]
  updatedAt: "2026-09-11T14:27:36Z"
createdAt: "2026-08-31T20:53:43Z"
updatedAt: "2026-09-11T14:27:36Z"
effort: "medium"
references: ["task-3", "task-29"]
relatedFiles: ["tandem/src/cli/commands.rs", "tandem/src/cli/model.rs", "tandem/src/app/decisions.rs", "tandem/src/app/tasks.rs", "tandem/src/protocol/document.rs", "tandem/tests/decision_update_behavior.rs", "protocol/README.md", "docs/cli/index.md", "docs/guides/decisions.md"]
assignee: "worker-task-2-a5c95c01"
---

## Description

Migrating to protocol 0.3.0 surfaced this: 'tandem update decision-8 --status accepted' fails with 'only task documents can be updated in v0'. Contract D34 requires type-aware update; decision status/deciders/supersedes must be editable through common update. Migration workaround: frontmatter authoring.
