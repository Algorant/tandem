---
id: task-15
type: task
title: "Make durable Task and milestone workflow usable through the TUI"
state: "in-progress"
priority: "medium"
references: ["task-6", "task-13"]
relatedFiles: ["tandem/src/tui", "plan/task-15-pi-handoff.md"]
tags: ["tui", "accord"]
accord:
  status: "claimed"
  acceptance: ["Version-matched audit identifies actual missing durable actions/context and reuses the native app layer, with task-6 relationship reconciled explicitly.", "TUI shows current acceptance, blockers, milestone progress and final delivery evidence without Pi/session scraping or duplicated inline checklist state.", "Task claim/deliver/block/resume and ordinary completion are usable via a minimal valid-action interface; exceptional human review remains distinct, and invalid transitions fail clearly.", "Action/render tests exercise state changes and resulting native records; no agent-launch controls or broad layout redesign are included."]
  claimedAt: "2026-09-05T22:11:29Z"
  validation: ["Run cargo test --manifest-path tandem/Cargo.toml (version-matched baseline 3d179b0: 241 unit plus 6 CLI tests passing), targeted TUI tests, release build, and git diff --check. Do not reformat unrelated Rust modules to remedy baseline formatting.", "Drive actual keyboard/mouse/modal action paths for claim, deliver with summary/evidence, block with note, resume and normal completion. Re-read resulting native records and assert exact Accord/state/assignee/evidence/location values; merely calling a helper or checking a rendered label is insufficient.", "Exercise missing required inputs, invalid native transitions, stale selected records, prompt cancellation and explicit error presentation. Use native validity predicates; do not duplicate workflow rules in TUI tables.", "Render long acceptance, blockers, milestone statuses and final evidence; test scrolling/selection and archived outcome visibility without truncating required context or replacing it with duplicate inline checklists.", "Build cargo build --manifest-path tandem/Cargo.toml --release. Prepare a reproducible disposable preview workspace via the built native CLI, not hand-authored .tandem records. Report its absolute path and launch command to the orchestrator early so the canonical just dev preview slot can point at this worktree. Inspect actual release TUI output including ANSI using the available Herdr presentation; if pane controls are unavailable, supply a ready fixture/build and request that inspection from the orchestrator. Snapshots do not prove flicker/resize latency; state any unresolved temporal/taste criteria explicitly.", "Handoff: clean source commit, changed paths, criterion-specific assertions and actual observations, two-approach/task-6 reconciliation, exact preview command/path, native API requests if any, and Pi consumer/version availability in plan/task-15-pi-handoff.md. No ready claim with acknowledged acceptance gaps."]
  constraints: ["Exclusive production/test ownership: tandem/src/tui/** only. Dedicated handoff/research notes may be written at plan/task-15-pi-handoff.md; no broad layout redesign, agent-launch controls, separate workflow state machine or inline milestone store.", "Use existing version-matched 0.12.3 native app/protocol APIs. Task-13 exclusively owns native read/protocol changes concurrently. If an existing capability is insufficient, report the exact signature/input/output needed to the orchestrator before implementing around it. Do not edit app, protocol, project, CLI, manifests or adapters.", "Task-6 is still an unclaimed research proposal, not completed findings. Explicitly reconcile it in the handoff: compare two lightweight interaction approaches, recommend the smallest valid-action interface, and implement only task-15's durable parity scope. Do not claim task-6 was completed or change its lifecycle.", "Use current protocol/README.md and accepted decision-8 for protocol 0.3.0 behavior; root AGENTS.md has superseded pre-cutover specifics. Preserve architectural ownership. Routine complete archives delivered work without requiring exceptional Validation.", "Source delivery is not installation/release. No push, merge, cleanup, or modification of real coordination records; use native CLI/app-created disposable workspaces for action testing."]
  updatedAt: "2026-09-05T22:11:29Z"
createdAt: "2026-09-05T21:04:08Z"
updatedAt: "2026-09-05T22:26:22Z"
assignee: "worker-task-15-a66158c3"
blockers: ["task-13"]
---

## Description

Implementation proposal from Pi task-59. User requires native CLI/TUI operation without Pi, with milestones/blockers/evidence as the normal view. Existing task-6 researches contextual lifecycle actions; review/reconcile its useful findings, do not duplicate research or assume its old conventions are mandatory. No production changes are authorized merely by task creation.

Verify actual version-matched TUI gaps first. Expose complete acceptance, dependencies, existing milestone status and final evidence; offer valid durable lifecycle actions through one simple interaction invoking existing native app operations. Routine completion need not route through human Validation. Do not add Worker runtime controls, another state machine, an alternative checklist store or a broad visual redesign. User has separately chosen native Herdr/Worktrunk for runtime/Git controls. Core read/app changes belong to task-13; request exact missing API rather than independently inventing it.

Validation must drive actual actions and inspect resulting records; static appearance alone is insufficient. Follow repository guidance to verify renderable TUI facts before requesting taste/temporal human review.
