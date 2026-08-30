---
id: task-249
type: task
title: "Explore a contextual lifecycle action picker for the TUI"
state: todo
priority: "low"
effort: "medium"
blockers: ["task-247"]
references: ["task-246", "decision-12"]
relatedFiles: ["tandem/src/tui/", "plan/cli-protocol-cutover.md"]
tags: ["tui", "ui", "research", "accord"]
createdAt: "2026-08-30T17:48:48Z"
updatedAt: "2026-08-30T17:48:48Z"
---

## Description

## Goal

After the protocol 0.3.0 TUI adaptation is stable, explore whether one contextual action picker should expose valid Task/Accord lifecycle operations without changing the current Board list/subview architecture.

This is a proposal/research Task, not required by the core cutover.

## Direction

Preserve the current Board state subviews, full-width rows, detail behavior, and State/Epic arrangement. The candidate picker opens for the selected Task and lists only transitions valid for its current Task state and Accord status. Rare/destructive actions may be separated from the normal path.

Use Sideshow and rendered release TUI panes to compare the picker with CLI-only lifecycle behavior and separate existing validation controls. Do not implement a broad TUI redesign or kanban layout.

## Questions

- Does one picker improve discoverability without creating modal overload?
- Which actions deserve direct keys versus menu-only placement?
- How should required assignee, summary, evidence, criterion, reviewer, and note inputs be prompted?
- Can the picker call shared app operations without duplicating transition rules?
- Is retaining CLI-only Accord lifecycle simpler in practice?

## Acceptance

- At least two interaction approaches are rendered and compared.
- Current TUI structure and mouse/keyboard behavior are treated as the baseline.
- Recommendation is explicit: implement, revise, or reject.
- If implementation is recommended, produce a separate bounded Task with objective acceptance criteria.
- No production TUI behavior changes in this research Task.
