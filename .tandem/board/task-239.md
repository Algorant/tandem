---
id: task-239
type: task
title: "Tighten in-progress → logs vs validation transitions"
state: todo
priority: "high"
effort: "medium"
relatedFiles: ["protocol/plan/spec.md", "tandem/src/protocol", "AGENTS.md"]
createdAt: "2026-08-22T16:11:49Z"
updatedAt: "2026-08-22T16:13:54Z"
---
## Problem

`validation` has become the default parking state for delivered work. It should be an exception state. Most tasks should move `in-progress` → completed → logs without stopping.

Rough intent, not yet settled: `validation` is legitimate only when the orchestrator cannot verify the work itself, or when the work explicitly needs human judgment or human-only verification. That should be a small percentage of tasks.

## Observed causes

- `protocol/plan/spec.md` (~line 1142): Accord `delivered` and `accepted` both auto-move tasks into `validation`. Accepted work lands in the parking state and completion becomes a separate step agents skip.
- Pi skill `pi-tandem` states "use `validation` for delivered work awaiting acceptance", reading as the default destination for all delivered work.
- Pi skills `pi-tandem` and `pi-agency` use permissive framing ("may accept objective non-visual work") instead of an obligation, so agents choose the passive option.
- No protocol requirement to record why a task sits in `validation`, so there is no justification pressure and no way to audit the ratio.

## Status: unsettled

The fix is a design decision, not a known change. Nothing here is agreed yet, including whether `validation` stays a workflow state, whether a validation reason is required, and what evidence lets an orchestrator complete directly.

task-239-4 is a joint planning session with the human and blocks everything else. It must close with a `tandem decision` record. task-239-1, task-239-2, and task-239-3 execute that decision only.

Do not delegate this task or any subtask before task-239-4 closes.

## Scope note

Pi skills live outside this repository at `~/.pi/agent/skills/pi-tandem/SKILL.md` and `~/.pi/agent/skills/pi-agency/SKILL.md`. Skill edits are out-of-repo work tracked by this task.