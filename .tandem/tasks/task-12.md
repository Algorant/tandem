---
id: task-12
type: task
title: "Workspaces with rules embedded in tandem.md lose every rule silently"
state: "in-progress"
priority: "high"
effort: "small"
relatedFiles: ["tandem/src/project/rules.rs", "tandem/src/tui/reload.rs", "tandem/src/project/mod.rs", "tandem/src/app/rules.rs"]
tags: ["rules", "migration", "bug", "data-loss"]
accord:
  status: "delivered"
  acceptance: ["A workspace with a non-empty rules block in tandem.md and no .tandem/rules directory produces a visible diagnostic from `tandem rules list` and in the TUI Rules view, rather than reporting zero rules silently.", "The diagnostic names the legacy location and states plainly that those rules are not active.", "If migration is implemented, embedded rules become per-rule files under .tandem/rules with categories and source fields preserved, and the operation is reported rather than silent.", "A test covers a workspace fixture carrying embedded rules and asserts the diagnostic or the migration, so the silent-zero path cannot return."]
  claimedAt: "2026-09-04T02:06:22Z"
  deliveredAt: "2026-09-11T13:35:52Z"
  summary: "Reconciled the legacy embedded-rule detection fix (4260cbd, shipped in 0.12.3). Algorant explicitly approved closing this shipped fix while tracking its remaining TUI warning-readability limitation in task-32."
  evidence: ["cargo test --manifest-path tandem/Cargo.toml --release embedded_rules exited 0: protocol populated/empty-block detection and TUI reload warning regressions pass.", "Disposable 0.3.0 fixture with populated rules block and no .tandem/rules directory: rules list --json returned zero active rules with warning naming the rules block in tandem.md, stating rules are not active, and directing users to .tandem/rules/. Human CLI emitted the same diagnostic. No migration occurred and no rules directory was created.", "Rendered 0.13.1 TUI check at 132x61 found the warning text is generated but clipped behind footer hints; after transient expiration it is no longer discoverable. This is not claimed as full TUI readability validation. Algorant chose 'Close and track follow-up' after this gap was disclosed; task-32 owns the remaining presentation issue.", "git merge-base --is-ancestor 4260cbd tandem-v0.12.3 exited 0; RELEASES.md identifies this warn-only fix. Native history contained claim only, with no delivery/completion. Old worker_integrate explicitly did not accept or complete Tasks; the combined worker_finish was added September 6, after this September 3 integration. Exact historical Worker session records were not found, so the reason the separate closeout was omitted remains uncertain."]
  filesChanged: ["tandem/src/app/project.rs", "tandem/src/cli/commands.rs", "tandem/src/protocol/config.rs", "tandem/src/tui/mod.rs"]
  updatedAt: "2026-09-11T13:35:52Z"
createdAt: "2026-09-03T23:26:43Z"
updatedAt: "2026-09-11T13:35:52Z"
assignee: "worker-task-12-3ec94bce"
references: ["task-32"]
---

## Description

## Symptom

A workspace whose `tandem.md` frontmatter contains a populated `rules:` block reports zero rules in both surfaces, with no warning anywhere:

```
$ tandem rules list --json
{"data":[],"ok":true,"warnings":[]}
```

The TUI shows `[3] Rules (0)`. The same `tandem.md` contains six `rules.always` entries with ids 1 through 6, several marked `source: "user"`.

Observed on 0.12.2 in `/home/ivan/.pi`, whose `.tandem/` has `decisions`, `events`, `logs`, `tasks` and no `rules` directory. A workspace created by `tandem init` on 0.12.2 does get a `rules/` directory.

## Cause

Rules are one file per rule under `.tandem/rules/` (comment at `tandem/src/tui/reload.rs:85`), loaded through `crate::project::rules::rules_by_category(&self.workspace.rules_dir())` at `reload.rs:86`. Nothing reads a `rules` key out of the workspace config: grep for `rules_from_root` or any equivalent returns no hits, while `read_config_yaml` is used only for title and states (`reload.rs:95-99`).

So embedded rules are not migrated, not read, and not reported. A workspace that predates the per-file layout keeps a `rules:` block in `tandem.md` that looks authoritative to a human and to an agent reading the file, while every tool reports zero rules.

Both files claim `protocolVersion: 0.3.0`, so the version marker does not distinguish the two layouts. That makes the condition undetectable by version alone and detectable only by the presence of a non-empty embedded block.

## Why this matters

Rules are the mechanism projects use to constrain agent behavior. Silent loss means an agent asking Tandem for active rules is told there are none, while the workspace file it may also read still lists them. In this workspace that covered six standing rules, including ones governing Worker integration and decision-record usage.

## Suggested fix

At minimum, warn: when `tandem.md` carries a non-empty `rules:` block, emit a diagnostic on `rules list` and in the TUI Rules view stating the rules are in a legacy location and are not active.

Better, migrate: convert an embedded block into per-rule files under `.tandem/rules/` on first write, or provide an explicit command that does it and reports what moved.

Deciding between warn-only and automatic migration is a judgment call about how much the tool should rewrite a workspace it did not create.

