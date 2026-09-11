---
id: task-28
type: task
title: "Allow warning-free Epic closure after all children are completed and archived"
priority: "low"
tags: ["protocol", "papercut", "workflow"]
accord:
  status: "accepted"
  acceptance: ["An Epic with at least one child and all descendant Tasks/Subtasks completed and archived can complete without the missing-delivered-Accord warning.", "Completion does not synthesize a delivered Accord or copy child evidence; archived parent outcome and ordinary explicitly delivered-parent acceptance follow existing rules.", "Active descendants still prevent completion; empty Epics, noncompleted child outcomes, missing/blocking references and ordinary Task cases preserve their applicable existing checks and warning behavior.", "Native regression tests assert the full matrix of all-completed children, delivered-but-active children, no children, canceled/failed children, ordinary Tasks and explicitly delivered parents.", "Normative documentation states the narrow child-based Epic completion exception and distinguishes archive outcomes from delivery status."]
  claimedAt: "2026-09-11T14:46:57Z"
  deliveredAt: "2026-09-11T15:00:04Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml --check"]
  constraints: ["Schedule after task-2 (overlapping app/tasks.rs, protocol/README.md and docs/cli/index.md); do not overlap Workers on those files.", "Protocol owns policy meaning; derive Epic roles from resolved documents, never ID shape. App composes existing hierarchy input and performs writes.", "No new Accord status, no derived persisted delivery, no lifecycle bypass, no adapter or TUI implementation changes.", "Submit plan/design before edits and stop for approval; clarify ambiguous hierarchy/outcome cases through worker_ask."]
  summary: "Applied the requested documentation-only correction and reran the declared validations. protocol/README.md no longer claims child-based closure 'performs no lifecycle transition' or 'never rewrites the Epic's own Accord status'; it now states that eligibility only suppresses the missing-delivery warning, that an otherwise-undelivered eligible Epic gets no synthetic Accord delivery/acceptance and no copied child evidence, and that normal archive/checkpoint behavior and ordinary delivered-parent acceptance still apply. docs/cli/index.md carried the same unqualified absolute claim, so its child-based Epic bullet was qualified the same way for consistency. No code changed: feature commit 4a6138f is unchanged in behavior and sits under docs commit fd45b56. Both required validations pass on fd45b56 and the working tree is clean. No pushes, merges, branch deletion, or worktree removal."
  evidence: ["Normative documentation states the narrow child-based Epic completion exception and distinguishes archive outcomes from delivery status.: protocol/README.md now describes the exception as warning eligibility only and explicitly retains normal archive/checkpoint behavior and delivered-parent acceptance; docs/cli/index.md documents the same qualified rule and the resolution.outcome vs delivery-status distinction.", "Avoid contradictory absolute wording about lifecycle and Accord status.: The phrases 'performs no lifecycle transition' and 'never rewrites the Epic's own Accord status' were removed from protocol/README.md, and the matching 'never changes the Epic's own Accord status' clause was removed from docs/cli/index.md.", "An Epic with at least one child and all descendants completed and archived completes without the missing-delivered-Accord warning.: Unchanged code commit 4a6138f; the 10/10 epic_completion_behavior suite still passes under `just dev-check` on fd45b56, including eligible_epic_closes_warning_free_without_fabricating_delivery.", "Completion does not synthesize delivery/acceptance or copy child evidence for an otherwise-undelivered eligible Epic.: Documentation now states this for the otherwise-undelivered case; the integration test still asserts the archived eligible Epic keeps status ready and carries no delivered/accepted status and no evidence.", "Active descendants, empty Epics, and noncompleted child outcomes preserve existing behavior.: Unchanged code; tests epic_with_delivered_but_active_child_is_rejected, empty_epic_retains_policy_warning_in_json_and_human, canceled_child_retains_policy_warning, failed_child_retains_policy_warning, legacy_completion_outcome_alone_does_not_qualify, ordinary_task_with_completed_subtask_retains_policy_warning, and epic_unresolved_blocker_still_blocks_closure all pass in the fd45b56 dev-check run."]
  filesChanged: ["protocol/README.md", "docs/cli/index.md", "tandem/src/protocol/diagnostic.rs", "tandem/src/app/support.rs", "tandem/src/app/tasks.rs", "tandem/src/cli/commands.rs", "tandem/tests/epic_completion_behavior.rs"]
  updatedAt: "2026-09-11T15:00:04Z"
createdAt: "2026-09-09T21:16:54Z"
updatedAt: "2026-09-11T15:00:04Z"
effort: "medium"
relatedFiles: ["tandem/src/protocol/diagnostic.rs", "tandem/src/protocol/hierarchy.rs", "tandem/src/app/support.rs", "tandem/src/app/tasks.rs", "tandem/src/cli/commands.rs", "tandem/tests/epic_completion_behavior.rs", "protocol/README.md", "docs/cli/index.md"]
assignee: "worker-task-28-797c36dc"
archivedAt: "2026-09-11T15:00:04Z"
resolution:
  outcome: "completed"
---
## Reproduced baseline
On0.13.1, an Epic with a delivered-but-active child correctly refuses completion for active descendants. After that child is completed/accepted and archived, completing the Epic succeeds but warns that the Epic's own accord.status=ready and complete normally follows delivery.

## Algorant-approved policy
Suppress only the missing-parent-delivery warning when a resolved Epic has a nonempty child hierarchy consisting entirely of completed archived Tasks/Subtasks. Child-based closure is sufficient; no fabricated delivered status or copied evidence is written. Ordinary Tasks retain their existing completion behavior. Active descendants still block closure. Empty Epics and hierarchies containing canceled/failed children keep the existing delivery-warning policy (an explicitly delivered parent may still follow normal warning-free completion).

Implement protocol-owned policy over resolved hierarchy/location/outcome inputs, with the app assembling those inputs. Preserve blockers, structural validation and every other completion check. Avoid invoking completion transitions to synthesize an Epic delivery.

## History
Reported from Pi Epic task-102 after all six children were accepted/archived. Prior title said delivered/accepted interchangeably; this clarified scope preserves the crucial distinction. This Task is now authorized for implementation after an agreed plan.