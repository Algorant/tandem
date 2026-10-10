---
id: task-66
uid: e0229697-98d6-4aa5-98ef-f46bab7fa54f
type: task
title: "Docs and agent guidance disagree with the 0.16.2 CLI lifecycle (move, update --status, deliver, review, complete)"
state: todo
priority: "medium"
references: ["task-52", "task-51", "decision-8", "Algorant/.pi@9719220"]
relatedFiles: ["docs/cli/index.md", "docs/guides/agents-and-adapters.md", "docs/guides/decisions.md", "docs/concepts/index.md", "AGENTS.md", "tandem/README.md", "protocol/README.md"]
tags: ["docs", "accord", "review"]
accord:
  status: "ready"
  acceptance: ["docs/cli/index.md documents exactly the 0.16.2 clap command tree (verified against `tandem --help` and each subcommand's `--help`), with no `move`, `log`, `decision`, `papercut`, `upgrade`, `accord ready|accept`, or Task `update --status`, and with sections for `review`, `assignment`, `link`, and `accord resume|release`.", "The docs state the actual lifecycle: deliver requires --summary and non-empty --evidence and leaves state unchanged; validation is reached only through `tandem review <id> --criterion <exact acceptance criterion> --note`; `complete` accepts a delivered Accord and archives; completing an undelivered Task warns but succeeds, and the docs say so.", "docs/guides/agents-and-adapters.md, docs/guides/decisions.md, and docs/concepts/index.md contain no removed commands and no claim that delivery moves work to validation or that review.status is stored.", "The 'Locked v0' CLI command lists in AGENTS.md and tandem/README.md match the current CLI and protocol version, or are explicitly marked historical.", "A repository-wide search of docs/, AGENTS.md, README.md, and tandem/README.md for `tandem move`, `tandem log`, `tandem decision`, `accord accept`, and Task `--status` finds no live instruction.", "task-52 is resolved as fixed by this Task."]
  validation: ["$ just site-build", "Manual: rg for `tandem move`, `tandem log`, `tandem decision`, `accord accept`, and Task `--status` across docs/, AGENTS.md, README.md, tandem/README.md finds no live instruction."]
  updatedAt: "2026-10-10T14:10:00Z"
createdAt: "2026-10-10T13:43:38Z"
updatedAt: "2026-10-10T14:10:00Z"
blockers: ["task-61", "task-62"]
effort: "medium"
---

## Description

## Context

Reported by the fframes Ceph via Herdra (2026-10-09) while scripting real `tandem` commands against 0.16.2; verified by the tandem Ceph against the code (`tandem/src/protocol/accord.rs`, `app/accord.rs`, `app/review.rs`, `protocol/diagnostic.rs`) and a disposable sandbox run of 0.16.2.

**Verdict: the docs are wrong; the CLI is right.** CLI behavior matches the normative `protocol/README.md` (lines 62, 66, 112). The stale material is pre-0.3.0 (decision-8) reference text that was never rewritten.

## Verified CLI behavior (0.16.2)

- `tandem move` does not exist (`unrecognized subcommand 'move'`). No command sets workflow state directly; state changes only through `accord claim|rework|release`, `review`, `complete`, `cancel`.
- `update --status` exists only for Decisions; on a Task it fails: `update flags not valid for task documents: --status`.
- `accord deliver` requires `--summary` and at least one non-empty `--evidence`; it sets `accord.status: delivered` and leaves `state` unchanged (stays `in-progress`). It does not move to `validation`.
- There is no `accord accept`. Acceptance happens in `tandem complete`, which accepts a delivered Accord and archives atomically.
- `validation` is reached only via `tandem review <id> --criterion <exact acceptance criterion> --note <text> [--reviewer]`; a non-matching criterion is rejected with the list of expected criteria. `review` has no subcommands.
- `complete <id> [--note] [--reviewer] [--fixed-by]` (no `--summary`/`--file-changed`/`--validation`). Completing a Task whose Accord is neither delivered nor accepted prints `Warning: <id> has accord.status=<s>; complete normally follows a delivered Accord.` and still completes. This is by design (protocol/README.md:62), not a bug; the docs should say when to expect it.
- `add` is `add task <TITLE>` / `add decision` (positional title), not `add --title`.
- No `log`, `decision`, `papercut`, `upgrade` command families; `list/search --scope archived` and `list --type decision` replace them.

## Known mismatches

- `docs/cli/index.md`: `tandem move` section (233-252) and the `update` note pointing to it (274); `add --title ... --state` syntax (216-222); `complete --summary/--file-changed/--validation` (298-306); `tandem log list|show|search` (386-510); `accord ready`/`accord accept`, `deliver` evidence shown optional, and the claim that `deliver`/`accept` move work to `validation` (581-625); `tandem decision list|show|add|update|withdraw` (687-848); missing sections for `review`, `assignment`, `link`, `accord resume|release`; `--review` list filter. (Overlaps papercut task-52, which this Task absorbs.)
- `docs/guides/agents-and-adapters.md:62-68`: lists `review.status` as a stored signal and says synchronization moves delivered work to `validation`.
- `docs/guides/decisions.md:89-104`: `tandem decision add|list|show`.
- `docs/concepts/index.md:85-86`: `tandem log list|show`.
- `AGENTS.md` "Locked v0 decisions": CLI command list includes `move`, `log`, `decision`; `accord accept`; protocol `0.2.0`; `review` as a state. These "locked" lines contradict decision-8/decision-9 and mislead every agent that reads the file.
- `tandem/README.md:280-293` "Locked v0 CLI/TUI decisions": same stale command list (`move`, `log`, `papercut`, `decision`, `accord accept`).
- Out of repo (route to the pi Ceph, not fixed here): `~/.pi/agent/skills/tandem/SKILL.md:39` says `deliver` "accepts evidence" (it requires it; line 45 of the same file says so) and "completion is separate from Accord acceptance" (complete accepts a delivered Accord); it never says `review` needs an exact acceptance criterion or that `deliver` does not move state; line 43 still says "Tandem 0.15".

## Out of scope

No CLI behavior change. Whether complete-without-delivery should stay a warning is a separate product question.
