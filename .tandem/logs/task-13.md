---
id: task-13
type: task
title: "Implement CLI-backed pi-tandem extension MVP"
priority: "high"
tags: ["pi-extension", "pi-tandem"]
createdAt: "2026-06-27T23:30:04Z"
updatedAt: "2026-06-28T01:34:29Z"
accord:
  status: "accepted"
  assignee: "herd:pi-tandem-extension"
  claimedAt: "2026-06-28T00:35:13Z"
  deliveredAt: "2026-06-28T00:52:05Z"
  deliverables: ["code:extensions/pi-tandem/index.ts:Pi extension wrapper over tdm CLI", "docs:extensions/pi-tandem/README.md:usage and tool mapping", "test:extensions/pi-tandem/tests/smoke.ts:CLI-backed smoke test"]
  validation:
    commands: ["bun --check extensions/pi-tandem/index.ts", "bun extensions/pi-tandem/tests/smoke.ts"]
  constraints: ["Do not duplicate Tandem protocol mutation logic; call tdm for behavior.", "Do not edit global Pi config in this task."]
  summary: "Implemented the CLI-backed pi-tandem MVP with tdm_status/task/accord/log/rules/decision/search tools, /tandem diagnostics, prompt guidance, and smoke coverage."
  evidence: ["Commit 33dfd81 on branch tandem-task12-13-pi-tandem contains the MVP implementation and docs.", "Validation passed: bun --check extensions/pi-tandem/index.ts; bun extensions/pi-tandem/tests/smoke.ts; tdm read-path smoke; cargo test; git diff --check."]
  filesChanged: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/plan/todo.md"]
  reviewer: "pi"
  updatedAt: "2026-06-28T01:34:29Z"
completedAt: "2026-06-28T01:34:29Z"
completion:
  summary: "Implemented the CLI-backed pi-tandem MVP, including the tdm_task summary schema fix; integrated in ce6303d."
  validation: "bun --check extensions/pi-tandem/index.ts; bun extensions/pi-tandem/tests/smoke.ts; cargo test"
---

## Description

Implement the first `pi-tandem` extension as a lightweight Pi adapter over the installed `tdm` CLI.

Context:
- Model after `pi-web-tools`: `LLM -> Pi tool -> pi-tandem -> tdm CLI -> .tandem`.
- Do not duplicate Tandem protocol parsing/mutation logic in TypeScript except for trivial result formatting/diagnostics.
- Use `execFile` with argument arrays and no shell interpolation.

Initial tool/command direction:
- Register tools such as `tdm_status`, `tdm_task`, `tdm_accord`, `tdm_log`, `tdm_rules`, `tdm_decision`, and `tdm_search` as thin schemas over `tdm` subcommands.
- Prefer `tdm --json` read paths where available; preserve useful human-readable output for write commands until CLI JSON write output exists.
- Add `/tandem` help/status command.
- Add prompt snippets/guidelines so Pi agents prefer `tdm_*` tools when `.tandem/tandem.md` exists or when bootstrapping durable coordination is requested.
- Provide diagnostics for missing `tdm`, missing `.tandem`, unsupported CLI version/flags, and command failures.
