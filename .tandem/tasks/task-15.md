---
id: task-15
type: task
title: "Make durable Task and milestone workflow usable through the TUI"
state: todo
priority: "medium"
references: ["task-6"]
relatedFiles: ["tandem/src/tui/input.rs", "tandem/src/tui/board/mod.rs", "tandem/src/tui/validation.rs"]
tags: ["tui", "accord"]
accord:
  status: "ready"
  acceptance: ["Version-matched audit identifies actual missing durable actions/context and reuses the native app layer, with task-6 relationship reconciled explicitly.", "TUI shows current acceptance, blockers, milestone progress and final delivery evidence without Pi/session scraping or duplicated inline checklist state.", "Task claim/deliver/block/resume and ordinary completion are usable via a minimal valid-action interface; exceptional human review remains distinct, and invalid transitions fail clearly.", "Action/render tests exercise state changes and resulting native records; no agent-launch controls or broad layout redesign are included."]
  validation: ["Run repository-prescribed native/TUI tests and perform bounded actual TUI inspection using existing rendering guidance; report what remains unverifiable."]
  constraints: ["TUI modules/tests only; native app/API additions must be coordinated with task-13, not overlap silently.", "Keep scope to durable coordination parity and concise presentation. No requirement to preserve obsolete interaction conventions.", "Source delivery is not automatic installation/release; return handoff to Pi task-59."]
  updatedAt: "2026-09-05T21:04:08Z"
createdAt: "2026-09-05T21:04:08Z"
updatedAt: "2026-09-05T21:04:08Z"
---

## Description

Implementation proposal from Pi task-59. User requires native CLI/TUI operation without Pi, with milestones/blockers/evidence as the normal view. Existing task-6 researches contextual lifecycle actions; review/reconcile its useful findings, do not duplicate research or assume its old conventions are mandatory. No production changes are authorized merely by task creation.

Verify actual version-matched TUI gaps first. Expose complete acceptance, dependencies, existing milestone status and final evidence; offer valid durable lifecycle actions through one simple interaction invoking existing native app operations. Routine completion need not route through human Validation. Do not add Worker runtime controls, another state machine, an alternative checklist store or a broad visual redesign. User has separately chosen native Herdr/Worktrunk for runtime/Git controls. Core read/app changes belong to task-13; request exact missing API rather than independently inventing it.

Validation must drive actual actions and inspect resulting records; static appearance alone is insufficient. Follow repository guidance to verify renderable TUI facts before requesting taste/temporal human review.
