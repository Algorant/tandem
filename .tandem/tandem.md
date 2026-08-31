---
protocolVersion: 0.3.0
type: workspace
title: "Tandem (protocol 0.3.0)"
states:
  - id: todo
    title: To Do
  - id: in-progress
    title: In Progress
  - id: validation
    title: Validation
rules:
  always:
    - id: 1
      rule: "When cutting a Tandem release, create and push both the annotated git tag and the GitHub Release object for that tag; do not treat a pushed tag alone as a complete release unless explicitly asked for tag-only."
    - id: 2
      rule: "During core Tandem work, specify needed cross-framework behavior in Tandem-owned protocol or guidance documents and create an explicit implementation handoff when an adapter change is needed."
    - id: 3
      rule: "Before orchestrating parallel or potentially overlapping delegated work, assess whether each worker needs an isolated branch or worktree."
    - id: 4
      rule: "Request human review only when you cannot verify the acceptance criteria yourself. Exhaust verification first: run the code, read the output, spawn a Herdr pane and read the rendered result including ANSI color. Static rendering, layout, counts, and colors are verifiable this way and are not automatically human work; temporal behavior such as flicker, resize tearing, and redraw latency is not verifiable from a snapshot. When you do escalate, request review and state what you ran and what specifically stayed unresolved, because \"visual work\" is not a reason. When you are unsure whether something needs review, ask the orchestrator or user directly with ask_me instead of parking the task."
  never:
    - id: 1
      rule: "Do not mark a newly created task as claimed, delivered, validation, accepted, or completed unless the user explicitly asked to start or finish the work; automated tests or smoke checks are evidence only, not permission to advance lifecycle state."
    - id: 2
      rule: "Do not modify `extensions/pi-tandem/`, external Pi configuration, or any other agent/framework adapter implementation as part of core Tandem work unless an explicit adapter task authorizes it."
    - id: 3
      rule: "Do not treat a pushed Git tag alone as a complete Tandem release unless the caller explicitly requests tag-only."
    - id: 4
      rule: "Never commit checkout-local identity, caches, credentials, or other runtime state."
    - id: 5
      rule: "Do not treat an AUR publication or update failure as blocking a Tandem release after the tagged GitHub Release, curated notes, required assets, checksums, and primary installer are published and verified. Record the AUR outcome as a downstream packaging issue and complete the release lifecycle."
  prefer:
    - id: 1
      rule: "Use one primary area tag first: `protocol`, `tui`, `pi-tandem`, `docs`, `config`, `rules`, or `ui`."
    - id: 2
      rule: "Add only a few capability/workflow tags when they aid delegation, such as `accord`, `review`, `logs`, `editor`, `relationships`, `delegation`, `taxonomy`, `smoke`, `validation`, or concrete TUI facets like `theme`, `keyboard`, `mouse`, and `markdown`."
    - id: 3
      rule: "When prototyping documentation site features, themes, layouts, or diagrams, create quick Sideshow mockups first wherever practical; use those previews to narrow direction before committing implementation or durable design decisions."
    - id: 4
      rule: "Prefer separate worktrees for likely file overlap, visual/design experiments, release automation, or independently committed work; prefer a shared tree only for read-only or explicitly coordinated work."
    - id: 5
      rule: "Commit durable `.tandem` changes regularly. Do not leave important board, decision, rule, or completed-work changes only in a local working tree for an extended period. Use judgment about commit boundaries: include Tandem changes in a related project commit, group related changes, or create a standalone coordination commit. Do not require one commit for every Tandem command."
    - id: 6
      rule: "Prefer Bun as the package manager and script runner for JavaScript/TypeScript automation, including docs-site recipes and CI. Use another tool when a concrete incompatibility makes Bun impractical."
  context:
    - id: 1
      rule: "For delegated non-visual, non-manual work with passing automated validation and no blockers, the orchestrator is authorized to accept and complete the task without additional human validation."
---

# Tandem (protocol 0.3.0)
