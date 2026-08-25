---
id: task-239
type: task
title: "Tighten in-progress → logs vs validation transitions"
priority: "high"
effort: "medium"
relatedFiles: ["protocol/plan/spec.md", "tandem/src/protocol", "AGENTS.md"]
createdAt: "2026-08-22T16:11:49Z"
updatedAt: "2026-08-25T01:37:14Z"
references: ["papercut-4", "papercut-5"]
completedAt: "2026-08-25T01:37:14Z"
completion:
  summary: "Closed via decision-11 and its amendments A1-A4. validation is no longer the default parking state for delivered work: no accord action moves workflow state except claim and rework-from-validation, review.status pending is the only entrance to validation, completing with a pending review is hard error E067, and a missing review is normal and silent. Shipped as task-239-1 (protocol spec), task-239-2 (Rust implementation, 280 unit + 12 integration tests), and task-241 (Board surfaces delivered-but-untriaged work). task-239-4 produced the decision. task-239-3 was canceled and recreated as task-254 in the ~/.dotfiles/pi/.pi workspace, since Pi skill files live in a separate Stow-managed repository."
---
## Problem

`validation` has become the default parking state for delivered work. It should be an exception state. Most tasks should move `in-progress` → completed → logs without stopping.

Rough intent, not yet settled: `validation` is legitimate only when the orchestrator cannot verify the work itself, or when the work explicitly needs human judgment or human-only verification. That should be a small percentage of tasks.

## Observed causes

- `protocol/plan/spec.md` (~line 1142): Accord `delivered` and `accepted` both auto-move tasks into `validation`. Accepted work lands in the parking state and completion becomes a separate step agents skip.
- Pi skill `pi-tandem` states "use `validation` for delivered work awaiting acceptance", reading as the default destination for all delivered work.
- Pi skills `pi-tandem` and `pi-agency` use permissive framing ("may accept objective non-visual work") instead of an obligation, so agents choose the passive option.
- No protocol requirement to record why a task sits in `validation`, so there is no justification pressure and no way to audit the ratio.

## Evidence: 2026-08-22 orchestration session (task-237, task-238)

Both tasks were objective, non-visual, with explicit acceptance criteria the orchestrator verified by rerunning tests and inspecting compiler cfg output. Both qualified for direct orchestrator acceptance under existing guidance. Neither was accepted until the human intervened.

Three distinct failures, in order:

1. **Parked despite qualifying for acceptance.** After verifying task-238 and merging it to `main`, the orchestrator reported "task stays in validation, acceptance is yours." Confirms the permissive-framing cause above: "may accept" was read as "acceptance is optional and therefore not mine," and parking was treated as the safe default.

2. **Reported a state it never read.** The task was not in `validation`. It was `state: in-progress`, `accord: claimed`, assigned to a Worker that no longer existed. The orchestrator described workflow state from assumption. Distinct from the framing problem and more serious: a misapplied policy is recoverable, a fabricated status is not.

3. **Integration and board state diverged silently.** Code merged to `main` while the board still showed the task claimed and in progress. Neither `worker_integrate` nor `worker_cleanup` reads or reports task state, so nothing surfaced the gap. Recorded as `papercut-4`. The mechanism, that Worker-side `tandem accord` calls write to the Worker's own worktree uncommitted and are destroyed at cleanup, is recorded as `papercut-5`.

Resolution applied: the orchestrator recorded `deliver` → `accept` → `complete` for both tasks from the main checkout. Both completions warned `review.status=missing`, a field that stayed untouched throughout and that task-239-4 may want as the audit hook for justifying time in validation.

Bearing on the open questions: the failure was not that the rule was unknown. The rule was read, and its permissive half was discarded in favor of its restrictive half. A reworded obligation only helps if it is unambiguous about who decides and when; it does not address failure 2, which needs a read-before-assert requirement rather than a policy change.

## Status: unsettled

The fix is a design decision, not a known change. Nothing here is agreed yet, including whether `validation` stays a workflow state, whether a validation reason is required, and what evidence lets an orchestrator complete directly.

task-239-4 is a joint planning session with the human and blocks everything else. It must close with a `tandem decision` record. task-239-1, task-239-2, and task-239-3 execute that decision only.

Do not delegate this task or any subtask before task-239-4 closes.

## Scope note

Pi skills live outside this repository at `~/.pi/agent/skills/pi-tandem/SKILL.md` and `~/.pi/agent/skills/pi-agency/SKILL.md`. Skill edits are out-of-repo work tracked by this task.