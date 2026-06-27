---
id: task-14
type: task
title: "Project-local install and smoke pi-tandem"
state: todo
priority: "medium"
tags: ["pi-extension", "pi-tandem"]
createdAt: "2026-06-27T23:30:04Z"
updatedAt: "2026-06-27T23:30:05Z"
accord:
  status: "ready"
  deliverables: ["docs:extensions/pi-tandem/README.md:project-local install and smoke instructions", "test:extensions/pi-tandem/tests/project-smoke.md:manual smoke evidence"]
  validation:
    commands: ["test -f extensions/pi-tandem/index.ts", "rg -n 'project-local|pi -e|/reload|tdm_status|tdm_task' extensions/pi-tandem"]
  constraints: ["Do not commit private Pi sessions/cache/logs.", "Do not promote to global config until this project-local smoke passes."]
  updatedAt: "2026-06-27T23:30:05Z"
---

## Description

Install and smoke-test `pi-tandem` as a project-local Pi extension in this Tandem repository.

Acceptance direction:
- Use Pi's project-local extension mechanism (`.pi/extensions/...`) or `pi -e ./extensions/pi-tandem/index.ts` for testing.
- Keep committed source under `extensions/pi-tandem/`; avoid committing runtime/private `.pi` state unless an intentional lightweight project-local loader/symlink is explicitly appropriate.
- Verify `/reload` or fresh Pi startup discovers the extension in this repo.
- Smoke the core tools against this repo's `.tandem` board: locate/status, list/show/add/move or accord dry-run-safe flows, logs, rules, decisions, and search.
- Document exact test/install steps and any caveats.
