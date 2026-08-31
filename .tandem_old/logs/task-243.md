---
id: task-243
type: task
title: "Resolve the conflict between rules always-11 and never-4"
priority: "medium"
effort: "small"
references: ["decision-11"]
tags: ["rules", "review", "validation"]
createdAt: "2026-08-25T12:50:28Z"
updatedAt: "2026-08-28T12:39:21Z"
completedAt: "2026-08-28T12:39:21Z"
completion:
  summary: "Replaced the contradictory review rules with a single verification test. Added always-12: request human review only when you cannot verify the acceptance criteria yourself, exhaust verification first (run the code, read output, read a Herdr pane including ANSI color), state what was run and what stayed unresolved when escalating, and use ask_me when unsure instead of parking the task. Phrased as requesting review rather than moving to validation so it sits correctly under decision-11. Deleted always-11 and never-4, which gave opposite instructions for the same situation; never-4 won under uncertainty, making always-11 decorative. Also deleted always-10, which was preview plumbing rather than judgment, and moved its Git-local preview-slot guidance into a new AGENTS.md \"Validating TUI changes\" section alongside the verifiable/not-verifiable boundary. Retired the AGENTS.md campaign practice requiring human terminal validation for every visible TUI change, which restated the deleted never-4. Rejected two earlier proposals during review: a blast-radius rule (agents already handle reversibility) and a \"never decide taste\" rule (unfalsifiable in practice; the escalation-evidence clause in always-12 covers the real risk of not noticing a taste question exists). Left context-2 in place pending a separate decision; it still grants completion authority on the \"non-visual, non-manual\" axis that always-12 replaces."
---

## Description

## Problem

Two project rules contradict each other:

- `always-11` permits an agent to inspect TUI output directly.
- `never-4` forbids completing visual work without human review.

An agent that inspects a TUI change directly, confirms it is correct, and then must still park it for human review is following both rules and neither. In practice the contradiction resolves toward `never-4`, because parking is the safer reading under uncertainty. That makes `always-11` decorative.

## Evidence

In the 2026-08-24 session an orchestrator verified a TUI change itself by spawning a Herdr pane and reading the rendered output with `herdr pane read --format ansi`, including colors. The human confirmed that was correct and sufficient. `never-4` would have forbidden completing it.

## Direction

The category "visual work" is the wrong axis. The question is whether the agent can verify the acceptance criteria itself. Sometimes visual work is fully verifiable from a pane read. Sometimes it is not, and then a human is genuinely required. Taste, product direction, and ambiguity about what correct looks like are always human.

The downstream Pi configuration has already adopted this framing in its `pi-tandem` and `pi-agency` skills:

> An orchestrator escalates to human review only when it cannot verify the acceptance criteria itself. Before escalating, exhaust the tools available: run the code, read the output, spawn a Herdr pane, tab, or workspace and read the rendered result including colors. A TUI change is verifiable this way and is not automatically human work. Escalate when verification is genuinely out of reach, or when the question is taste, product direction, or ambiguity about what correct looks like.

Reuse or reject that framing as you see fit; it is offered as a starting point, not a constraint.

## Scope

Amend, merge, or delete `always-11` and `never-4` so one coherent rule remains. Neither is authoritative or permanent.

## Context

Raised while aligning Pi's Tandem guidance with `decision-11` and the 0.11.0 review workflow. That work is complete and does not depend on this. Filed here because the rules live in this workspace; no external tracking is attached.

