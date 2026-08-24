---
id: task-228
type: task
title: "Define a tidy-up procedure for task acceptance and archival to Logs"
state: todo
priority: "medium"
references: ["decision-8", "papercut-4", "papercut-5"]
relatedFiles: ["AGENTS.md", ".tandem/tandem.md"]
tags: ["rules", "logs", "validation", "git"]
createdAt: "2026-08-16T17:01:03Z"
updatedAt: "2026-08-24T22:32:43Z"
blockers: ["task-239-4"]
---

## Description

## Goal

Define a repeatable tidy-up procedure that runs whenever a task leaves active work, keeping the Tandem side of the repository consistent with what actually shipped.

## Blocked on task-239-4

This task no longer decides anything. Its open questions moved into task-239-4, which settles the acceptance/archival boundary alongside the validation rules in one design session. Do not start this work until task-239-4 closes with a decision record; the trigger for tidy-up depends on whether `validation` survives in its current form.

## Problem

Task lifecycle work leaves Tandem residue: uncommitted or scattered `.tandem/` board, log, rule, and decision changes; stale references; task bodies that no longer match the shipped outcome; and completions that warn `review.status=missing` because nothing in the flow ever sets it.

## Scope

Produce durable guidance, not a one-off cleanup:

- A written tidy-up checklist for whatever boundary task-239-4 defines, covering:
  - verify the archived log document reflects the real outcome, summary, and validation evidence;
  - verify board state, accord status, and `review:` metadata are consistent;
  - resolve or close related Papercuts and dangling references;
  - commit the `.tandem/` changes as part of the tidy-up.
- One or more Tandem rules (`always` / `prefer`) recording the procedure so agents apply it without re-deriving it.
- An `AGENTS.md` note pointing at the procedure, if the rules alone are not enough context.

## Removed from scope

The Git-history half moved to its own task. The original squash guidance was written before `.tandem/` state was auto-committed and no longer describes how commits are actually produced: Worker branches are already squashed at integration, and the remaining noise comes from an auto-checkpointer whose commits interleave several tasks and cannot be grouped per task.

## Non-goals

- Do not rewrite already-pushed `main` history.
- Do not add new Tandem CLI commands as part of this task.