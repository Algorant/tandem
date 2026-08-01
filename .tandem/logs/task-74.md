---
id: task-74
type: task
title: "Add tandem update command for task metadata edits"
priority: "high"
relatedFiles: ["tandem/src/main.rs", "tandem/plan/spec.md", "protocol/plan/spec.md"]
tags: ["cli", "metadata", "priority", "agents"]
createdAt: "2026-06-30T18:03:40Z"
updatedAt: "2026-06-30T18:42:43Z"
subtasks:
  - id: task-74-1
    title: "Implement `tandem update <id>` for active task metadata edits: title, priority, assignee, dueDate, tags, blockers, references, relatedFiles"
    completed: false
  - id: task-74-2
    title: "Implement raw-source frontmatter patching that preserves unknown fields and updates updatedAt only on real changes"
    completed: false
  - id: task-74-3
    title: "Validate update behavior through CLI and JSON reads using existing/default field validation except priority"
    completed: false
  - id: task-74-4
    title: "Document the command in CLI/TUI specs as the supported alternative to manual .tandem edits"
    completed: false
  - id: task-74-5
    title: "Update canonical global Pi config pi-tandem extension to expose the new update action"
    completed: false
accord:
  status: "accepted"
  assignee: "herd:task-74-update-command"
  claimedAt: "2026-06-30T18:26:49Z"
  deliveredAt: "2026-06-30T18:33:26Z"
  deliverables: ["CLI command `tandem update <id>` for active task metadata: scalar title/priority/assignee/dueDate replacements, append/dedup tags/blockers/references/relatedFiles, no state/parent/log updates.", "Priority validation for critical/high/medium/low; strict blocker resolution; soft unresolved-reference warnings; no-op handling without updatedAt/event changes; task.updated events on real changes.", "Show JSON now exposes blockers/references/relatedFiles for validation/read coverage.", "Repo-local and canonical global pi-tandem adapters support tandem_task action=update as thin CLI argument builders."]
  validation:
    commands: ["cargo test --manifest-path tandem/Cargo.toml: 96 passed", "bun extensions/pi-tandem/tests/smoke.ts: passed", "Manual temp-workspace smoke verified update old/new output, duplicate list-entry no-op, and show --json metadata."]
  summary: "Accepted non-visual CLI/tooling work after automated and manual smoke validation passed."
  evidence: ["Tandem repo commit 2155cce Add tandem update task metadata command", "Dotfiles commit 89960e76 Expose tandem update in pi-tandem"]
  filesChanged: ["tandem/src/main.rs", "tandem/README.md", "tandem/plan/spec.md", "protocol/plan/spec.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/tests/smoke.ts", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem/index.ts"]
  reviewer: "orchestrator"
  updatedAt: "2026-06-30T18:42:38Z"
completedAt: "2026-06-30T18:42:43Z"
completion:
  summary: "Implemented and verified `tandem update <id>` for active task metadata edits, including scalar replacements, append/dedupe list metadata, priority validation, old/new mutation output, docs/spec updates, repo-local pi-tandem smoke coverage, and canonical Pi config pi-tandem update."
  validation: "Accepted after cargo test passed 96 tests, pi-tandem smoke passed, and manual temp-workspace smoke verified update output, duplicate no-op behavior, and show --json metadata."
  reviewer: "orchestrator"
---

## Description

Add a first-class CLI mutation path for updating active task metadata without requiring a state transition or manual frontmatter edits. The core need is setting/changing priority after task creation, but the command should be designed around workflow-orthogonal metadata so agents and humans can update fields like priority, tags, assignee, dueDate, and related metadata safely through the CLI. Avoid overloading `tandem move`; keep state transitions separate from metadata updates unless a deliberate spec decision says otherwise.

## Delegation notes

This is deferred for later implementation, not ready for validation now.

Decided command shape:

- `tandem update <id>` only updates active board task documents.
- Logs remain immutable by convention; do not support updating logs.
- Do not support `--state`; state remains exclusively `tandem move` because it has workflow side effects.
- Do not support updating `parentId`.
- Scalar replacements: `--title`, `--priority`, `--assignee`, `--due-date`.
- Priority is constrained to `critical`, `high`, `medium`, or `low`.
- Append/deduplicate list fields by default: `--tag`, `--blocker`, `--reference`, `--related-file`.
- No clear/remove flags for this task.
- Repeated existing tags/list entries are no-ops, not errors.
- Keep existing/default validation behavior for fields other than priority. In particular, blockers should follow current strict blocker validation, references should follow current soft warning behavior, and related files should remain path metadata.
- Mutation output should show old and new values for changed fields, plus the path. If nothing changed, report a no-op clearly.
- Update canonical global Pi config `pi-tandem` after CLI support lands; do not treat the repo-local extension guidance as canonical for the user's active Pi setup.
