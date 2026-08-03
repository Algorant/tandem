---
id: task-194
type: task
title: "Define universal agent-adapter guidance and produce implementation handoffs"
state: "in-progress"
priority: "medium"
relatedFiles: ["extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/tests/relationship-smoke.ts", "protocol/plan/spec.md", "docs/concepts/index.md", "extensions/plan/spec.md"]
tags: ["pi-tandem", "guidance", "docs", "protocol", "agents", "adapters"]
createdAt: "2026-08-03T15:13:05Z"
updatedAt: "2026-08-03T20:19:01Z"
accord:
  status: "claimed"
  assignee: "worker-task-194-2a74c93a"
  claimedAt: "2026-08-03T20:19:01Z"
  updatedAt: "2026-08-03T20:19:01Z"
assignee: "worker-task-194-2a74c93a"
---

## Description

Define framework-neutral guidance for agents and adapters that consume Tandem. Keep Tandem core documentation authoritative for universal behavior, while preserving a strict implementation boundary around all adapter code.

Scope:
- Inventory agent-facing behavior currently implied by protocol documentation, active Tandem rules, CLI behavior, and existing adapter-facing material.
- Classify each behavior as universal Tandem semantics, repository-specific policy, adapter implementation detail, duplicate/obvious, obsolete, or unsupported.
- Define and document the operational meaning of each rule category (`always`, `never`, `prefer`, and `context`) so agents and adapters classify directives by semantics rather than wording alone.
- Include classification guidance and examples for mixed directives. A prohibition such as “do not modify adapter code” belongs in `never`; its positive follow-up behavior can remain in the same rule or be separated when that improves enforcement.
- Document only universal, framework-neutral behavior in Tandem-owned protocol or guidance documents outside adapter directories.
- Explain how any agent or framework should discover a workspace, inspect rules, interpret rule categories, use lifecycle operations, respect authority boundaries, and obtain context without assuming Pi-specific tools or prompts.
- Keep repository-specific policy in active `.tandem` rules rather than presenting it as universal adapter behavior.
- Produce explicit handoff documents for changes that should later be implemented in `pi-tandem` or another adapter. Each handoff must identify the generic requirement, rationale, acceptance behavior, and adapter-owned files likely affected.
- Do not modify `extensions/pi-tandem/`, external Pi configuration, or any other adapter implementation in this task.

Required design outcome:
- Tandem-owned protocol or guidance documents are the authority for universal agent and adapter behavior.
- Active `.tandem` rules remain the authority for repository-specific policy.
- Rule categories have clear behavioral semantics that generic adapters can preserve without framework-specific interpretation.
- Adapter repositories consume the generic contract and own framework-specific prompts, tools, rendering, and integration code.
- Adapter implementation changes occur only through a later explicit adapter task based on a Tandem-authored handoff.

Acceptance evidence:
- A concise disposition table identifies universal guidance, repository policy, and adapter-specific concerns without assuming existing adapter text is correct.
- Guidance includes category-selection examples and correctly classifies prohibitions under `never`.
- Generic guidance does not name Pi tools except in clearly labeled examples or handoffs.
- One or more implementation handoff documents capture necessary future adapter changes without editing adapter code.
- Repository-specific commit-frequency policy remains in active Tandem rules and is not promoted to universal guidance without independent justification.
- Relevant documentation checks and `git diff --check` pass.
