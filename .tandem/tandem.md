---
protocolVersion: "0.2.0"
type: workspace
title: "tandem"
states:
  - id: todo
    title: To Do
  - id: in-progress
    title: In Progress
  - id: validation
    title: Validation
rules:
  always:
    - id: 6
      rule: "Use Bun as the default package manager and script runner for JavaScript/TypeScript project automation, including docs-site local recipes and CI, unless a concrete incompatibility makes Bun impractical."
      source: "decision-2"
    - id: 7
      rule: "When cutting a Tandem release, create and push both the annotated git tag and the GitHub Release object for that tag; do not treat a pushed tag alone as a complete release unless explicitly asked for tag-only."
      source: "task-15"
  never:
    - id: 1
      rule: "Do not mark a newly created task as claimed, delivered, validation, accepted, or completed unless the user explicitly asked to start or finish the work; automated tests or smoke checks are evidence only, not permission to advance lifecycle state."
      source: "User correction 2026-06-30"
  prefer:
    - id: 1
      rule: "Use one primary area tag first: `protocol`, `tui`, `pi-tandem`, `docs`, `config`, `rules`, or `ui`."
      source: "task-22"
    - id: 2
      rule: "Add only a few capability/workflow tags when they aid delegation, such as `accord`, `review`, `logs`, `editor`, `relationships`, `delegation`, `taxonomy`, `smoke`, `validation`, or concrete TUI facets like `theme`, `keyboard`, `mouse`, and `markdown`."
      source: "task-22"
    - id: 3
      rule: "When prototyping documentation site features, themes, layouts, or diagrams, create quick Sideshow mockups first wherever practical; use those previews to narrow direction before committing implementation or durable design decisions."
      source: "User request 2026-06-29"
    - id: 4
      rule: "For delegated non-visual/non-manual work with passing automated validation and no blockers, the orchestrator may accept and complete/log the Tandem task without waiting for human validation; keep visual, UX, manual, high-risk, or ambiguous work in validation for human review."
      source: "User guidance 2026-06-30"
    - id: 5
      rule: "When orchestrating parallel or potentially overlapping delegated work, decide up front whether each worker needs an isolated branch/worktree so changes remain independently reviewable and committable. Use separate worktrees for likely file overlap, visual/design experiments, release automation, or any task expected to commit independently; avoid multiple workers editing the same working tree unless the work is read-only or explicitly coordinated."
      source: "User guidance 2026-06-30"
    - id: 6
      rule: "Commit durable `.tandem` changes regularly. Do not leave important board, decision, rule, or completed-work changes only in a local working tree for an extended period. Regular commits make work visible to collaborators and agents, preserve tasks across clones and worktrees, protect state from resets or cleanup, and keep Tandem history reasonably aligned with project history. Use judgment about commit boundaries: include Tandem changes in a related project commit, group related changes, or create a standalone coordination commit. A separate commit is useful when changes would otherwise remain uncommitted, represent planning without implementation, or need to be visible to other participants. Do not require one commit for every Tandem command. Never commit checkout-local identity, caches, credentials, or other runtime state."
      source: "User guidance 2026-07-31"
  context:
    - id: 1
      rule: "For delegated TUI or visual work, configure the repository's single Git-local preview slot so the user runs only `just dev` from the normal checkout; route it to the delegated code and a safe fixture, report no extra command/setup, and clear the route during cleanup."
      source: "task-132"
    - id: 2
      rule: "For Tandem Board-only visual validation, use tab 2 of the current orchestrator Herdr workspace: run the configured `just dev` preview there and inspect the TUI directly. Continue without pausing for human validation when it looks correct; escalate only when behavior is off, ambiguous, or requires product preference."
      source: "User guidance 2026-07-28"
---

# tandem
