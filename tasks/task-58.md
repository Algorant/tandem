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
  status: "rework"
  acceptance: ["Either `extensions/pi-tandem` classifies and lists papercuts by `--kind papercut` (no tag path remains), or the adapter is retired with that decision recorded.", "The adapter's tests or smoke checks reflect the chosen outcome."]
  claimedAt: "2026-10-01T23:36:03Z"
  deliveredAt: "2026-10-01T23:46:38Z"
  summary: "Retired extensions/pi-tandem per decision-10. Deleted the whole directory (code, tests, plan, docs). Redirected the live docs and wiring to the tandem CLI with --json as the integration surface. Kept extensions/ with a short README and plan/spec.md and plan/todo.md. Left AGENTS.md, .tandem/ and .tandem_old/ untouched. Committed as da8af73."
  evidence: ["Adapter retired with the decision recorded (AC1): `git rm -r extensions/pi-tandem` removed the code, tests, plan and docs. No tag path remains because the code is gone. The retirement is recorded in extensions/README.md, extensions/plan/spec.md, extensions/plan/todo.md (citing decision-10 and task-58), plan/spec.md, plan/todo.md and docs/extensions/index.md. docs/extensions/index.md now documents `tandem add task --kind papercut` and `tandem list --kind papercut --json` as the CLI path.", "Adapter tests and smoke checks reflect the retirement (AC2): The four pi-tandem lines (one `bun --check` and three smoke runs) are gone from the `justfile` release recipe and from tandem/RELEASE.md. tandem/RELEASE.md also no longer has the `pi -e` smoke example, and its install section now says 'integrations'. No CI wiring referenced pi-tandem: there is no .github hit, and the grep of the remaining tree finds none. site/ has only a generated copy of the Extensions docs page, which is not tracked.", "Remaining grep hits and why each is acceptable: Archived or historical: RELEASES.md (3, release history); plan/agent-adapter-implementation-handoffs.md (7), plan/cli-protocol-cutover.md (8), plan/delegated-task-tree-worker-spec.md (4), plan/refactor_spec.md (4), plan/task-40-pi-handoff.md (7), plan/task-49-pi-handoff.md (3), plan/papercuts.md (1, a resolved record), protocol/plan/independent-metadata-sync.md (1) and tandem/plan/clap-migration-research.md (9), all dated records. Deliberate retirement statements: docs/extensions/index.md (1), extensions/README.md (1), extensions/plan/spec.md (1), extensions/plan/todo.md (1), plan/spec.md (2) and plan/todo.md (1). Not scanned: AGENTS.md (yours), .tandem and .tandem_old (excluded), and site/src/content/docs/extensions/index.md (1, an untracked generated copy of docs/extensions/index.md that site/scripts/sync-docs.mjs regenerates, so it will lose the pi-tandem mention on the next build)."]
  filesChanged: ["extensions/pi-tandem/ (deleted)", "extensions/README.md", "extensions/plan/spec.md", "extensions/plan/todo.md", "docs/extensions/index.md", "docs/guides/agents-and-adapters.md", "README.md", "plan/spec.md", "plan/todo.md", "tandem/README.md", "tandem/RELEASE.md", "justfile"]
  note: "No code changes are needed; the work is accepted as delivered. worker_finish rejected your ready report as stale for the current Task scope: the assignment definition changed after you started (task-58's references now resolve to decision-10). Re-read the current assignment and re-submit the same ready report against commit da8af73. A fresh `just dev-check` is enough as validation. I've also decided on your two questions: no banners on historical plan docs, and leave the generic \"adapter\" wording alone."
  updatedAt: "2026-10-01T23:46:39Z"
createdAt: "2026-10-01T22:21:05Z"
updatedAt: "2026-10-01T23:46:39Z"
relatedFiles: ["extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests", "docs/extensions/index.md"]
assignee: "worker-task-58-da6de440"
---

## Description

Explicit adapter Task, as rule never-2 requires. `tandem_papercut` currently adds `--tag papercut`, and `list` filters by that tag. The ~/.pi configuration uses its own Tandem tools and no longer calls `tandem_papercut`, so retiring the adapter is a valid outcome.
