---
id: task-32
type: task
title: "Make legacy-rule warnings readable in the TUI Rules view"
state: todo
priority: "low"
effort: "medium"
references: ["task-12", "task-20"]
relatedFiles: ["tandem/src/tui/rules.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs"]
tags: ["tui", "papercut", "rules"]
accord:
  status: "ready"
  acceptance: ["At 132-column terminal width, a user in the Rules view can read or explicitly expand the full diagnostic naming tandem.md, saying embedded rules are not active, and identifying .tandem/rules/ as the correct location.", "The warning remains discoverable after ordinary view navigation and transient status expiration while the legacy block exists.", "A rendering regression test verifies visible actionable diagnostic content, not only the internal warning string; no automatic migration or activation of legacy rules is introduced."]
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml --check"]
  constraints: ["TUI warning presentation only. No automatic migration, legacy rule activation, rule toggles, or app/protocol/CLI changes.", "Use a persistent visible diagnostic or clearly discoverable expanded warning in Rules view; no broad redesign of all notifications.", "Render-test actual diagnostic content and normal no-warning behavior; use valid native 0.3.0 fixture initialization for ordinary workspace tests.", "Submit implementation design and plan before editing. Parent will inspect a release TUI in a safe fixture and own preview routing."]
  updatedAt: "2026-09-11T14:21:34Z"
createdAt: "2026-09-11T13:35:26Z"
updatedAt: "2026-09-11T14:21:34Z"
---

## Description

Found during Algorant-requested closeout of task-12 on 0.13.1. Algorant explicitly approved closing the shipped detection fix and tracking this presentation limitation separately.

Reproduction: a valid 0.3.0 workspace with a populated legacy rules block in tandem.md and no rules directory. CLI human/JSON correctly says the rules are not active and directs them to .tandem/rules/. The TUI reload also generates the same warning, but in the Rules view at 132x61 the normal footer hints consume most of the row: immediately after r, only an orange 'Reloaded 0 active documents from .tand...' suffix is visible. After the transient status expires, the view shows Rules (0) with no actionable diagnostic.

Existing reload_warns_for_legacy_embedded_rules asserts ReloadOutcome.first_warning, not rendered visibility. Add real rendering coverage. Temporary reproduction script and ANSI captures: /tmp/tandem-closeout.I7vnyI/verify.sh and legacy-reload.ansi (ephemeral evidence; reproduction described here is durable).

Scope is warning presentation only, not migration or compatibility loading. Choose the smallest readable presentation in the Rules view; do not redesign all TUI notifications as part of this papercut.
