---
id: task-20
type: task
title: "Explore and implement rule on/off toggles across protocol, context, and TUI"
state: todo
references: ["task-4", "task-12", "task-32"]
tags: ["rules"]
accord:
  status: "ready"
  acceptance: ["Exploration records the chosen on/off representation, existing-rule defaults, CLI/JSON contract, TUI interaction, and context refresh behavior before implementation; unresolved product choices are surfaced for review.", "The normative protocol defines enabled/disabled rule semantics, and implementation persists toggles without deleting rules or losing their IDs, text, category, source, or other supported metadata.", "CLI read and mutation paths expose rule on/off state unambiguously; context consumers can obtain enabled rules without treating disabled rules as active instructions.", "The TUI lets users navigate rules, clearly distinguish on from off without relying solely on color, and toggle the selected rule; saved state remains correct after reload.", "Context-window behavior is verified end to end for the supported integration, including turning a rule back on; any necessary adapter implementation is tracked through an explicit authorized handoff/task.", "Project tests cover defaults, persistence round trips, CLI/JSON exposure, enabled-rule context selection, and TUI toggle behavior; documentation explains usage and refresh limitations."]
  constraints: ["Rule-storage work task-4 and legacy detection task-12 are completed baseline, not pending implementation dependencies.", "Keep disabled rules distinct from inactive legacy embedded rules. Task-32 owns visibility of the latter; do not duplicate or absorb that focused fix.", "Outside the approved papercut cleanup implementation wave: retain as future exploration/implementation, with adapter changes requiring a separate explicit handoff."]
  updatedAt: "2026-09-11T14:24:35Z"
createdAt: "2026-09-06T12:49:04Z"
updatedAt: "2026-09-11T14:24:35Z"
---

## Description

## Goal
Allow users to temporarily turn individual Tandem rules on or off without deleting them, with a durable protocol representation, correct context-window inclusion, and clear TUI navigation and visibility.

## Exploration before implementation
Evaluate the canonical representation (for example a status parameter or enabled key/value), defaults for existing rules, CLI/JSON exposure, and how enabled rules reach agent context windows. Explore a clearly labeled TUI status column or equivalent indicator and keyboard/mouse toggle interactions. Treat these as design options, not settled choices. Clarify whether context changes apply immediately or on refresh/reload, and document the selected behavior before implementation.

## Implementation scope
Specify semantics in normative protocol documentation first, then implement persistence and shared app operations, CLI support, and TUI navigation/display/toggling. Retain disabled rules as editable, discoverable records; distinguish disabled rules from missing or unmigrated rules. Coordinate with the related rule-storage and legacy-rule diagnostic tasks.

Define the context-consumer contract in Tandem-owned documentation. If agent/framework adapter code must change, create an explicit adapter implementation handoff/task rather than modifying adapter implementations under this core task.
