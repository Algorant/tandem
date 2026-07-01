---
title: Concepts
description: Core Tandem vocabulary and workflow model.
---

# Concepts

Tandem coordinates project work through plain files and explicit lifecycle states.

## Workspace

A Tandem workspace stores coordination data in `.tandem/` inside a repository. Active work lives in `.tandem/board/`, completed work lives in `.tandem/logs/`, and per-actor `.tandem/events/<actor_id>.jsonl` logs record a lightweight audit trail while legacy `.tandem/events.jsonl` remains readable during transition.

## Work documents

Each active task or decision is a Markdown document with YAML frontmatter. Markdown stays readable in any editor while Tandem tools provide structured views and safe mutations.

## Epics

An epic is a convention on a normal task, not a separate protocol object. Mark a grouping task with `type: task` and `kind: epic`, then create child tasks with `parentId` pointing at the epic. Use `references` for loose related context such as decisions, sibling tasks, or completed logs.

Epics complete through the normal task completion/archive flow after their children are done, canceled, or intentionally superseded. Do not create `type: epic`, ADR-style epic records, or a special done state.

## State

Task workflow uses `state` values. The default active states are `todo`, `in-progress`, and `validation`. Completion archives a task into logs instead of moving it to a permanent `done` column.

## Accord

An accord is the explicit work agreement for a task. Its statuses include `ready`, `claimed`, `delivered`, `accepted`, `rework`, `failed`, and `blocked`.

## Validation and review

Tandem keeps human workflow state, accord status, and validation/review metadata separate so agents can deliver work without pretending human review has happened.

## Logs

Completed logs are first-class project history. They preserve the task body, completion summary, validation notes, and relevant metadata for future search and audit.
