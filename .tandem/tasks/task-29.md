---
id: task-29
type: task
title: "Reference links: accept absolute URLs and scope unresolved warnings to the Board"
state: "in-progress"
priority: "medium"
effort: "medium"
references: ["task-2"]
relatedFiles: ["tandem/src/app/queries.rs", "tandem/src/protocol/", "tandem/src/web/ui.js", "tandem/tests/", "protocol/README.md", "protocol/assignment.md", "docs/cli/index.md", "docs/guides/decisions.md"]
tags: ["protocol", "validation", "ui"]
accord:
  status: "claimed"
  acceptance: ["A reference holding an absolute http(s) URL produces no warning; an unresolved document-ID reference (for example missing-task) still warns.", "Board-scoped reads emit no 'references missing target' warnings for archived Logs records.", "Web detail renders URL references as external links and document-ID references as internal #document/<id> links.", "Protocol and CLI docs state that references accepts document IDs and absolute URLs, and that repo paths belong in relatedFiles.", "just dev-check passes with tests covering URL acceptance, unresolved-ID warning, and Log exclusion."]
  claimedAt: "2026-09-10T18:00:13Z"
  validation: ["$ just dev-check"]
  constraints: ["Keep URL reference classification semantics in protocol; app queries compose location-scoped warning behavior and web renders links.", "Do not modify TUI code; task-30 owns the Board References row.", "No adapter changes or workspace data cleanup."]
  updatedAt: "2026-09-10T18:00:13Z"
createdAt: "2026-09-10T17:54:37Z"
updatedAt: "2026-09-10T18:00:13Z"
assignee: "worker-task-29-b95826a2"
---

## Description

## Description

Tandem 0.13.0 warns `"<id> references missing target <value>."` for every `references` entry that does not resolve to a document (`tandem/src/app/queries.rs`). The scan covers active Board records and archived Logs, so the same warnings appear on every workspace read (`tandem show`, `tandem assignment`, web), regardless of which record an operation touches.

Reproduction from ffsync: `decision-4` references a repo fixture path, and archived `task-20`, `task-31`, `task-32`, `task-46` reference Sideshow post URLs. Verified against the installed 0.13.0 binary: `tandem show task-55 --json` returns exactly those five warnings; `tandem list --json` returns none.

Two defects:

1. External artifact links have no structured home. Sideshow posts are normal workflow evidence, but body prose never appears in the References row, so evidence links are invisible to the UI.
2. Warnings are effectively unfixable. Archived Logs are immutable by design, and `tandem update` rejects decisions ("only task documents can be updated in v0", tracked in task-2) and archived records ("active task not found"). A single legacy non-document reference therefore warns forever with no CLI remedy.

The web UI confirms the current document-ID-only assumption: every reference renders as an internal `#document/<value>` link (`tandem/src/web/ui.js`).

## Decision (Algorant approved)

1. `references` accepts document IDs and absolute `http(s)` URLs. Document IDs remain validated so unresolved ones still warn (typo detection). URLs are opaque loose links and are never fetched; validation stays offline and fast. Repo paths remain `relatedFiles` path metadata, which is unvalidated.
2. Reference validation covers active Board documents only. Archived Logs are immutable history and must not emit missing-target warnings.
3. Web detail renders absolute URLs as external links; document IDs keep internal `#document/<id>` navigation.
4. Docs state the accepted vocabulary and the `relatedFiles` split.

## Required changes

- `tandem/src/app/queries.rs`: classify reference values, skip warnings for absolute `http://`/`https://` URLs, and restrict the reference scan to Board-location documents.
- `tandem/src/web/ui.js`: render URL-valued references as external links in the References row.
- `protocol/README.md`, `protocol/assignment.md`, `docs/cli/index.md`, `docs/guides/decisions.md`: document the vocabulary.
- Tests: URL accepted without warning; unresolved document ID still warns; archived Logs excluded.

## Out of scope

- TUI Board References row (follow-up task; Decisions and Papercuts panels already show references).
- Type-aware decision update (task-2).
- Workspace data cleanup. Once this lands, URL references stop warning and no edits are needed; `decision-4`'s fixture path should move to `relatedFiles` when decision editing exists (task-2) or by manual edit.
