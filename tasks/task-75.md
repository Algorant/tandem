---
id: task-75
uid: 197f29a1-27bb-4e18-b5ab-c5ed42e996a9
type: task
title: "Deferred: reimagine web orchestration across many projects"
state: todo
priority: "medium"
effort: "large"
references: ["task-5", "task-70", "task-73", "task-74"]
relatedFiles: ["tandem/src/web.rs", "tandem/src/web/app.js", "tandem/src/web/api.js", "tandem/src/app/queries.rs"]
tags: ["ui", "orchestrator"]
accord:
  status: "blocked"
  acceptance: ["When authorized, a reviewed design describes multi-project orchestration/multitasking, possible kanban and real-time workflow monitoring with Ceph/Worker/open-item/handoff/Now concepts; proposals are not treated as settled UI.", "The design weighs plugin/adapter neutrality versus a separable Pi + Herdr-specific web view, records pros/cons and obtains a decision before implementation.", "The approved plan identifies live agent/session/workflow data sources, separation from board-recorded status, required plumbing, ownership and implementation boundaries beyond today's read-only web."]
  validation: ["$ just dev-check"]
  constraints: ["Explicitly deferred until Algorant starts it; board edit only, no implementation/research/mockups/plugin dispatch.", "Tool-neutrality options remain open; do not create a Decision or encode either option as chosen."]
  note: "Web orchestration explicitly deferred by Algorant; no work without separate approval."
  updatedAt: "2026-10-13T13:27:41Z"
createdAt: "2026-10-12T15:51:25Z"
updatedAt: "2026-10-13T13:27:41Z"
---
## Direction from Algorant via Herdra

A complete rethink of the web UI around orchestration and multitasking across many projects at once. Explore a possible kanban-style view and high-level monitoring of agentic workflows in real time; no layout is settled.

Carry the flavour of Herdra's current workflow—Cephs, Workers, open items, handoffs and one Now item—into a more logical, codified model so the pieces work together. This likely needs live agent/session/workflow-state plumbing beyond today's single-workspace read-only web; it is a larger design/implementation effort, not merely a selector or summary page.

## Open tool-neutrality choice: decide when the Task starts

No architecture decision is made now. Weigh and document both options:
(a) Plugin/adapter system, with Pi + Herdr as the first/reference integration. Pros: Tandem core stays neutral, other tools can supply their own state, clear integration contract. Cons: additional adapter/API design and lifecycle/versioning work, configuration/maintenance overhead before the first complete view.
(b) A web orchestration view explicitly built for Pi + Herdr, with that code kept separable and the CLI, TUI and board remaining tool-neutral. Pros: smaller direct first slice, concrete existing live-state source, less speculative adapter machinery. Cons: narrower audience, setup coupling, future integrations may need refactoring and the separation must be enforced.

Agent/session/workflow signals would come from the chosen integration boundary, never be inferred as live status merely from board assignees/Accord state. Consider how this fits today's arrangement, where Herdr project discovery lives outside Tandem.

## Deferral and absorbed scope

Task-5's old full-product web review is superseded by this reimagining. Existing roster/open-by-path/BoardSummary interfaces may be reused where helpful, but don't constrain the new product to the old /all/configured-selector design. No permanent workspaces.toml requirement.

Still deferred until Algorant explicitly starts it. No research dispatch, mockups, code, plugin or live-state service now. Start with product/plumbing exploration and pros/cons; obtain approval before implementation.