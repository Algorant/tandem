---
id: task-222
type: task
title: "Specify and implement the lightweight Papercuts inbox MVP"
priority: "medium"
relatedFiles: ["protocol/plan/spec.md", "protocol/plan/todo.md", "tandem/plan/spec.md", "tandem/plan/todo.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/plan/todo.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "plan/papercuts.md", "docs/cli/index.md", "docs/concepts/index.md", "docs/workspace/index.md", "docs/guides/agents-and-adapters.md", "docs/extensions/index.md", "docs/reference/index.md", "site/astro.config.mjs", "site/README.md"]
tags: ["protocol", "papercuts", "pi-tandem"]
createdAt: "2026-08-10T02:09:53Z"
updatedAt: "2026-08-10T03:02:36Z"
accord:
  status: "accepted"
  assignee: "worker-task-222-86fee578"
  claimedAt: "2026-08-10T02:31:36Z"
  deliveredAt: "2026-08-10T03:02:21Z"
  validation:
    commands: ["cargo test --manifest-path tandem/Cargo.toml --no-fail-fast (246 unit and 11 integration tests passed)", "cargo clippy --manifest-path tandem/Cargo.toml --all-targets -- -D warnings (passed in Worker checkout)", "cargo fmt --manifest-path tandem/Cargo.toml -- --check (passed in Worker checkout)", "bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts (passed)", "bun extensions/pi-tandem/tests/smoke.ts (passed after integration)", "cd site && bun run check:docs (19 pages and 912 links passed in Worker checkout)"]
  summary: "Implemented and integrated the lightweight Papercuts inbox MVP across core Tandem, pi-tandem, protocol specifications, user documentation, and site content."
  evidence: ["Integrated Worktrunk squash commit c7f682a", "Parent review found and Worker corrected Task/Decision/Rule reference boundaries in commit 3ea9d1b before integration", "Integrated checkout is clean and full Rust plus pi-tandem smoke validation passed"]
  filesChanged: ["README.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md", "tandem/README.md", "tandem/plan/spec.md", "tandem/plan/todo.md", "tandem/src/protocol/papercut.rs", "tandem/src/project/mod.rs", "tandem/src/app/papercuts.rs", "tandem/src/app/queries.rs", "tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/app/rules.rs", "tandem/src/cli/args.rs", "tandem/src/cli/commands.rs", "tandem/src/cli/output.rs", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "docs/cli/index.md", "docs/concepts/index.md", "docs/workspace/index.md", "docs/guides/agents-and-adapters.md", "docs/extensions/index.md", "docs/reference/index.md"]
  reviewer: "pi-orchestrator"
  note: "Accepted after parent diff review, targeted rework of Papercut reference boundaries, Worktrunk integration, 246 unit and 11 integration tests, and passing pi-tandem smoke validation."
  updatedAt: "2026-08-10T03:02:27Z"
assignee: "worker-task-222-86fee578"
completedAt: "2026-08-10T03:02:36Z"
completion:
  summary: "Implemented the Papercuts inbox MVP with protocol-owned records, CLI add/list/show/resolve, global search and audit events, thin pi-tandem support, tests, specifications, and website documentation."
  filesChanged: ["README.md", "protocol/README.md", "protocol/plan/spec.md", "protocol/plan/todo.md", "tandem/README.md", "tandem/plan/spec.md", "tandem/plan/todo.md", "tandem/src/protocol/papercut.rs", "tandem/src/project/mod.rs", "tandem/src/app/papercuts.rs", "tandem/src/app/queries.rs", "tandem/src/app/tasks.rs", "tandem/src/app/decisions.rs", "tandem/src/app/rules.rs", "tandem/src/cli/args.rs", "tandem/src/cli/commands.rs", "tandem/src/cli/output.rs", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "docs/cli/index.md", "docs/concepts/index.md", "docs/workspace/index.md", "docs/guides/agents-and-adapters.md", "docs/extensions/index.md", "docs/reference/index.md"]
  validation: "Parent review completed; Worker full validation passed; integrated cargo test passed 246 unit and 11 integration tests; pi-tandem Bun check and smoke passed."
  reviewer: "pi-orchestrator"
---
## Description

## Outcome

Add a small, project-local Papercuts inbox that works immediately in existing Tandem workspaces. Papercuts record small, non-blocking friction without creating Tasks, entering Board workflow, or using the Accord lifecycle.

This Task explicitly includes both the core Tandem work and the corresponding thin `pi-tandem` adapter work. Keep the adapter CLI-only and implement normative behavior in Tandem first.

## Product definition

A Papercut is small, non-blocking friction encountered during work that caused confusion, avoidable retries, unnecessary effort, or a workaround worth preserving.

Papercuts can describe misleading instructions, missing helpers, awkward workflows, surprising tool contracts, or recurring low-level friction.

A Papercut does not replace a Task, blocker, Accord state, Decision, Rule, or general telemetry. If work is blocked, use the existing blocking lifecycle. If corrective work needs planning and ownership, create a Task and reference the Papercut.

## MVP protocol contract

### Storage

- Store one lightweight record per file under `.tandem/papercuts/papercut-N.md`.
- Create `.tandem/papercuts/` lazily on the first add.
- Treat Papercuts as protocol-owned inbox records, not general Tandem documents.
- Do not add `papercut` to the general document-type taxonomy.
- Do not place Papercuts in `.tandem/board/` or `.tandem/logs/`.
- Preserve unknown frontmatter and body content during mutation.
- Existing workspaces without the directory must continue to work unchanged and must not require migration solely for this additive capability.

### Record shape

Required CLI-created fields:

```yaml
id: papercut-12
title: Edit errors do not identify ambiguous replacements
status: open
createdAt: 2026-08-01T12:00:00Z
updatedAt: 2026-08-01T12:00:00Z
```

Optional fields:

```yaml
references:
  - task-173
tags:
  - tooling
```

The Markdown body holds optional context, evidence, impact, or a workaround.

Resolved records add:

```yaml
status: resolved
resolution:
  note: The error now lists the ambiguous source locations.
  resolvedAt: 2026-08-03T09:30:00Z
```

Rules:

- IDs use the immutable sequential `papercut-N` form.
- Allocation scans existing Papercut records and does not intentionally reuse IDs.
- Status is `open` or `resolved` in the MVP.
- References are loose relationships. They do not create hierarchy.
- Unresolved references warn rather than fail.
- Duplicate titles are permitted.
- There is no deletion lifecycle.

## MVP CLI surface

```text
tandem papercut add --title <text> [--body <markdown>] [--reference <id>]... [--tag <tag>]...
tandem papercut list [--status <open|resolved>] [--all] [--json]
tandem papercut show <id> [--json]
tandem papercut resolve <id> --note <text> [--reference <id>]...
```

Behavior:

- `list` shows open Papercuts by default.
- `--status` selects one status and `--all` includes both statuses.
- `show` returns the full frontmatter projection, Markdown body, and path.
- `resolve` updates the record in place, requires a concise note, and may add references such as the Task that addressed the friction.
- Read commands support structured JSON using normal Tandem envelopes.
- Mutations use Tandem-owned atomic write and event behavior.

## Search and audit

- Include Papercuts in global `tandem search` results.
- Search title, body, status, tags, references, and resolution note.
- Identify the result location as `papercuts`.
- Do not include Papercuts in `tandem list`, `tandem log`, Board views, hierarchy, completion progress, review, or Accord behavior.
- Append `papercut.created` and `papercut.resolved` audit events.
- Papercut files remain the current-state source of truth. Events remain audit-only.

## Task promotion in the MVP

Do not add a dedicated promotion command. Promotion uses existing commands and references:

1. Create a normal Task that references the Papercut.
2. Resolve the Papercut with a note and a reference to the new Task.

A future command may make this atomic if real usage justifies it.

## pi-tandem adapter authorization and guidance

Add a thin `tandem_papercut` tool in `extensions/pi-tandem/` with these actions:

- `add`
- `list`
- `show`
- `resolve`

Requirements:

- Translate tool parameters to Tandem CLI argument arrays.
- Use CLI JSON for read actions where supported.
- Do not parse or mutate Papercut Markdown in TypeScript.
- Do not allocate IDs, resolve references, or implement status behavior in the adapter.
- Add focused adapter smoke coverage.
- Keep global Pi configuration and external Worker/handoff implementations out of this Task.

Runtime guidance:

> Record a Papercut when small, non-blocking friction causes confusion, avoidable retries, or a workaround worth preserving. Then continue the current work.

Agents should use judgment. A failed tool call is only a signal. Expected test failures, empty searches, and deliberate invalid probes are not automatically Papercuts.

Worker handoffs may mention Papercut IDs in existing free-form summaries. Structured handoff schema changes are deferred.

## Implementation sequence

1. Update normative protocol and CLI specifications.
2. Implement protocol parsing, validation, storage, allocation, and events.
3. Implement shared app operations and CLI commands.
4. Extend global search and JSON/human output.
5. Add core tests for normal, malformed, missing-directory, reference, search, resolution, and allocation cases.
6. Implement the explicitly authorized `pi-tandem` tool as a thin CLI adapter.
7. Add adapter smoke coverage and concise runtime guidance.
8. Update relevant README and planning documentation.

Preserve the established ownership boundaries: protocol owns meaning, project owns concrete filesystem access, app owns shared operations, and CLI/TUI are consumers. No TUI work is required for this MVP.

## Acceptance criteria

1. `papercut add` lazily creates the directory and one valid record.
2. Separate additions create separate files with unique sequential IDs without overwriting existing records.
3. Existing workspaces without Papercuts continue to work without migration.
4. Default `papercut list` shows only open records.
5. Status filters and `--all` return the expected records in human and JSON output.
6. `papercut show` returns metadata, body, and path.
7. `papercut resolve` preserves the file and adds valid resolution metadata.
8. Resolved records remain available through `--all`, `show`, and global search.
9. Global search finds Papercut content and reports location `papercuts`.
10. Unresolved references warn, while malformed required Papercut structure fails validation without affecting unrelated Board operations.
11. Audit events are written through the existing actor-owned event mechanism.
12. Papercuts never appear as Board items, Logs, hierarchy members, or completion progress.
13. `tandem_papercut` exercises add/list/show/resolve through the installed CLI without direct protocol handling.
14. Focused core and adapter tests pass.

## Explicitly excluded from the MVP

Keep these as future considerations. Do not implement them unless usage evidence or a later Task justifies the added complexity:

- automatic capture of failed tool calls;
- automatic detection, heuristics, or telemetry ingestion;
- duplicate detection or merging;
- severity, priority, assignment, due dates, or Accord lifecycle;
- comments, attachments, or threaded discussion;
- a dedicated TUI view;
- deletion, dismissal outcomes, or reopening;
- an atomic `papercut promote` command;
- automatic structured insertion into Worker handoffs, validation records, or Logs;
- cross-workspace aggregation;
- a global Pi-system inbox or fallback storage outside Tandem workspaces;
- an extension event bus for other producers;
- dashboards, metrics, scoring, or trend analysis.

## Supplementary context

The manually maintained `plan/papercuts.md` demonstrates the current ad hoc pattern: non-blocking findings are recorded separately so primary work can continue. The MVP should make that behavior durable, searchable, agent-accessible, and consistent without turning the inbox into another work-management system.

## Documentation and site adaptation

Documentation is part of the MVP deliverable, not a follow-up. Update each authoritative layer after the behavior is implemented:

### Normative and implementation specifications

- Update `protocol/plan/spec.md` with the Papercut purpose, optional workspace area, record contract, validation, references, events, search behavior, and the boundary from general document types.
- Update `tandem/plan/spec.md` with complete CLI syntax, human output, JSON envelopes, errors, and search integration.
- Update `protocol/plan/todo.md` and `tandem/plan/todo.md` so planning state matches implementation.
- Update `extensions/pi-tandem/plan/spec.md` and its todo with the thin `tandem_papercut` tool contract and guidance.

### User documentation and website

Canonical site content lives under `docs/`; generated copies under `site/src/content/docs/` must not be edited or committed.

- Add the Papercut commands and examples to `docs/cli/index.md`.
- Explain the inbox concept and its boundary from Tasks and blockers in `docs/concepts/index.md`.
- Add `.tandem/papercuts/` and its file shape to `docs/workspace/index.md`.
- Document agent judgment and the `tandem_papercut` adapter flow in `docs/guides/agents-and-adapters.md`.
- Add the new Pi tool to `docs/extensions/index.md`.
- Update `docs/reference/index.md` or add a focused Papercuts reference page if the final content is too large for the existing reference page. If a page is added, update `site/astro.config.mjs` navigation.
- Update root and area READMEs only where they enumerate supported concepts, commands, workspace layout, or adapter tools.
- Do not add a TUI workflow or imply that Papercuts appear on the Board.

### Documentation validation

- Run `cd site && bun run check:docs` after canonical `docs/` changes.
- Confirm generated Markdown under `site/src/content/docs/` remains ignored and uncommitted.
- Check command examples against the implemented CLI rather than documenting proposed syntax that differs from the final behavior.
- Confirm internal links, headings, and navigation pass the site check.

Documentation acceptance requires that a new user can discover what a Papercut is, record one, list and inspect it, resolve it, link it to a Task, find it through search, and understand what the MVP intentionally does not do.
