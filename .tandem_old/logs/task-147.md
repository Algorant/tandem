---
id: task-147
type: task
title: "Finalize the refactor specification and compatibility policy"
priority: "high"
parentId: "task-146"
references: ["task-145", "decision-7"]
relatedFiles: ["plan/refactor_spec.md", "tandem/plan/modularization-research.md", "protocol/plan/spec.md", "plan/spec.md", "tandem/plan/spec.md", "AGENTS.md"]
tags: ["docs", "protocol", "architecture", "refactor", "compatibility"]
createdAt: "2026-07-22T19:17:28Z"
updatedAt: "2026-07-22T20:24:44Z"
completedAt: "2026-07-22T20:24:44Z"
completion:
  summary: "Closed without implementation: this planning conversation was mistakenly recorded as a lifecycle-bearing Task. The project owner correctly directed that the remaining specification choices be resolved directly before real implementation Tasks are created."
  validation: "No implementation was started or claimed, and no source or specification change is attributed to this mistaken Task. The record is retained only to prevent ID reuse and preserve an honest audit trail."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Finish `plan/refactor_spec.md` as the reviewable, decision-ready architecture specification for Epic task-146 before any Rust refactor or integration-branch work begins.

## Scope

- Apply all twelve architecture answers already agreed with the project owner.
- Use `protocol`, `project`, `app`, `cli`, and `tui` as the proposed top-level ownership boundaries.
- Use `project::TandemProject` for one discovered/opened Tandem project and its concrete `.tandem/` files.
- Remove remaining architectural use of vague `workspace` and rejected `persistence` terminology while preserving quotations or historical context only where necessary.
- Keep file-format interpretation/validation in protocol and concrete file access/raw minimal patching in project.
- Retain configurable active workflow states.
- Specify built-in `task` and `decision` document types only, fixed priority/effort sets, and canonical warn-but-complete behavior when review/accord acceptance is missing.
- Specify the Rust freeze on `main`, precise temporary Clippy expectations, real-command tests, `tui/mod.rs`, and a decision that locks layers/dependency direction rather than every leaf filename.
- Reconcile the proposal with the currently accepted protocol and existing docs without hiding behavior changes inside architectural movement.

## Compatibility questions to resolve

- How existing config-defined custom type declarations and custom documents are read, reported, preserved, or migrated.
- Whether removing custom-type support requires a protocol-version change.
- How existing project completion-policy fields are handled after completion warning behavior becomes canonical.

## Deliverable and acceptance criteria

- `plan/refactor_spec.md` is internally consistent, concise enough to review, and contains no answered questions as if they were still open.
- Every remaining open compatibility choice is explicitly resolved or presented to the owner one at a time for decision.
- Normative protocol docs versus executable Rust protocol ownership are unambiguous.
- The specification clearly defines Task ordering, branch operation, module checkpoints, lint cleanup, behavior tests, documentation/agent-guidance scope, and final merge criteria.
- No Rust source, Cargo, release, installed binary, personal configuration, Epic lifecycle, or production behavior changes occur.
- The final document is ready to support the broad architecture decision and remaining direct Tasks under task-146.

Creating this Task does not authorize implementation; leave it in `todo` until explicitly started.
