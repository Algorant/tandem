---
id: task-53
uid: 26e2bc8c-edb8-46d2-98d8-00e628cd44a0
type: task
title: "Rules end to end: starter set, categories, toggles and agent visibility — mockup first"
state: "in-progress"
priority: "medium"
tags: ["rules", "ui"]
accord:
  status: "blocked"
  acceptance: ["Phase one: an Astra or Fable Worker publishes an explicitly proposed Sideshow end-to-end rules mockup using a toy project, showing the seeded set, always/never/prefer/context categories, on/off behavior and what agents see, with Excalidraw-method diagram assets; Algorant reviews it before any implementation.", "After separate approval: normative on/off representation/defaults, CLI/JSON and context refresh behavior are decided before code; disabled rules remain discoverable and retain IDs, category, text/source and supported metadata.", "After separate approval: init seeds agreed categorized rules, reports them, and idempotently adds a short rules pointer to existing AGENTS.md/CLAUDE.md without creating absent files.", "After separate approval: TUI keyboard/mouse state and agent-facing enabled-rule selection are unambiguous, reload-safe and not color-only; any external integration change is separately authorized.", "After separate approval: temporary native checks cover seeded/default/present-absent/idempotent pointer cases, toggling/readback and context refresh; published guidance matches the verified contract."]
  claimedAt: "2026-10-17T02:47:10Z"
  validation: ["$ git diff --check"]
  constraints: ["Phase one is mockup/design only, on a fresh Astra (openai-codex/gpt-6-astra) or Fable Worker; explicit plan before artifacts. The named model is Algorant's exception to Luna for this design pass.", "No implementation, protocol/code changes or actual init/toggle behavior until Algorant reviews the prototype and approves a subsequent plan.", "Keep entire root Task with its Worker; prototype milestone is internal, never separately delegated. Mockup milestone does not complete the whole Task.", "Serialize Excalidraw/Obsidian access after task-81 visual production; existing vault/EA PNG method only, editable source retained, no exporter or hand-written scenes.", "No external adapter implementation under this Task; needed integration work requires a separately authorized handoff. Temporary checks only."]
  note: "Phase-one53-1 delivered: http://desktop.tail1cefc.ts.net:8228/session/XLDMECmdbMI. Parentactualpost/PNG/source/hash/cleanGit/HTTPchecks passed. Awaitprototypejudgment/openchoices and separatelyapprovedimplementationplan. No rootcompletion/worker_finish/merge/codeauthorized; retainedAstraWorker clean. EA slotreleased; no browsers/previews."
  updatedAt: "2026-10-17T03:58:15Z"
createdAt: "2026-10-01T18:27:09Z"
updatedAt: "2026-10-17T03:58:15Z"
blockers: ["task-81"]
effort: "large"
references: ["task-54", "task-20", "task-81"]
relatedFiles: ["protocol/README.md", "tandem/src/app/project.rs", "tandem/src/app/rules.rs", "tandem/src/project/rules.rs", "tandem/src/tui/rules.rs"]
assignee: "worker-task-53-b81689dc"
---
## Consolidated scope and gate

Algorant via Herdra combined tasks 53, 54 and 20 into this one root Task. Tasks 54/20 are canceled as absorbed, not implemented. They are not reparented because immutable global IDs cannot become canonical Subtask IDs. Keep categorized starter rules, init agent-file pointer, on/off state and agent context visibility together as one end-to-end outcome.

## Phase one: proposal only

A fresh Astra (openai-codex/gpt-6-astra) or Fable Worker first sends its plan, then imagines the complete experience in Sideshow: a neutral toy project, a short plausible default set, categorized always/never/prefer/context, visible on/off toggles without deletion, and the enabled rules that a human/agent reads. Include Excalidraw-method PNG diagrams as assets with editable vault source, not another diagram tool. All proposed controls/CLI/JSON are labelled unimplemented. Show context freshness explicitly—what happens on a rules read/refresh versus any already-running agent context—and surface open representation/default/safety choices rather than deciding them through visuals. No private project paths/hosts/names.

Publish to a separate rules-proposal Sideshow session and report its verified MagicDNS link. The Worker stops for Algorant's review; deliver the internal prototype milestone, not the entire Task as finished. No code/protocol/actual init/toggle implementation is authorized yet.

Task-81 owns current docs diagram production first; serialize the shared Obsidian EA/global result slot to avoid two drawing Workers overwriting each other's state. Once that lane is free, prototype work may run alongside docs prose. No actual adapter edits.

## Future scope after prototype review and a new approved plan

Preserve original init requirement: prebuilt rules helping coordination (read rules, use Tandem CLI/tools, don't mutate board files directly, archive records read-only), a reviewed category for every seeded rule, stable IDs, explicit init output, and an idempotent one-line pointer in existing AGENTS.md and/or CLAUDE.md only. On/off design must specify its durable representation and existing-rule defaults, preserve disabled records, expose CLI/JSON and accessible TUI behavior, and define how context consumers obtain current enabled rules. Normative semantics first, shared app operations next; do not duplicate protocol parsing or modify external framework configuration without an explicit authorized adapter handoff.

This is the combined work from tasks 53/54/20, not permission to start that implementation now.