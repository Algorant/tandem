---
id: task-94
uid: b22ebbd6-332f-44db-9db9-128b0aeb9ffb
type: task
title: "Bulk JSON board export with an updated-time window"
priority: "medium"
effort: "medium"
references: ["task-93"]
relatedFiles: ["tandem/src/cli/model.rs", "tandem/src/cli/commands.rs", "tandem/src/app/queries.rs", "tandem/src/app/dto.rs", "docs/cli/index.md"]
tags: ["protocol", "cli", "export"]
accord:
  status: "ready"
  acceptance: ["A native `tandem export --json --scope active|archived|all` command (default all) opens/validates one board read snapshot and returns complete Task/Decision details in one existing {ok,data,warnings} envelope; it does not invoke show or reload the snapshot per record.", "Each exported record reuses the authoritative full detail projection and includes stable UID, role/parent relationships, body/metadata, Accord/counts/resolution, original archival/terminal timestamps (including archivedAt and decision decidedAt where applicable), plus workspace identity and record-location/relative-path provenance without credentials or guessed host identity.", "Optional --since and --until RFC3339 bounds filter updatedAt in process before full projection, with inclusive lower/exclusive upper bounds and timezone-aware comparisons. Invalid bounds/ranges fail clearly; missing or invalid record timestamps are surfaced explicitly rather than silently treated as a fabricated time. No filter exports all records in scope.", "Returned records preserve deterministic ordering and full relationship/context projection even when related records fall outside the time window. Archived records are read-only; optional audit history may enrich details but never becomes mandatory or rewrites record history.", "Short real-CLI E2E checks on generated temporary boards verify active/archived detail parity, terminal/provenance fields, time-window boundaries and empty results; source/performance verification confirms one snapshot load rather than N show calls. Formatting/build/lint pass; no unit tests, mocks, permanent fixtures or test harnesses.", "Publish the exact command/JSON/time semantics and shipped version to Herdra/obsmd when released; durable change cursor, pagination and historical event-window reconstruction remain outside this initial cut."]
  validation: ["$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Capture/scoping request only: leave unstarted until explicit execution direction. Concrete Worker plan before implementation when started.", "Keep this independent of held task93; do not recover/resume/edit/discard its Worker or compiling seam. Implement from clean main in its own isolated Task checkout when authorized.", "V1 time filter is explicitly updatedAt, which satisfies the requested updated-or-terminal filter; include original terminal timestamp fields for obsmd's own finer filtering. No invented terminal-event semantics, durable cursor, new log command, cache/service or broad snapshot/TUI refactor.", "Use shared app snapshot/detail projections and one snapshot open; no per-record CLI, timestamp subprocess or duplicate Markdown/frontmatter parser.", "E2E behavioral checks only under current global rule; compiler/fmt/lint are separate checks. Temporary generated state only, no real-board mutations or archived-record edits."]
  updatedAt: "2026-10-26T02:34:39Z"
createdAt: "2026-10-26T02:34:39Z"
updatedAt: "2026-10-26T02:36:50Z"
archivedAt: "2026-10-26T02:36:50Z"
resolution:
  outcome: "canceled"
  note: "superseded: Algorant chose a raw .tandem file reader in obsmd's devlog (2026-10-26)"
---

## Description

Approved feature request from Algorant via Herdra mf5a67bc7. obsmd's devlog harvest issued2,991 tandemshow calls (desktop1,347/x1nano1,006/archbox638) while only662records were in the requested window; eachshow loads and validates the whole board. Source confirms compactlist onlyid/title, no time/cursor filter, and nativeReadSnapshot/detail projections already exist.

Scope: `tandem export --json --scope all [--since RFC3339] [--until RFC3339]`, with existing active/archived/all Scope enum and defaultall. One shared read snapshot and filtered full records under data.records plus workspace/filter provenance. Reuse/extend app-owned detail DTO for terminal timestamps/UID/provenance, preserving role/hierarchy/Accord/resolution and existing warnings. Apply updatedAt window before expensive per-record projection; derive relationships against the FULL loaded snapshot, not only matched rows. State inclusiveSince/exclusiveUntil and missing-time diagnostics precisely in help/docs. No silent truncation; no durable cursor/pagination now. Event-time historical queries are not promised by an updated-time filter; original terminal metadata is returned explicitly for consumers.

This is a standalone CLI reader/performance feature, NOT permission to resume held SSH task93 or merge its interim snapshot code. No source work started by filing. When executed, obtain a bounded plan, verify with real CLI E2E generated boards (no units/mocks), ship through normal authorized release gates and notify Herdra so obsmd can adopt the final contract. No release date/version promised yet.
