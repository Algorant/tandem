---
id: task-184
type: task
title: "Investigate Pi/Codex premature stop after worker handoffs"
priority: "high"
relatedFiles: ["extensions/pi-herdr/index.ts", "extensions/pi-subagents"]
tags: ["config", "bugfix", "herdr", "validation"]
createdAt: "2026-07-24T13:43:06Z"
updatedAt: "2026-07-24T13:54:17Z"
completedAt: "2026-07-24T13:54:17Z"
completion:
  outcome: "canceled"
  summary: "Canceled: Created in error: user intended a task about the release issue, not premature Pi/Codex continuation behavior."
---

## Description

## Bug

With the Codex GPT-5.6 Terra provider, Pi sometimes returns a normal `stopReason: stop` after announcing that it will continue work, or immediately after a delivered Worker handoff. In one confirmed Worker case, Herdr injected the handoff and explicit review instruction, the agent turn began, then the provider returned an empty final response with no tool calls.

## Scope

- Reproduce and capture minimal session evidence for ordinary continuation and Worker-handoff cases.
- Determine whether the behavior originates in Pi request/event construction, Codex provider response handling, or model behavior.
- Inspect the provider/agent handoff boundary without modifying byte-exact `pi-herdr` unless an explicit lifecycle contract is approved.
- Propose and, only if evidence supports it, implement a narrowly scoped fix with regression coverage.

This is not a Tandem protocol or Worker transport bug; validate that event delivery succeeds before assigning blame.
