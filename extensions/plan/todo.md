# Tandem Extensions Todo

Status: no active integrations

This todo tracks agent/editor integration work under `extensions/`.

## Accomplished

- [x] Added `extensions/` as the third major child area for Tandem integrations.
- [x] Documented the adapter principle: integrations call `tandem`; protocol behavior stays in `protocol/` and `tandem/`.
- [x] Retired the `pi-tandem` Pi adapter (decision-10, task-58). The `tandem` CLI with `--json` is the integration surface.

## Open questions

- Should a future integration live under `extensions/<target>/` with its own README, or outside this repository?
- Should `tandem` eventually expose structured mutation output so integrations can return richer details without parsing human text?
