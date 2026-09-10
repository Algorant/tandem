---
id: task-29
type: task
title: "Reference links: accept absolute URLs and scope unresolved warnings to the Board"
state: "in-progress"
priority: "medium"
effort: "medium"
references: ["task-2"]
relatedFiles: ["tandem/src/app/queries.rs", "tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/protocol/document.rs", "tandem/src/web.rs", "tandem/src/web/ui.js", "tandem/src/cli/commands.rs", "tandem/tests/reference_behavior.rs", "tandem/Cargo.toml", "tandem/Cargo.lock", "protocol/README.md", "protocol/assignment.md", "docs/cli/index.md", "docs/guides/decisions.md"]
tags: ["protocol", "validation", "ui"]
accord:
  status: "delivered"
  acceptance: ["A reference holding an absolute http(s) URL produces no warning; an unresolved document-ID reference (for example missing-task) still warns.", "Board-scoped reads emit no 'references missing target' warnings for archived Logs records.", "Web detail renders URL references as external links and document-ID references as internal #document/<id> links.", "Protocol and CLI docs state that references accepts document IDs and absolute URLs, and that repo paths belong in relatedFiles.", "just dev-check passes with tests covering URL acceptance, unresolved-ID warning, and Log exclusion."]
  claimedAt: "2026-09-10T18:00:13Z"
  deliveredAt: "2026-09-10T18:11:59Z"
  validation: ["$ just dev-check"]
  constraints: ["Keep URL reference classification semantics in protocol; app queries compose location-scoped warning behavior and web renders links.", "Apply the same URL classification to existing Task add/update and Decision add reference warnings; supersedes and blockers remain document-ID-only.", "Use one shared protocol classifier and expose web-only referenceLinks value/external metadata without changing CLI/assignment references arrays. Recognize valid absolute HTTP(S) URLs with a host, not merely a scheme prefix; never fetch URLs.", "Do not modify TUI code; task-30 owns the Board References row.", "No adapter changes or workspace data cleanup."]
  summary: "Delivered task-29. Reference values are now classified in one protocol helper, `is_absolute_reference_url` (tandem/src/protocol/document.rs), backed by the `url` crate: trim, require explicit case-insensitive `http://`/`https://`, reject embedded whitespace/control characters, require a non-empty authority (so `https://`, `https:///path`, `https://?q=x`, `https://#fragment` are not URLs), then parse and require scheme http/https plus a host. Stored text is preserved exactly; nothing is fetched or canonicalized. Read warnings in `load_read` now scan only DocumentLocation::Board (tasks under tasks/ plus decisions under decisions/) and skip URL values, so unresolved document IDs still warn and archived Logs never warn. The same URL skip is applied to mutation-time warnings for Task add/update and Decision add; supersedes/blockers stay document-ID-only. Web detail now carries a web-only `referenceLinks` [{value, external}] array computed with the protocol classifier; ui.js consumes that field directly and renders external entries as `<a href target=_blank rel=\"noopener noreferrer\">` and everything else as percent-encoded internal `#document/<value>` links, so non-http(s) values can never become raw hrefs. Docs updated in the four required files only; task-30's TUI file untouched. Remaining gaps/notes: (1) I also changed `add decision --json` to emit the warnings the app already computed (it hardcoded []), because the approved Decision add warning behavior is otherwise unobservable at the CLI; (2) browser DOM rendering is verified by supplemental chromium headless evidence, not by just dev-check; (3) the ffsync fixture itself was unavailable in this checkout, so the reported reproduction is confirmed from code rather than reproduced locally."
  evidence: ["A reference holding an absolute http(s) URL produces no warning; an unresolved document-ID reference (for example missing-task) still warns.: Protocol classifier test accepts http/https with query/fragment/uppercase/localhost/port/IPv6 and rejects ftp/mailto/relative/js and malformed empty-authority/port cases. CLI test task_add_and_update_skip_url_references_but_warn_missing_ids asserts add and update warnings omit URL values but include 'reference not found: missing-task'/'other-missing', and show --json read warnings are only for the unresolved IDs. decision_add_skips_url_references_but_warns_missing_ids covers the Decision add path. Browser check showed malformed https:///path and javascript:alert(1) still warn and render internally.", "Board-scoped reads emit no 'references missing target' warnings for archived Logs records.: queries.rs test load_read_scopes_reference_warnings_to_board_documents writes an archived Log (task-9) with unresolved reference gone-task and asserts no warning contains 'task-9 references missing target'; it also asserts a Board Decision with missing-decision still warns and a Board task referencing archived task-9 does not warn. CLI test archived_logs_never_warn_and_board_references_resolve_to_logs repeats this end-to-end (cancel task-1, reference it from task-2).", "Web detail renders URL references as external links and document-ID references as internal #document/<id> links.: web.rs test detail_projects_reference_links_with_protocol_classification asserts referenceLinks = [(decision-1,false),(https://example.com/artifacts/42,true),(HTTPS://Example.COM/Upper,true),(missing-task,false)] while data.references stays the unchanged string array. chromium headless DOM dumped <a href=\"https://example.com/artifacts/42\" target=\"_blank\" rel=\"noopener noreferrer\">https://example.com/artifacts/42</a>, <a href=\"#document/task-2\">task-2</a>.", "Protocol and CLI docs state that references accepts document IDs and absolute URLs, and that repo paths belong in relatedFiles.: protocol/README.md Documents section states references accepts document IDs and absolute http(s) URLs, URLs are never fetched/rewritten/warned, repo paths are relatedFiles metadata, and warnings cover Board only. protocol/assignment.md notes references values are IDs or URLs and are not dependencies. docs/cli/index.md add/update/decision --reference text states the vocabulary and --related-file split. docs/guides/decisions.md frontmatter and status sections state the same.", "just dev-check passes with tests covering URL acceptance, unresolved-ID warning, and Log exclusion.: just dev-check exited 0; cargo test ran bin unit tests 265 passed and integration suites 3+6+9+6+3 passed with 0 failed; scripts/tests/test_dev.sh printed both PASS lines. New coverage lives in protocol::document::tests::classifies_only_absolute_well_formed_http_reference_urls, queries tests, web.rs detail_projects_reference_links_with_protocol_classification, and tandem/tests/reference_behavior.rs (3 tests)."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/src/protocol/document.rs", "tandem/src/app/queries.rs", "tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/web.rs", "tandem/src/web/ui.js", "tandem/src/cli/commands.rs", "tandem/tests/reference_behavior.rs", "protocol/README.md", "protocol/assignment.md", "docs/cli/index.md", "docs/guides/decisions.md"]
  updatedAt: "2026-09-10T18:11:59Z"
createdAt: "2026-09-10T17:54:37Z"
updatedAt: "2026-09-10T18:11:59Z"
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
