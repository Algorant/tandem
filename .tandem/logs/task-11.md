---
id: task-11
type: task
title: "TUI Decisions view is always empty because reload never reads the decisions directory"
priority: "high"
effort: "small"
relatedFiles: ["tandem/src/tui/reload.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/state.rs", "tandem/src/project/mod.rs"]
tags: ["tui", "decisions", "bug"]
accord:
  status: "accepted"
  acceptance: ["The TUI Decisions tab lists every document in .tandem/decisions/, with the tab count matching `tandem list --type decision`.", "Selecting a decision renders its detail, and the Board tab still excludes decisions.", "A decision created or edited outside the running TUI appears after the reload interval, so the reload fingerprint covers the decisions directory.", "A regression test loads a workspace containing at least one decision and asserts the Decisions view is non-empty, so a future refactor cannot silently reintroduce a board-only load."]
  claimedAt: "2026-09-04T01:45:19Z"
  deliveredAt: "2026-09-11T13:32:41Z"
  summary: "Reconciled previously shipped Decisions reload fix (0354d7d, released in 0.12.3). Algorant authorized closeout; current 0.13.1 regression and rendered checks pass."
  evidence: ["cargo test --manifest-path tandem/Cargo.toml --release reload_loads_decisions_and_detects_external_decision_changes exited 0; test checks non-empty Decisions tab, Board exclusion and external decision-edit fingerprint detection.", "Release TUI in a native CLI-created disposable workspace rendered Decisions (1), selected decision title and body, and Board (1). After a second decision was added via external CLI while the TUI remained open, ANSI capture showed Decisions (2), both records and unchanged Board (1), without manual reload.", "git merge-base --is-ancestor 0354d7d tandem-v0.12.3 exited 0; RELEASES.md documents this shipped fix. Git history and native events showed the task remained claimed with no delivery/completion; this is a retrospective closeout, not new implementation.", "Historical Pi source at ee232a4 explicitly described worker_integrate as technical integration that does not accept or complete Tasks. Combined worker_finish was introduced later in 9be3e34 (September 6). Exact original Worker sessions/finish outcomes were not found, so skipped versus interrupted closeout is unknown."]
  filesChanged: ["tandem/src/project/mod.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/state.rs"]
  updatedAt: "2026-09-11T13:32:45Z"
createdAt: "2026-09-03T23:26:13Z"
updatedAt: "2026-09-11T13:32:45Z"
assignee: "worker-task-11-690d1eb5"
archivedAt: "2026-09-11T13:32:45Z"
resolution:
  outcome: "completed"
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

