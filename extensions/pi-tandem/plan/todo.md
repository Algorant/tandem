# pi-tandem Todo

Status: MVP implementation
Last updated: 2026-07-22

## Accomplished

- [x] Defined `pi-tandem` as a lightweight Pi adapter over installed `tandem`.
- [x] Added `tandem_status`, `tandem_init`, `tandem_task`, `tandem_accord`, `tandem_log`, `tandem_rules`, `tandem_decision`, and `tandem_search` tools.
- [x] Added `/tandem help|status` command.
- [x] Added diagnostics for missing `tandem`, missing `.tandem`, unsupported CLI surface, timeout/abort, and command failures.
- [x] Added prompt snippets/guidelines and workspace-aware prompt guidance.
- [x] Added smoke test coverage for CLI-backed wrapper mappings.
- [x] Added repo read smoke coverage for this workspace's `.tandem` board.
- [x] Added a project-local Pi runtime smoke that creates an ignored `.pi/extensions/pi-tandem/index.ts` loader, verifies fresh RPC startup discovers `/tandem`, runs `/tandem status`, and cleans up.
- [x] Aligned relationship guidance and Bun smokes with strict Epic → global Task → parent-derived leaf Subtask roles; pi-tandem forwards kind/parent and consumes CLI-returned `epic-task`, `subtask`, and generic `parent` without reclassification.
- [x] Documented Task-only delegation: one Task worker owns its Subtasks through the todo projection, while Epics and Subtasks are not delegation roots.
- [x] Added strict invalid-structure smoke coverage for nested Epics, children beneath Subtasks, role-changing reparenting, erroneous hierarchical direct Epic children, and erroneous global-ID Subtasks.

## Current tasks

- [ ] Collect review feedback on tool schemas before global promotion.
- [ ] Optionally run an interactive TUI `/reload` smoke if a human wants visual confirmation beyond the automated fresh-start RPC smoke.

## Next recommended steps

1. Complete review and integration of the canonical hierarchy guidance and smoke coverage.
2. Apply and validate the separate canonical Pi-config handoff in `plan/delegated-task-tree-worker-spec.md`, then reload/restart Pi; do not modify personal dotfiles from this repository.
3. After repository and Pi-config acceptance, review promotion of the project-local adapter into shared Pi config and run an interactive `/reload` smoke if human confirmation is desired.

## Open questions

- Should mutation commands gain JSON output in `tandem`, letting this adapter return structured mutation details without parsing human output?
- Should Pi UI autocomplete offer task IDs from `tandem list --json`?
