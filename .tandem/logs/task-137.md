---
id: task-137
type: task
title: "Implement mixed event reads, ordering, and corruption diagnostics"
priority: "high"
blockers: ["task-135"]
references: ["decision-6", "decision-7"]
relatedFiles: ["tandem/src/main.rs", "tandem/src/tui.rs", "tandem/src/tui/logs.rs"]
tags: ["rust", "tui", "events", "compatibility", "diagnostics"]
createdAt: "2026-07-15T19:43:46Z"
updatedAt: "2026-07-30T13:05:49Z"
parentId: "task-133"
completedAt: "2026-07-30T13:05:49Z"
completion:
  outcome: "canceled"
  summary: "Canceled: Superseded by decision-9 after the per-actor event ledger shipped in d974133. Existing mixed legacy/per-actor readers and diagnostics remain the selected baseline."
---

## Description

This is a direct Task of Epic task-133. Upgrade CLI/TUI event consumers to read decision-6's mixed legacy/new single-ledger format safely.

Acceptance criteria:
- Parse new UUIDv7-identified records and legacy records without `eventId` from the same `.tandem/events.jsonl` file without rewriting history.
- Deduplicate new records only when the same `eventId` has an identical payload.
- Surface the same `eventId` with differing payloads as explicit corruption; do not silently choose one.
- Never interpret physical JSONL line order as event chronology; use timestamp and event ID for deterministic new-record ordering with a documented stable fallback for legacy ties.
- Preserve malformed-line warnings and ensure one bad record does not hide unrelated valid history where safe.
- Update Logs/history rendering and hot-reload fingerprints as needed without reintroducing per-actor directory aggregation.
- Add focused tests for arbitrary union order, duplicate merges, conflicting payloads, legacy-only files, mixed files, equal timestamps, malformed lines, empty ledgers, and event context in completed Logs.
