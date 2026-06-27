---
id: task-13
type: task
title: "Implement CLI-backed pi-tandem extension MVP"
state: todo
priority: "high"
tags: ["pi-extension", "pi-tandem"]
createdAt: "2026-06-27T23:30:04Z"
updatedAt: "2026-06-27T23:30:04Z"
accord:
  status: "ready"
  deliverables: ["code:extensions/pi-tandem/index.ts:Pi extension wrapper over tdm CLI", "docs:extensions/pi-tandem/README.md:usage and tool mapping", "test:extensions/pi-tandem/tests/smoke.ts:CLI-backed smoke test"]
  validation:
    commands: ["bun --check extensions/pi-tandem/index.ts", "bun extensions/pi-tandem/tests/smoke.ts"]
  constraints: ["Do not duplicate Tandem protocol mutation logic; call tdm for behavior.", "Do not edit global Pi config in this task."]
  updatedAt: "2026-06-27T23:30:04Z"
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
