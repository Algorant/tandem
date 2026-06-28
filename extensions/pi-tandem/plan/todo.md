# pi-tandem Todo

Status: MVP implementation
Last updated: 2026-06-28

## Accomplished

- [x] Defined `pi-tandem` as a lightweight Pi adapter over installed `tdm`.
- [x] Added `tdm_status`, `tdm_task`, `tdm_accord`, `tdm_log`, `tdm_rules`, `tdm_decision`, and `tdm_search` tools.
- [x] Added `/tandem help|status` command.
- [x] Added diagnostics for missing `tdm`, missing `.tandem`, unsupported CLI surface, timeout/abort, and command failures.
- [x] Added prompt snippets/guidelines and workspace-aware prompt guidance.
- [x] Added smoke test coverage for CLI-backed wrapper mappings.
- [x] Added repo read smoke coverage for this workspace's `.tandem` board.
- [x] Added a project-local Pi runtime smoke that creates an ignored `.pi/extensions/pi-tandem/index.ts` loader, verifies fresh RPC startup discovers `/tandem`, runs `/tandem status`, and cleans up.

## Current tasks

- [ ] Collect review feedback on tool schemas before global promotion.
- [ ] Optionally run an interactive TUI `/reload` smoke if a human wants visual confirmation beyond the automated fresh-start RPC smoke.

## Next recommended steps

1. Review the project-local smoke results and decide whether they satisfy task-14.
2. Decide whether a dedicated `tdm_init`/workspace tool is needed or whether workspace bootstrap should remain a manual CLI action.
3. Promote accepted extension code into canonical Pi config in a separate task.

## Open questions

- Should mutation commands gain JSON output in `tdm`, letting this adapter return structured mutation details without parsing human output?
- Should Pi UI autocomplete offer task IDs from `tdm list --json`?
