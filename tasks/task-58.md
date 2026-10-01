---
id: task-58
uid: 498cd99a-cf8d-4c9d-a724-29f01431d70c
type: task
title: "Update or retire extensions/pi-tandem for research and papercut kinds"
state: "in-progress"
priority: "low"
effort: "small"
blockers: ["task-55"]
references: ["decision-10", "task-55"]
tags: ["pi-tandem", "taxonomy"]
accord:
  status: "delivered"
  acceptance: ["Either `extensions/pi-tandem` classifies and lists papercuts by `--kind papercut` (no tag path remains), or the adapter is retired with that decision recorded.", "The adapter's tests or smoke checks reflect the chosen outcome."]
  claimedAt: "2026-10-01T23:36:03Z"
  deliveredAt: "2026-10-01T23:47:21Z"
  summary: "Re-submitted against commit da8af73 with no code changes. Retired extensions/pi-tandem per decision-10 and deleted the whole directory. The live docs and wiring now point to the tandem CLI with --json as the integration surface. extensions/ stays as the home for future integrations. AGENTS.md, .tandem/ and .tandem_old/ are untouched. No banners were added to historical plan docs, and the generic \"adapter\" wording is unchanged, as you decided."
  evidence: ["Adapter retired with the decision recorded (AC1): `extensions/pi-tandem` is deleted, so no tag path remains. The retirement is recorded in extensions/README.md, extensions/plan/spec.md, extensions/plan/todo.md (citing decision-10 and task-58), plan/spec.md, plan/todo.md and docs/extensions/index.md. docs/extensions/index.md documents `tandem add task --kind papercut` and `tandem list --kind papercut --json` as the CLI path.", "Adapter tests and smoke checks reflect the retirement (AC2): The `bun --check` line and the three pi-tandem smoke-test lines are removed from the `justfile` release recipe and from tandem/RELEASE.md. `just dev-check` passes at da8af73. Nothing in the tree runs or references the removed adapter outside the historical records.", "Remaining grep hits and why each is acceptable: Historical records: RELEASES.md and the dated plan documents (plan/agent-adapter-implementation-handoffs.md, plan/cli-protocol-cutover.md, plan/delegated-task-tree-worker-spec.md, plan/refactor_spec.md, plan/task-40-pi-handoff.md, plan/task-49-pi-handoff.md, plan/papercuts.md, protocol/plan/independent-metadata-sync.md, tandem/plan/clap-migration-research.md). Deliberate retirement statements: docs/extensions/index.md, extensions/README.md, extensions/plan/spec.md, extensions/plan/todo.md, plan/spec.md and plan/todo.md. Not scanned or changed: AGENTS.md (yours), .tandem and .tandem_old. site/src/content/docs/extensions/index.md is an untracked generated copy of docs/extensions/index.md that the build script regenerates."]
  filesChanged: ["extensions/pi-tandem/ (deleted)", "extensions/README.md", "extensions/plan/spec.md", "extensions/plan/todo.md", "docs/extensions/index.md", "docs/guides/agents-and-adapters.md", "README.md", "plan/spec.md", "plan/todo.md", "tandem/README.md", "tandem/RELEASE.md", "justfile"]
  updatedAt: "2026-10-01T23:47:21Z"
createdAt: "2026-10-01T22:21:05Z"
updatedAt: "2026-10-01T23:47:21Z"
relatedFiles: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests", "docs/extensions/index.md"]
assignee: "worker-task-58-da6de440"
---

## Description

Explicit adapter Task, as rule never-2 requires. `tandem_papercut` currently adds `--tag papercut`, and `list` filters by that tag. The ~/.pi configuration uses its own Tandem tools and no longer calls `tandem_papercut`, so retiring the adapter is a valid outcome.
