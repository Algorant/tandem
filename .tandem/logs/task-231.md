---
id: task-231
type: task
title: "Diagnose tandem web snapshot loading regression and design a robust fix"
priority: "high"
effort: "small"
relatedFiles: ["tandem/src/app/queries.rs", "tandem/src/project/mod.rs", "tandem/src/web.rs"]
tags: ["ui", "web", "performance"]
createdAt: "2026-08-21T03:26:28Z"
updatedAt: "2026-08-21T03:35:57Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-08-21T03:26:34Z"
  deliveredAt: "2026-08-21T03:35:50Z"
  deliverables: ["Reproduction and measured performance evidence", "Root-cause analysis identifying O(R × D) repeated document parsing", "Architectural design comparison and recommendation", "Future synchronized-server context and design constraints recorded in task-231"]
  validation:
    commands: ["Static web assets returned successfully while API snapshot requests stalled", "Reference-free 258-document copy returned /api/v1/project in about 297 ms", "Unchanged workspace exceeded the 30-second API timeout", "Verified no implementation diff remains in tandem/src"]
  summary: "Diagnosed the tandem web loading failure, measured the scaling regression, mapped architectural constraints, compared fix designs, and documented the recommended coherent snapshot plus server-owned coordinator direction. No implementation code was changed."
  evidence: ["258 documents and 250 reference targets imply about 64,500 repeated document reads/parses per snapshot", "Regression traced to Papercut target support replacing in-memory hierarchy lookup with per-reference project scans", "Independent read-only architecture audit agreed with the diagnosis and D2+D3 recommendation"]
  filesChanged: [".tandem/board/task-231.md", ".tandem/events/3ba2bee6-e75b-41ec-8124-f68506643fea.jsonl"]
  reviewer: "user"
  note: "User confirmed the diagnostic and architecture-design task is done for now."
  updatedAt: "2026-08-21T03:35:53Z"
assignee: "pi"
completedAt: "2026-08-21T03:35:57Z"
completion:
  summary: "Diagnosed the tandem web snapshot loading regression, measured its O(R × D) behavior, compared architectural fixes, and documented the coherent project snapshot plus server-owned coordinator recommendation and future synchronization context."
  filesChanged: [".tandem/logs/task-231.md", ".tandem/events/3ba2bee6-e75b-41ec-8124-f68506643fea.jsonl"]
  validation: "User reviewed the diagnostic outcome and confirmed task-231 is done for now."
  reviewer: "user"
---
## Scope

Diagnostic and architecture analysis only. Do not change implementation until the owner reviews the findings and selects a design.

## Reproduction

- Static HTML, CSS, and JavaScript routes return normally.
- `/api/v1/project` does not return within 30 seconds on the real project workspace.
- `tandem list --json` returns in about 62 ms because it does not run the long-read reference warning path.
- A copied 258-document workspace with all `references:` fields removed returns `/api/v1/project` in about 297 ms. The unchanged copy still exceeds 30 seconds.
- The current workspace has 250 loose reference targets across 258 documents, causing an estimated 64,500 full document reads/parses per snapshot.

## Root cause

`app::queries::load_read` acquires the hierarchy lock and loads all Board and Log documents into `ProjectHierarchy`. It then checks every loose reference with `TandemProject::reference_target_exists`. That method calls `find_document`, which rereads and reparses every Board and Log document for every reference before checking for a Papercut filename. The resulting reference phase is O(R × D), not O(R + D).

The web path amplifies this cost. Initial loading independently requests project and attention snapshots, then loads the route snapshot. Synchronous filesystem and lock work runs directly in async handlers. The exclusive lock is held across parsing and diagnostics, so concurrent reads serialize and can block Tokio workers.

The regression entered when Papercut target support replaced the existing in-memory hierarchy lookup in `load_read` with `project.reference_target_exists`.

## Correctness constraints

- Loose references can target Board documents, Logs, or Papercuts.
- Papercuts remain outside the general document hierarchy.
- Papercut existence must use canonical filenames without parsing records, so malformed Papercuts do not break unrelated Board/web reads.
- Missing loose references remain warnings.
- One snapshot must be coherent across hierarchy, warnings, and revision.
- Papercut filename changes that affect reference warnings must affect snapshot revision.

## Future direction supplied by the owner

The intended longer-term product is a fully synchronized server that updates in real time while agents run and can be monitored remotely. After that foundation is solid, later capabilities may include creating Tasks and sending feedback to active agents or active work.

This is context, not current implementation scope. The read-only repair should preserve a path toward:

- one authoritative server-side workspace projection rather than independent request-local snapshots;
- ordered change streams with explicit revision or cursor semantics;
- reconnect and catch-up behavior rather than browser-only polling assumptions;
- transport-independent app operations shared by CLI, TUI, local web, and a future remote service;
- an explicit mutation command boundary suitable for later authentication, authorization, audit, idempotency, and optimistic concurrency;
- agent-run and feedback concepts modeled separately from filesystem watching or HTTP transport.

Do not add remote binding, authentication, live agent control, mutation endpoints, WebSockets, SSE, or database/sync infrastructure as part of the current repair.

## Designs considered

1. Query-local document ID set plus Papercut filename checks. Small tactical fix, but leaves duplicated snapshots, blocking async I/O, and weak API boundaries.
2. Project-owned coherent read snapshot containing config, Board/Log documents, and canonical Papercut filename IDs. App builds hierarchy, warning indexes, and revision from that immutable snapshot. Correct ownership and linear O(D + R + P) work.
3. Web-level cached/singleflight snapshot coordinator using `Arc<ReadSnapshot>`, bounded blocking work, cheap fingerprints, and explicit lock timeout/busy behavior. Fixes duplicate endpoint loads and Tokio starvation, but must sit on top of design 2.
4. Shared read locks. May reduce contention but does not fix repeated parsing or async blocking and is not sufficient.

## Recommendation

Use design 2 as the canonical foundation and add design 3 for the long-running web interface. Keep the changes separately reviewable: first establish one coherent project/app snapshot and linear reference validation; then add web snapshot deduplication, bounded blocking execution, and refresh semantics.

Treat the server-side snapshot coordinator as the future synchronization seam. Keep it independent of Axum DTOs and browser state so a later ordered event stream or remote service can publish the same canonical projection without moving protocol meaning into the web layer.

## Proposed validation

- Board, Log, missing, and canonical Papercut reference cases.
- Existing malformed Papercut filename counts as present without parsing; malformed Papercut content still fails only Papercut-specific reads.
- App snapshot construction performs no filesystem reads after the project snapshot is captured.
- Papercut filename addition/removal changes revision.
- Concurrent `/project` and `/attention` requests share one snapshot build.
- Lock contention returns a bounded busy/error result instead of hanging.
- Static assets remain responsive during a blocked refresh.
- Generated-project benchmark demonstrates near-linear scaling in documents, references, and Papercut filenames.