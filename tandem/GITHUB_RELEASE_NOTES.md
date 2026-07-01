# Tandem v0.4.0

Concise public notes for the `tandem-v0.4.0` GitHub Release. Keep reusable release validation and install checklist details in `tandem/RELEASE.md`.

## Highlights

- Added lightweight Epic support using normal task documents with `kind: epic`.
- Added ADR-compatible Decision metadata and CLI handling.
- Improved the TUI Board with configurable default badges and grouped Epic arrangement.
- Improved the TUI Decisions view with cleaner metadata/detail rendering and compact expandable list rows.
- Documented Git-safer per-actor event log direction for future protocol work.

## Protocol

- Added the `kind: epic` task convention for lightweight planning/grouping tasks while preserving `type: task`, `task-N` IDs, normal workflow states, accords, review metadata, and completion behavior.
- Clarified parent/child semantics for epics: child work links through `parentId`, while looser associations continue to use `references`.
- Strengthened Decision records as Tandem's ADR-compatible durable decision type without adding a separate `adr` type.
- Clarified Decision metadata such as `status`, `date`, `deciders`, `context`, `consequences`, `alternatives`, `supersedes`, and `supersededBy`; Decision `status` is metadata, not task workflow state.
- Documented the planned per-actor event log shape under `.tandem/events/<actor>.jsonl`.

## CLI

- Added `tandem add --kind epic` for creating lightweight epic tasks.
- Added `tandem update <id> --kind epic` for marking existing tasks as epics.
- Extended Decision CLI handling with ADR-style metadata flags for `tandem decision add`, including status/date/deciders/context/consequences/alternatives/supersession fields.
- Improved Decision list/show output and JSON metadata exposure for ADR-compatible fields.

## TUI

- Added a default, action-oriented Board badge set for priority, work-type tags, visual validation, attention statuses, and subtask progress.
- Added badge configuration for style modes, opt-in tag badges, and badge suppression through the existing user/workspace TUI config stack.
- Added Board rendering for `kind: epic` tasks and a grouped Epic arrangement that can nest child tasks under their parent epic without noisy parent-id chips.
- Added Enter-expanded Epic/child relationship context while preserving the normal Board workflow states.
- Improved the Decisions view so Decision records render as decisions rather than unfiled board work.
- Improved Decision detail rendering for ADR metadata and body content.
- Refined the Decision list into compact rows with minimal selection and Enter expansion for additional preview details.

## Docs

- Added `docs/guides/decisions.md` for ADR-compatible Decision usage.
- Updated protocol, CLI, TUI, concepts, reference, and integration docs for epics, decisions, badges, and event-log direction.
- Updated release-note guidance to group public notes by product surface when releases include distinct protocol, CLI, TUI, docs, or integration work.

## Integrations

- Updated `pi-tandem` guidance and smoke coverage for epic and ADR-compatible Decision fields while keeping protocol behavior in the Tandem CLI.

## Install

```text
cargo install --git git@github.com:Algorant/tandem.git --tag tandem-v0.4.0 --path tandem --locked
tandem --version
```

## Notes

- No binary artifacts are published yet; install from the git tag with Cargo.
- Mutation commands remain human-readable only; structured JSON mutation output is deferred.
- TUI visual polish remains active work; Board and Decisions UX are improved in this release, with richer interactions still evolving.
