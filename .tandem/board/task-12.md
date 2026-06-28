---
id: task-12
type: task
title: "Scaffold pi-tandem extension area and adapter spec"
state: "in-progress"
priority: "high"
tags: ["pi-extension", "pi-tandem"]
createdAt: "2026-06-27T23:27:58Z"
updatedAt: "2026-06-28T00:35:13Z"
accord:
  status: "claimed"
  assignee: "herd:pi-tandem-extension"
  claimedAt: "2026-06-28T00:35:13Z"
  deliverables: ["docs:extensions/README.md:extension area overview", "docs:extensions/pi-tandem/plan/spec.md:pi-tandem adapter MVP scope"]
  validation:
    commands: ["rg -n 'rules_decisions|contract|handoff|done column|tandem binary' README.md plan AGENTS.md extensions || true"]
  constraints: ["Do not implement extension behavior in this task; define structure and scope only.", "Preserve no-drift docs by updating parent docs and AGENTS.md."]
  updatedAt: "2026-06-28T00:35:13Z"
---

## Description

Define the new `extensions/` project area and the initial `extensions/pi-tandem/` planning scaffold.

Context:
- Pi is the preferred agent surface for Tandem.
- The extension should follow the existing pi-web-tools convention: a lightweight adapter over an installed CLI/control surface, not a duplicate protocol implementation.
- `pi-tandem` should assume `tdm` is installed/available and should facilitate correct Tandem usage from Pi.

Acceptance direction:
- Add top-level `extensions/` docs (`README.md`, `plan/spec.md`, `plan/todo.md`) as a discrete repo area for agent/editor integrations.
- Add `extensions/pi-tandem/` planning docs for the Pi extension.
- Update parent docs and `AGENTS.md` to allow the third major child area without contradicting the current protocol/tandem-tui layout rules.
- Specify the adapter principle explicitly: Pi tools call `tdm`, Tandem behavior belongs in the CLI/protocol, and the extension owns schemas, prompts, diagnostics, rendering, and Pi ergonomics.
- Document local project-extension testing first, then later global canonical Pi config promotion.
