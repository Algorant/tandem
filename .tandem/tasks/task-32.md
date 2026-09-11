---
id: task-32
type: task
title: "Make legacy-rule warnings readable in the TUI Rules view"
state: "in-progress"
priority: "low"
effort: "medium"
references: ["task-12", "task-20"]
relatedFiles: ["tandem/src/tui/rules.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs"]
tags: ["tui", "papercut", "rules"]
accord:
  status: "delivered"
  acceptance: ["At 132-column terminal width, a user in the Rules view can read or explicitly expand the full diagnostic naming tandem.md, saying embedded rules are not active, and identifying .tandem/rules/ as the correct location.", "The warning remains discoverable after ordinary view navigation and transient status expiration while the legacy block exists.", "A rendering regression test verifies visible actionable diagnostic content, not only the internal warning string; no automatic migration or activation of legacy rules is introduced."]
  claimedAt: "2026-09-11T14:22:25Z"
  deliveredAt: "2026-09-11T14:30:37Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml --check"]
  constraints: ["TUI warning presentation only. No automatic migration, legacy rule activation, rule toggles, or app/protocol/CLI changes.", "Use a persistent visible diagnostic or clearly discoverable expanded warning in Rules view; no broad redesign of all notifications.", "Render-test actual diagnostic content and normal no-warning behavior; use valid native 0.3.0 fixture initialization for ordinary workspace tests.", "Submit implementation design and plan before editing. Parent will inspect a release TUI in a safe fixture and own preview routing."]
  summary: "Delivered the Rules-view legacy rule diagnostic presentation. reload() now captures app::project::warnings() into a new TuiApp.rules_warnings field (recomputed on every reload, so it clears when the legacy block is removed) while still extending load_errors, preserving existing ReloadOutcome/status behavior. draw_rules_view renders a persistent warning block between the category tabs and the rule list: each message is word-wrapped with a bold warning-tone `! ` marker and 2-space hanging indent, and the block uses a Length constraint so the full actionable text is never silently truncated; the list takes remaining rows (Min(0)). The no-warning path still uses the original [Length(1), Min(3)] layout. No app/protocol/CLI changes, no migration or rule activation. At the supported minimum 45x12 the full diagnostic is still visible and navigation does not panic; the tradeoff is the rule list pane is squeezed to 2 border-only rows so the message stays complete. Six new render tests in tandem/src/tui/mod.rs cover 132x61 readability, 60-column wrapping, 45x12 minimum size with Board/Rules navigation, persistence after view navigation and transient status expiry, clean no-warning layout, and clearing the banner after the legacy block is removed. Parent-owned release-TUI ANSI inspection remains outstanding; I did not touch the preview route or the /tmp/tandem-closeout.I7vnyI/legacy fixture."
  evidence: ["At 132-column width a user in Rules view can read the full diagnostic naming tandem.md, saying embedded rules are not active, and identifying .tandem/rules/.: rules_view_renders_legacy_rules_diagnostic_at_132_columns clears status then draws 132x61; normalized buffer text contains 'tandem.md', 'not active', and '.tandem/rules/'. Banner message renders as '! Legacy embedded rules found in the `rules:` block in tandem.md; those rules are not active. Move them to individual files under `.tandem/rules/`.'", "The warning remains discoverable after ordinary view navigation and transient status expiration while the legacy block exists.: rules_view_legacy_rules_diagnostic_persists_after_status_expiry navigates Rules->Board->Rules, forces expire_transient_status() true with empty app.status, then redraws 132x61 and all three tokens are present; the banner is derived from reload state, not transient status.", "A rendering regression test verifies visible actionable diagnostic content, not only the internal warning string.: Six new tests assert real TestBackend-rendered buffer content (normalized rows), including narrow-width wrapping and the 45x12 minimum size; the existing reload_warns_for_legacy_embedded_rules internal-string test was left intact.", "No automatic migration or activation of legacy rules is introduced.: The 132-column test asserts app.workspace.rules_dir() does not exist after load and all app.rules categories are empty; no filesystem writes occur in the presentation path. rules_warning behavior only reads app::project::warnings().", "No stale state: removing the legacy block removes the banner.: rules_view_legacy_rules_diagnostic_clears_when_block_is_removed overwrites tandem.md with a native default_project_config, calls reload(), asserts rules_warnings is empty, and confirms a redraw contains no '.tandem/rules/'.", "Normal no-warning layout unchanged.: draw_rules_view keeps the original [Constraint::Length(1), Constraint::Min(3)] layout when rules_warnings is empty; rules_view_without_legacy_block_has_no_diagnostic_banner renders the normal 'No always rules defined' list with no banner tokens.", "Supported minimum-size render/no-panic/navigation.: rules_view_legacy_rules_diagnostic_renders_at_minimum_terminal_size draws 45x12, navigates Rules->Board->Rules, and asserts the full diagnostic tokens after redraw with no panic."]
  filesChanged: ["tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/rules.rs"]
  updatedAt: "2026-09-11T14:30:37Z"
createdAt: "2026-09-11T13:35:26Z"
updatedAt: "2026-09-11T14:30:37Z"
assignee: "worker-task-32-7222a706"
---

## Description

Found during Algorant-requested closeout of task-12 on 0.13.1. Algorant explicitly approved closing the shipped detection fix and tracking this presentation limitation separately.

Reproduction: a valid 0.3.0 workspace with a populated legacy rules block in tandem.md and no rules directory. CLI human/JSON correctly says the rules are not active and directs them to .tandem/rules/. The TUI reload also generates the same warning, but in the Rules view at 132x61 the normal footer hints consume most of the row: immediately after r, only an orange 'Reloaded 0 active documents from .tand...' suffix is visible. After the transient status expires, the view shows Rules (0) with no actionable diagnostic.

Existing reload_warns_for_legacy_embedded_rules asserts ReloadOutcome.first_warning, not rendered visibility. Add real rendering coverage. Temporary reproduction script and ANSI captures: /tmp/tandem-closeout.I7vnyI/verify.sh and legacy-reload.ansi (ephemeral evidence; reproduction described here is durable).

Scope is warning presentation only, not migration or compatibility loading. Choose the smallest readable presentation in the Rules view; do not redesign all TUI notifications as part of this papercut.
