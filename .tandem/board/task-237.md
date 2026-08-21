---
id: task-237
type: task
title: "Audit and accelerate the `just release` workflow"
state: todo
priority: "medium"
effort: "medium"
references: ["task-236", "task-185"]
relatedFiles: ["justfile", "scripts/release_checks.py", "scripts/tests/test_release_checks.py", "tandem/RELEASE.md", "tandem/dist-workspace.toml"]
tags: ["release", "performance", "automation"]
createdAt: "2026-08-21T21:41:14Z"
updatedAt: "2026-08-21T21:41:14Z"
---

## Description

## Goal

Reduce the end-to-end time and avoidable friction of `just release <version>` without weakening release correctness or publication verification.

## Scope

- Measure the current release path stage by stage on representative warm and cold runs.
- Identify the main time sinks, duplicated validation, serial work that can safely run in parallel, ineffective caching, unnecessary rebuilds, network waits, and manual steps.
- Separate mandatory safety checks from redundant or misplaced work.
- Propose and implement the smallest high-confidence speedups where practical.
- Document larger follow-up opportunities with expected benefit, risk, and implementation cost.

## Acceptance criteria

1. A reproducible baseline records total duration and major stage timings.
2. The dominant bottlenecks are identified with evidence rather than guesswork.
3. Safe improvements are implemented and covered by relevant tests or validation.
4. Before/after timings quantify the result.
5. Release guarantees remain intact, including version and notes checks, build/test validation, artifact publication verification, installer verification, and downstream packaging checks.
6. Any deferred opportunities are recorded as concrete follow-up tasks or recommendations.
