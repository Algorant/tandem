---
id: task-228
type: task
title: "Define a tidy-up procedure for task acceptance and archival to Logs"
state: todo
priority: "medium"
references: ["decision-8"]
relatedFiles: ["AGENTS.md", ".tandem/tandem.md"]
tags: ["rules", "logs", "validation", "git"]
createdAt: "2026-08-16T17:01:03Z"
updatedAt: "2026-08-16T17:01:03Z"
---

## Description

## Goal

Define a repeatable tidy-up procedure that runs whenever a task leaves active work: when it moves from `validation` to accepted, or when it is completed/canceled and archived into `.tandem/logs/`. The procedure keeps the Tandem side of the repository tidy and keeps `main` history clean.

## Problem

Task lifecycle work produces two kinds of residue:

1. **Tandem residue**: uncommitted or scattered `.tandem/` board, log, rule, and decision changes; stale references; task bodies that no longer match the shipped outcome.
2. **Git residue**: several small coordination and work-in-progress commits created while the task moved through `todo` → `in-progress` → `validation` → accepted.

Today there is no defined checkpoint that cleans both up, so `main` history accumulates noisy intermediate commits and `.tandem/` changes can sit uncommitted (see prefer rule 6).

## Scope

Produce durable guidance, not a one-off cleanup. Deliver:

- A written tidy-up checklist for the acceptance/archival boundary, covering:
  - verify the archived log document reflects the real outcome, summary, and validation evidence;
  - verify board state, accord status, and `review:` metadata are consistent;
  - resolve or close related Papercuts and dangling references;
  - commit the `.tandem/` changes as part of the tidy-up commit.
- A Git history policy for the same boundary: squash the intermediate commits created for that task into one coherent commit (or a small, meaningful set) before or while integrating into `main`.
- One or more Tandem rules (`always` / `prefer`) recording the procedure, so agents apply it without re-deriving it.
- An `AGENTS.md` note pointing at the procedure, if the rules alone are not enough context.

## Open questions

- Squash mechanism: squash-merge of a Task worktree branch into `main`, interactive rebase on `main`, or both depending on whether the work used an isolated worktree (see prefer rule 5)?
- Commit message convention for the squashed commit: keep the existing `coord:` / area prefixes and reference the task ID?
- Should the tidy-up commit be separate from the squashed work commit, or folded into it?
- Should this be automated later (a `just` recipe or `tandem`-adjacent helper), or stay a documented manual checklist in v0?

## Non-goals

- Do not rewrite already-pushed `main` history.
- Do not add new Tandem CLI commands as part of this task.

