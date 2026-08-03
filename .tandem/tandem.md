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
    - id: 8
      rule: "During core Tandem work, specify needed cross-framework behavior in Tandem-owned protocol or guidance documents and create an explicit implementation handoff when an adapter change is needed."
    - id: 9
      rule: "Before orchestrating parallel or potentially overlapping delegated work, assess whether each worker needs an isolated branch or worktree."
      source: "User guidance 2026-06-30"
    - id: 10
      rule: "For delegated TUI or visual work, configure the repository's Git-local preview slot so the user runs only `just dev` from the normal checkout; route it to delegated code and a safe fixture, report no extra setup, and clear the route during cleanup."
      source: "task-132"
    - id: 11
      rule: "For Tandem Board-only visual validation, use tab 2 of the current orchestrator Herdr workspace: run the configured `just dev` preview there and inspect the TUI directly. Continue without pausing for human validation when it looks correct; escalate only when behavior is off, ambiguous, or requires product preference."
      source: "User guidance 2026-07-28"
  never:
    - id: 1
      rule: "Do not mark a newly created task as claimed, delivered, validation, accepted, or completed unless the user explicitly asked to start or finish the work; automated tests or smoke checks are evidence only, not permission to advance lifecycle state."
      source: "User correction 2026-06-30"
    - id: 2
      rule: "Do not modify `extensions/pi-tandem/`, external Pi configuration, or any other agent/framework adapter implementation as part of core Tandem work for now. Specify generic, framework-neutral behavior in Tandem-owned protocol or guidance documents, and create explicit handoff documents for adapter maintainers when implementation changes are needed."
    - id: 3
      rule: "Do not treat a pushed Git tag alone as a complete Tandem release unless the caller explicitly requests tag-only."
      source: "task-15"
    - id: 4
      rule: "Do not accept or complete delegated visual, UX, manual, high-risk, or ambiguous work without human review; keep it in validation."
      source: "User guidance 2026-06-30"
    - id: 5
      rule: "Never commit checkout-local identity, caches, credentials, or other runtime state."
      source: "User guidance 2026-07-31"
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
      rule: "Prefer separate worktrees for likely file overlap, visual/design experiments, release automation, or independently committed work; prefer a shared tree only for read-only or explicitly coordinated work."
      source: "User guidance 2026-06-30"
    - id: 6
      rule: "Commit durable `.tandem` changes regularly. Do not leave important board, decision, rule, or completed-work changes only in a local working tree for an extended period. Use judgment about commit boundaries: include Tandem changes in a related project commit, group related changes, or create a standalone coordination commit. Do not require one commit for every Tandem command."
      source: "User guidance 2026-07-31"
    - id: 7
      rule: "Prefer Bun as the package manager and script runner for JavaScript/TypeScript automation, including docs-site recipes and CI. Use another tool when a concrete incompatibility makes Bun impractical."
      source: "decision-2"
  context:
    - id: 1
      rule: "For delegated TUI or visual work, configure the repository's single Git-local preview slot so the user runs only `just dev` from the normal checkout; route it to the delegated code and a safe fixture, report no extra command/setup, and clear the route during cleanup."
      source: "task-132"
    - id: 2
      rule: "For delegated non-visual, non-manual work with passing automated validation and no blockers, the orchestrator is authorized to accept and complete the task without additional human validation."
      source: "User guidance 2026-06-30"
---

# tandem
