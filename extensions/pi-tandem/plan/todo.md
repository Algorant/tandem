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

## Current tasks

- [ ] Keep `README.md`, `plan/spec.md`, and `plan/todo.md` synchronized with parent and extension-area docs.
- [ ] Run a real Pi runtime smoke with `pi -e extensions/pi-tandem/index.ts` after code review.
- [ ] Collect review feedback on tool schemas before global promotion.

## Next recommended steps

1. Project-local install/smoke in Pi without editing global config.
2. Decide whether a dedicated `tdm_init`/workspace tool is needed or whether workspace bootstrap should remain a manual CLI action.
3. Promote accepted extension code into canonical Pi config in a separate task.

## Open questions

- Should mutation commands gain JSON output in `tdm`, letting this adapter return structured mutation details without parsing human output?
- Should Pi UI autocomplete offer task IDs from `tdm list --json`?
