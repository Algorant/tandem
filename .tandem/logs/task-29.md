---
id: task-29
type: task
title: "Reference links: accept absolute URLs and scope unresolved warnings to the Board"
priority: "medium"
effort: "medium"
references: ["task-2"]
relatedFiles: ["tandem/src/app/queries.rs", "tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/protocol/document.rs", "tandem/src/web.rs", "tandem/src/web/ui.js", "tandem/src/cli/commands.rs", "tandem/tests/reference_behavior.rs", "tandem/Cargo.toml", "tandem/Cargo.lock", "protocol/README.md", "protocol/assignment.md", "docs/cli/index.md", "docs/guides/decisions.md"]
tags: ["protocol", "validation", "ui"]
accord:
  status: "accepted"
  acceptance: ["A reference holding an absolute http(s) URL produces no warning; an unresolved document-ID reference (for example missing-task) still warns.", "Board-scoped reads emit no 'references missing target' warnings for archived Logs records.", "Web detail renders URL references as external links and document-ID references as internal #document/<id> links.", "Protocol and CLI docs state that references accepts document IDs and absolute URLs, and that repo paths belong in relatedFiles.", "just dev-check passes with tests covering URL acceptance, unresolved-ID warning, and Log exclusion."]
  claimedAt: "2026-09-10T18:00:13Z"
  deliveredAt: "2026-09-10T18:14:03Z"
  validation: ["$ just dev-check"]
  constraints: ["Keep URL reference classification semantics in protocol; app queries compose location-scoped warning behavior and web renders links.", "Apply the same URL classification to existing Task add/update and Decision add reference warnings; supersedes and blockers remain document-ID-only.", "Use one shared protocol classifier and expose web-only referenceLinks value/external metadata without changing CLI/assignment references arrays. Recognize valid absolute HTTP(S) URLs with a host, not merely a scheme prefix; never fetch URLs.", "Do not modify TUI code; task-30 owns the Board References row.", "No adapter changes or workspace data cleanup."]
  summary: "Delivered task-29 with the requested documentation correction. Code (commit 52e5d4c): one protocol classifier `is_absolute_reference_url` in tandem/src/protocol/document.rs (url-crate parser; trim; explicit case-insensitive http:// or https://; reject embedded whitespace/control; require non-empty authority; require parsed http/https scheme and a host; never fetch or rewrite). `load_read` reference warnings now scan only DocumentLocation::Board (tasks under tasks/ and decisions under decisions/) and skip URL values, while unresolved document IDs still warn and archived Logs never warn. The same URL skip is applied to Task add/update and Decision add mutation warnings; supersedes/blockers remain document-ID-only. Web detail exposes web-only `referenceLinks` [{value, external}] from the protocol classifier; ui.js consumes that field directly and renders external links with target=_blank rel=\"noopener noreferrer\" and everything else as percent-encoded internal #document/<value> (so non-http(s) values can never become raw hrefs). Docs correction (commit b755441): docs/cli/index.md decision `--reference` now says repository paths belong in `relatedFiles` metadata, not `references`, and that the current decision CLI lacks `--related-file` with adding it out of scope; docs/guides/decisions.md now states the same directly. Other docs (protocol/README.md, protocol/assignment.md) already carried the relatedFiles split. Acknowledged: the approved one-line `add decision --json` warning projection is the only unlisted-file change and is now in durable relatedFiles; I will ask before touching an unlisted file again. Remaining gaps: browser DOM rendering is supplemental chromium evidence (just dev-check does not run JS), and the ffsync fixture was not present in this checkout so behavior is verified with synthetic URL/ID/malformed fixtures."
  evidence: ["A reference holding an absolute http(s) URL produces no warning; an unresolved document-ID reference (for example missing-task) still warns.: Protocol classifier unit test accepts http/https with query/fragment/uppercase/localhost/port/IPv6 and rejects ftp/mailto/relative/js plus empty-authority and malformed-port forms. tandem/tests/reference_behavior.rs asserts Task add/update and Decision add warn only for unresolved IDs and never for URL values, and that show --json read warnings contain only the unresolved IDs. Browser fixture confirmed javascript:alert(1) and https:///path still warn and render internally.", "Board-scoped reads emit no 'references missing target' warnings for archived Logs records.: queries.rs test load_read_scopes_reference_warnings_to_board_documents asserts no 'task-9 references missing target' warning from an archived Log with unresolved references, no warning when a Board task references archived task-9, and a still-warning Board Decision missing-decision; CLI test archived_logs_never_warn_and_board_references_resolve_to_logs covers the same end-to-end.", "Web detail renders URL references as external links and document-ID references as internal #document/<id> links.: web.rs test detail_projects_reference_links_with_protocol_classification asserts referenceLinks [(decision-1,false),(https://example.com/artifacts/42,true),(HTTPS://Example.COM/Upper,true),(missing-task,false)] with data.references unchanged. Chromium DOM emitted <a href=\"https://example.com/artifacts/42\" target=\"_blank\" rel=\"noopener noreferrer\">https://example.com/artifacts/42</a>, <a href=\"#document/task-2\">task-2</a>.", "Protocol and CLI docs state that references accepts document IDs and absolute URLs, and that repo paths belong in relatedFiles.: protocol/README.md states references accepts document IDs and absolute http(s) URLs, URLs are never fetched/rewritten/warned, repo paths are relatedFiles metadata, and warnings cover Board only. protocol/assignment.md notes references values are IDs or URLs. docs/cli/index.md add/update text states the vocabulary; its decision --reference text and docs/guides/decisions.md both now direct repository paths to relatedFiles metadata and note the decision CLI lacks --related-file (adding it out of scope).", "just dev-check passes with tests covering URL acceptance, unresolved-ID warning, and Log exclusion.: just dev-check exited 0 with 265+3+6+9+6+3 passing and 0 failing, plus both test_dev.sh PASS lines. New coverage: classifies_only_absolute_well_formed_http_reference_urls, queries load_read reference tests, web.rs detail_projects_reference_links_with_protocol_classification, and three reference_behavior CLI tests."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/src/protocol/document.rs", "tandem/src/app/queries.rs", "tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/web.rs", "tandem/src/web/ui.js", "tandem/src/cli/commands.rs", "tandem/tests/reference_behavior.rs", "protocol/README.md", "protocol/assignment.md", "docs/cli/index.md", "docs/guides/decisions.md"]
  updatedAt: "2026-09-10T18:14:03Z"
createdAt: "2026-09-10T17:54:37Z"
updatedAt: "2026-09-10T18:14:03Z"
assignee: "worker-task-29-b95826a2"
archivedAt: "2026-09-10T18:14:03Z"
resolution:
  outcome: "completed"
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
