---
id: decision-11
uid: 66896990-8580-4e09-8286-d8a04c9c11a7
type: decision
title: "Keep blockers workspace-local; record external dependencies as prose and blocked Accords"
status: "accepted"
references: ["task-69", "task-70"]
tags: ["protocol", "accord"]
createdAt: "2026-10-12T16:17:51Z"
updatedAt: "2026-10-12T16:18:02Z"
decidedAt: "2026-10-12T16:18:02Z"
---

## Status

Accepted. Algorant delegated the choice in task-69 via Herdra; this records the chosen existing path rather than a new blocker schema.

## Context

Tasks may wait on work in another repository/workspace or a non-record condition. Cross-workspace IDs and free text cannot currently satisfy workspace-local blocker validation. The orchestrator workspace list is machine-local and must not become a source of protocol identity or cross-board resolution.

## Decision

Keep blockers as IDs resolving within the same workspace (Board or Logs). Do not add free-text or cross-workspace blocker forms. Describe external dependencies in the Task body and optionally attach an absolute http(s) URL as a loose reference. Mark the work with accord block --note, normally after claim, and explicitly resume when the external condition is satisfied. Keep the durable explanation in the body because resume clears the block note. References are not prerequisites, and Tandem does not fetch or monitor external conditions.

A blocked Accord does not change workflow state, blockers, assignment readiness or completion eligibility. Orchestrators must inspect Accord status and the recorded condition rather than treating derived assignment readiness alone as permission to proceed. External completion does not automatically resume the Task.

## Consequences

No protocol-version change, cross-board lookup, auto-unblock service or arbitrary-path resolution. External dependency handling remains explicit and manual; protocol and user guidance must state this distinction clearly.
