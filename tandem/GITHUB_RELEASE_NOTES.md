# Tandem v0.6.0

Tandem v0.6.0 hardens the canonical Epic → Task → Subtask hierarchy across the protocol, CLI, TUI, documentation, and repository-local Pi integration.

## Highlights

- Epics and Tasks now use global `task-N` IDs; only leaf Subtasks directly beneath Tasks use parent-derived `task-N-M` IDs.
- Hierarchy roles come from resolved documents: direct Epic children are Tasks (`epic-task`), direct Task children are Subtasks (`subtask`), and decision/custom-document parents remain generic (`parent`).
- The CLI rejects parented Epics, children beneath Subtasks, role/ID mismatches, and role-changing or ID-invalidating reparenting.
- State Board and Epic Board render canonical Tasks and Subtasks consistently, preserve ancestor context while filtering, and keep completed descendants represented through Logs rollups.
- `pi-tandem` passes parent operations through to the CLI and consumes Tandem's role and relationship output without reclassifying it.

## Bug fixes

- Fixed direct Epic children being allocated or labeled as Subtasks instead of global-ID Tasks.
- Fixed invalid nested hierarchy records and incompatible reparenting being accepted through legacy compatibility behavior.
- Fixed workspace validation stopping after the first hierarchy error instead of reporting all structural failures together.
- Fixed Board and Review hierarchy details mislabeling Task, Subtask, and generic-parent relationships.

## Compatibility note

Workspaces containing hierarchical direct Epic children, global-ID Subtasks, parented Epics, or children beneath Subtasks must correct those invalid records before Tandem can load them.
