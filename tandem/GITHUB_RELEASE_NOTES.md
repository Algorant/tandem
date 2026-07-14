# Tandem v0.5.0

Tandem v0.5.0 introduces first-class hierarchical subtasks across the protocol, CLI, TUI, public documentation, and repository-local Pi integration.

## Highlights

- Create full child tasks with `tandem add --parent <task-id>`.
- Automatically allocate parent-derived IDs such as `task-103-1` and nested `task-103-1-1`.
- Navigate recursive task hierarchies in Epic Board.
- Preserve completed-child history without reusing IDs.
- Expose parent and subtask relationships consistently in human and JSON output.

## Bug fixes

- Fixed `tandem show --json` omitting `parentId`.
- Fixed new child tasks receiving unrelated flat IDs instead of parent-derived IDs.
- Prevented concurrent task creation from overwriting occupied destinations.
- Prevented completed child IDs from being reused.
- Fixed non-task parents being presented as subtasks instead of generic parent relationships.
- Fixed reparenting ambiguity by preserving immutable IDs and warning when their designation no longer matches the new parent.
- Fixed Epic Board filters dropping the ancestor path of matching nested tasks.
- Fixed active descendants disappearing when an intermediary parent had already moved to Logs.
- Fixed Review details misclassifying children whose task parent was in Logs.
- Fixed inconsistent and overly verbose Epic Board child-row alignment.

## Protocol

- Defined a first-class subtask as a normal task whose `parentId` resolves to another task.
- Child tasks retain their own workflow, owner, accord, validation, blockers, references, and completion history.
- New children receive parent-derived sequential IDs by default, with arbitrary nested depth.
- Allocation scans active Board documents and completed Logs.
- `parentId` remains canonical; ID shape alone does not establish hierarchy.
- Existing flat-ID children remain valid, and task IDs remain immutable during reparenting.
- Inline `subtasks` checklist data remains readable but is deprecated for new tracked work.
- Decision and custom-document parents remain generic relationships rather than subtasks.

## CLI

- Added hierarchical and nested allocation to `tandem add --parent`.
- Added collision-safe concurrent task creation and completed-log sequence continuity.
- Added `--parent` filtering to list and search.
- Added parent relationship information to show, list, and search output.
- Added `tandem add --json`.
- Added computed `parentRelationship` values for task and non-task parents.
- Added immutable-ID warnings when reparenting changes a task's expected designation.
- Deprecated inline `--subtask` authoring in favor of separate child tasks.

## TUI

- Epic Board now recursively displays active task descendants.
- Child rows use compact `SUB` and state labels such as `TODO`, `WIP`, and `VAL`.
- Relationships use a concise `<parent> → <child>` column.
- Filters retain matching descendants and their ancestor context.
- Existing flat-ID children remain visible in the hierarchy.
- Epic rows summarize active and logged descendants without returning completed tasks to active rows.
- Traversal continues through completed intermediary parents to active descendants.
- Board and Review details distinguish subtasks from generic parent relationships.

## Pi integration

- Updated repository-local `pi-tandem` for hierarchical child tasks.
- Pi passes `parent` to Tandem and does not construct IDs itself.
- Added coverage for nested allocation, completed-log continuity, generic parents, legacy flat children, and allocation collisions.
- Child tasks can be created through `tandem_task` and delegated using the returned ID.

## Documentation

- Added guidance for epics, ordinary parent tasks, first-class subtasks, blockers, references, and legacy inline checklists.
- Added CLI examples for creation, inspection, filtering, and reparenting.
- Documented the updated Epic Board hierarchy and visual language.
- Added an event-log storage options research note.

## Upgrade notes

- Existing flat-ID children do not require migration.
- Existing inline `subtasks` data remains readable.
- New tracked child work should use a separate task with `--parent`.
- Reparenting does not rename tasks or rewrite references.
