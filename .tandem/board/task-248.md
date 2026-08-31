---
id: task-248
type: task
title: "Align the project-local pi-tandem extension with protocol 0.3.0"
state: "in-progress"
priority: "high"
effort: "large"
blockers: ["task-247"]
references: ["task-246", "decision-12"]
relatedFiles: ["extensions/pi-tandem/", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/tests/", "plan/cli-protocol-cutover.md", "tandem/plan/clap-migration-research.md"]
tags: ["pi-tandem", "cli", "adapter"]
createdAt: "2026-08-30T17:48:37Z"
updatedAt: "2026-08-31T20:18:40Z"
accord:
  status: "claimed"
  assignee: "pi"
  claimedAt: "2026-08-31T20:18:40Z"
  updatedAt: "2026-08-31T20:18:40Z"
---

## Description

## Goal

After `task-247` lands, audit and overhaul the project-local `extensions/pi-tandem/` adapter against the installed protocol 0.3.0 CLI. This is an explicit adapter Task; do not implement it inside the core cutover.

## Direction

Do not mechanically mirror CLI command families or preserve the existing tool inventory. Evaluate each agent intent against the finished CLI. Retain, refactor, combine, create, or remove tools based on model-call utility. Preserve a convenient way for agents to add Papercut-tagged low-priority Tasks with mandatory Accord acceptance.

## Constraints

- Core Tandem is authoritative for protocol, IDs, hierarchy, storage, Accord, lifecycle, Rules, and errors.
- Use `execFile`/argument arrays and global JSON envelopes only.
- Never parse or mutate Tandem Markdown, storage directories, events, or frontmatter in TypeScript.
- Do not add compatibility shims for the previous CLI or protocol.
- Final tool count, names, and schemas are decided during this Task against real CLI behavior.

## Required audit

Inspect every tool schema, argument builder, renderer, command, prompt guideline, README section, and test. Remove old assumptions about `move`, per-command JSON, Review status/actions, `accord accept`, separate Log/Decision/Papercut CLI families, additive list updates, `--description`, old Rule arguments, and protocol 0.2 storage.

## Acceptance

- No emitted argv references a removed command or flag.
- Every retained/new tool consumes canonical JSON success/error envelopes and process status.
- No adapter code owns protocol or persistence behavior.
- Papercut-tagged Task capture remains convenient.
- Tool inventory is intentionally justified by agent utility.
- Stale tools and guidance are removed rather than shimmed.
- Focused closed schemas pass unit tests and real-command smoke tests against protocol 0.3.0.
