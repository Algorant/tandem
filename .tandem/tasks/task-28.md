---
id: task-28
type: task
title: "Allow warning-free Epic closure after all children are completed and archived"
state: "in-progress"
priority: "low"
tags: ["protocol", "papercut", "workflow"]
accord:
  status: "claimed"
  acceptance: ["An Epic with at least one child and all descendant Tasks/Subtasks completed and archived can complete without the missing-delivered-Accord warning.", "Completion does not synthesize a delivered Accord or copy child evidence; archived parent outcome and ordinary explicitly delivered-parent acceptance follow existing rules.", "Active descendants still prevent completion; empty Epics, noncompleted child outcomes, missing/blocking references and ordinary Task cases preserve their applicable existing checks and warning behavior.", "Native regression tests assert the full matrix of all-completed children, delivered-but-active children, no children, canceled/failed children, ordinary Tasks and explicitly delivered parents.", "Normative documentation states the narrow child-based Epic completion exception and distinguishes archive outcomes from delivery status."]
  claimedAt: "2026-09-11T14:46:57Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml --check"]
  constraints: ["Schedule after task-2 (overlapping app/tasks.rs, protocol/README.md and docs/cli/index.md); do not overlap Workers on those files.", "Protocol owns policy meaning; derive Epic roles from resolved documents, never ID shape. App composes existing hierarchy input and performs writes.", "No new Accord status, no derived persisted delivery, no lifecycle bypass, no adapter or TUI implementation changes.", "Submit plan/design before edits and stop for approval; clarify ambiguous hierarchy/outcome cases through worker_ask."]
  updatedAt: "2026-09-11T14:46:57Z"
createdAt: "2026-09-09T21:16:54Z"
updatedAt: "2026-09-11T14:46:57Z"
effort: "medium"
relatedFiles: ["tandem/src/protocol/diagnostic.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/app/tasks.rs", "tandem/tests/epic_completion_behavior.rs", "protocol/README.md", "docs/cli/index.md"]
assignee: "worker-task-28-797c36dc"
---
## Reproduced baseline
On0.13.1, an Epic with a delivered-but-active child correctly refuses completion for active descendants. After that child is completed/accepted and archived, completing the Epic succeeds but warns that the Epic's own accord.status=ready and complete normally follows delivery.

## Algorant-approved policy
Suppress only the missing-parent-delivery warning when a resolved Epic has a nonempty child hierarchy consisting entirely of completed archived Tasks/Subtasks. Child-based closure is sufficient; no fabricated delivered status or copied evidence is written. Ordinary Tasks retain their existing completion behavior. Active descendants still block closure. Empty Epics and hierarchies containing canceled/failed children keep the existing delivery-warning policy (an explicitly delivered parent may still follow normal warning-free completion).

Implement protocol-owned policy over resolved hierarchy/location/outcome inputs, with the app assembling those inputs. Preserve blockers, structural validation and every other completion check. Avoid invoking completion transitions to synthesize an Epic delivery.

## History
Reported from Pi Epic task-102 after all six children were accepted/archived. Prior title said delivered/accepted interchangeably; this clarified scope preserves the crucial distinction. This Task is now authorized for implementation after an agreed plan.