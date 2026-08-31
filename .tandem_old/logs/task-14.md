---
id: task-14
type: task
title: "Project-local install and smoke pi-tandem"
priority: "medium"
tags: ["pi-extension", "pi-tandem"]
createdAt: "2026-06-27T23:30:04Z"
updatedAt: "2026-06-28T02:04:09Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-06-28T01:43:16Z"
  deliveredAt: "2026-06-28T02:04:09Z"
  deliverables: ["docs:extensions/pi-tandem/README.md:project-local install and smoke instructions", "test:extensions/pi-tandem/tests/project-smoke.md:manual smoke evidence"]
  validation:
    commands: ["bun extensions/pi-tandem/tests/smoke.ts", "bun extensions/pi-tandem/tests/pi-runtime-smoke.ts"]
  constraints: ["Do not commit private Pi sessions/cache/logs.", "Do not promote to global config until this project-local smoke passes."]
  summary: "Verified project-local pi-tandem runtime loading and smoke coverage."
  evidence: ["Merged c7b066e via 11d724c; integrated validation passed."]
  filesChanged: ["extensions/pi-tandem/tests/pi-runtime-smoke.ts", "extensions/pi-tandem/tests/smoke.ts"]
  reviewer: "pi"
  updatedAt: "2026-06-28T02:04:09Z"
completedAt: "2026-06-28T02:04:09Z"
completion:
  summary: "Integrated project-local pi-tandem runtime smoke coverage and documentation."
  validation: "bun --check extensions/pi-tandem/index.ts extensions/pi-tandem/tests/smoke.ts extensions/pi-tandem/tests/pi-runtime-smoke.ts; bun extensions/pi-tandem/tests/smoke.ts; bun extensions/pi-tandem/tests/pi-runtime-smoke.ts; cargo test --manifest-path tandem-tui/Cargo.toml"
---

## Description

Install and smoke-test `pi-tandem` as a project-local Pi extension in this Tandem repository.

Acceptance direction:
- Use Pi's project-local extension mechanism (`.pi/extensions/...`) or `pi -e ./extensions/pi-tandem/index.ts` for testing.
- Keep committed source under `extensions/pi-tandem/`; avoid committing runtime/private `.pi` state unless an intentional lightweight project-local loader/symlink is explicitly appropriate.
- Verify `/reload` or fresh Pi startup discovers the extension in this repo.
- Smoke the core tools against this repo's `.tandem` board: locate/status, list/show/add/move or accord dry-run-safe flows, logs, rules, decisions, and search.
- Document exact test/install steps and any caveats.
