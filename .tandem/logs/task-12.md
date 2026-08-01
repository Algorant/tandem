---
id: task-12
type: task
title: "Scaffold pi-tandem extension area and adapter spec"
priority: "high"
tags: ["pi-extension", "pi-tandem"]
createdAt: "2026-06-27T23:27:58Z"
updatedAt: "2026-06-28T01:34:29Z"
accord:
  status: "accepted"
  assignee: "herd:pi-tandem-extension"
  claimedAt: "2026-06-28T00:35:13Z"
  deliveredAt: "2026-06-28T00:51:58Z"
  deliverables: ["docs:extensions/README.md:extension area overview", "docs:extensions/pi-tandem/plan/spec.md:pi-tandem adapter MVP scope"]
  validation:
    commands: ["rg -n 'rules_decisions|contract|handoff|done column|tandem binary' README.md plan AGENTS.md extensions || true"]
  constraints: ["Do not implement extension behavior in this task; define structure and scope only.", "Preserve no-drift docs by updating parent docs and AGENTS.md."]
  summary: "Scaffolded the extensions area and pi-tandem adapter planning docs; updated parent docs and AGENTS.md for the third major area and adapter boundaries."
  evidence: ["Commit 33dfd81 on branch tandem-task12-13-pi-tandem adds the extension area, pi-tandem docs/spec/todo, and parent doc synchronization.", "Validation: rg stale-term scan, git diff --check, cargo test, bun --check extensions/pi-tandem/index.ts, bun extensions/pi-tandem/tests/smoke.ts."]
  filesChanged: ["AGENTS.md", "README.md", "plan/spec.md", "plan/todo.md", "extensions/README.md", "extensions/plan/spec.md", "extensions/plan/todo.md", "extensions/pi-tandem/README.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/plan/todo.md"]
  reviewer: "pi"
  updatedAt: "2026-06-28T01:34:29Z"
completedAt: "2026-06-28T01:34:29Z"
completion:
  summary: "Scaffolded the extensions/pi-tandem area and adapter guidance; integrated in ce6303d."
  validation: "bun --check extensions/pi-tandem/index.ts; bun extensions/pi-tandem/tests/smoke.ts; cargo test"
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
