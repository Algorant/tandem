---
id: task-11
type: task
title: "TUI Decisions view is always empty because reload never reads the decisions directory"
state: "in-progress"
priority: "high"
effort: "small"
relatedFiles: ["tandem/src/tui/reload.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/state.rs", "tandem/src/project/mod.rs"]
tags: ["tui", "decisions", "bug"]
accord:
  status: "claimed"
  acceptance: ["The TUI Decisions tab lists every document in .tandem/decisions/, with the tab count matching `tandem list --type decision`.", "Selecting a decision renders its detail, and the Board tab still excludes decisions.", "A decision created or edited outside the running TUI appears after the reload interval, so the reload fingerprint covers the decisions directory.", "A regression test loads a workspace containing at least one decision and asserts the Decisions view is non-empty, so a future refactor cannot silently reintroduce a board-only load."]
  claimedAt: "2026-09-04T01:45:19Z"
  updatedAt: "2026-09-04T01:45:19Z"
createdAt: "2026-09-03T23:26:13Z"
updatedAt: "2026-09-04T01:45:19Z"
assignee: "worker-task-11-690d1eb5"
---

## Description

## Symptom

The TUI Decisions tab reports `[4] Decisions (0)` and `no decision documents loaded` in a workspace that has ten decision documents on disk. The detail pane shows `No decision documents. Press a to add one.`

The CLI is correct in the same workspace at the same moment:

```
$ tandem list --type decision --json | jq '.data | length'
10
$ ls .tandem/decisions/ | wc -l
10
```

Reproduced on 0.12.2 against `/home/ivan/.pi`. Board (4) and Logs (52) populate normally in the same session, so the workspace loads.

## Root cause

`TuiApp::reload` populates `self.docs` from board documents only:

- `tandem/src/tui/reload.rs:80-83` calls `self.workspace.read_board_documents_tolerant(&mut load_errors)`.
- `tandem/src/project/mod.rs:205-210` shows that reads `self.tasks_dir`, which is `.tandem/tasks/`.

The Decisions view then filters that same collection:

- `tandem/src/tui/decisions.rs:345-353` (`sorted_decision_docs`) filters `self.docs` with `is_decision_doc`.
- `tandem/src/tui/state.rs:1063-1065` matches `doc.doc_type() == "decision"`.

Decision documents are written to `.tandem/decisions/` (`tandem/src/app/decisions.rs:78` uses `project.decisions_dir()`), which `reload` never reads. `decisions_dir()` has exactly one caller in the whole source tree, in the app layer. So `self.docs` cannot contain a decision, and the filter always yields zero.

This is not workspace-specific and not a migration artifact: no workspace can ever show a decision in the TUI.

## Suggested fix

Read the decisions directory in `reload` alongside board documents and extend `self.docs`, or give `DecisionsState` its own collection loaded from `decisions_dir()`. The second keeps board filtering (`is_board_visible_doc` at `state.rs:1067-1069` already excludes decisions from the Board) from depending on a mixed collection.

Whichever shape is chosen, the reload fingerprint in `tandem/src/tui/reload.rs` should include the decisions directory so external decision writes refresh the open TUI.

