# pi-tandem Todo

Status: MVP implementation
Last updated: 2026-07-10

## Accomplished

- [x] Defined `pi-tandem` as a lightweight Pi adapter over installed `tandem`.
- [x] Added `tandem_status`, `tandem_task`, `tandem_accord`, `tandem_log`, `tandem_rules`, `tandem_decision`, and `tandem_search` tools.
- [x] Added `/tandem help|status` command.
- [x] Added diagnostics for missing `tandem`, missing `.tandem`, unsupported CLI surface, timeout/abort, and command failures.
- [x] Added prompt snippets/guidelines and workspace-aware prompt guidance.
- [x] Added smoke test coverage for CLI-backed wrapper mappings.
- [x] Added repo read smoke coverage for this workspace's `.tandem` board.
- [x] Added a project-local Pi runtime smoke that creates an ignored `.pi/extensions/pi-tandem/index.ts` loader, verifies fresh RPC startup discovers `/tandem`, runs `/tandem status`, and cleans up.
- [x] Added relationship guidance and smoke coverage for first-class parent-linked child tasks, CLI-owned hierarchical/nested allocation, completed-log sequence continuity, generic non-task parents, existing flat-ID child compatibility, allocation collision errors, CLI-computed `parentRelationship`/subtask summaries, blockers, references, and related files; deprecated inline checklist authoring is no longer exposed.

## Current tasks

- [ ] Collect review feedback on tool schemas before global promotion.
- [ ] Optionally run an interactive TUI `/reload` smoke if a human wants visual confirmation beyond the automated fresh-start RPC smoke.

## Next recommended steps

1. Review the project-local smoke results and decide whether they satisfy task-14.
2. Decide whether a dedicated `tandem_init`/workspace tool is needed or whether workspace bootstrap should remain a manual CLI action.
3. Promote accepted extension code into canonical Pi config in a separate task.

## Open questions

- Should mutation commands gain JSON output in `tandem`, letting this adapter return structured mutation details without parsing human output?
- Should Pi UI autocomplete offer task IDs from `tandem list --json`?
