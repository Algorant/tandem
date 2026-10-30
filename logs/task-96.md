---
id: task-96
uid: c134ee18-5415-4fb3-bc3f-b75390835d63
type: task
title: "Board exclude-tag filter with a saved default view per workspace"
priority: "medium"
tags: ["tui", "config"]
accord:
  status: "accepted"
  acceptance: ["The TUI Board can hide every task carrying a chosen tag, and the active filter summary shows the exclusion.", "A workspace can save that exclusion as its Board view in `.tandem/config.toml`, and a fresh `tandem tui` in that workspace opens with it applied and can clear it for the session.", "A plan is approved through Herdra before implementation starts."]
  claimedAt: "2026-10-30T13:50:20Z"
  deliveredAt: "2026-10-30T14:07:12Z"
  summary: "TUI-only saved Board view: `[board] hide_tags = [...]` in workspace `.tandem/config.toml` hides tasks with any listed tag (case-insensitive, `#` prefix accepted) when the TUI opens or switches workspace; filter bar shows \"hiding #work, #after-setup\"; `f` → \"Show hidden tags\" (or \"Clear all filters\") clears it for the session and background reloads don't re-impose it. User-level config is ignored with a warning. CLI list/search and web unchanged. Shipped in 0.17.4 (feature 2c938af, release 40294ed, docs-lock refresh 0a35c54)."
  evidence: ["E2E release build in hc terminal on temp Git board (tasks tagged work, WORK+tui, home, after-setup, untagged) with hide_tags=[\"work\"]: bar 'FILTERS hiding #work', tabs 'ALL 3/5', rows only Fix bike/Plan trip/Read book.", "f picker showed 'Show hidden tags — Stop hiding #work for this session'; Enter restored all 5 rows; adding a new #work task via CLI then waiting 4 s showed ALL 6 with it visible (clear stuck across reload).", "Restart with hide_tags = [\"#Work\", \"after-setup\"] # saved view: bar 'hiding #work, #after-setup', ALL 2/6 (Fix bike, Read book); user-level hide_tags=[\"home\"] did not hide #home.", "CLI `tandem list --json` count stayed 5 then 6 throughout.", "just dev-check pass, cargo fmt + strict clippy clean, just site-build pass; just release 0.17.4 verified GitHub Release, assets, notes and AUR workflow 41068819430."]
  filesChanged: ["tandem/src/tui/board/mod.rs", "tandem/src/tui/theme.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/pickers.rs", "docs/reference/configuration.md", "RELEASES.md", "tandem/Cargo.toml", "tandem/Cargo.lock", "site/bun.lock"]
  reviewer: "Algorant"
  updatedAt: "2026-10-30T14:36:57Z"
createdAt: "2026-10-30T13:35:21Z"
updatedAt: "2026-10-30T14:36:57Z"
assignee: "tandem"
archivedAt: "2026-10-30T14:36:57Z"
resolution:
  outcome: "completed"
  reviewer: "Algorant"
---

## Description

Requested by Algorant via Herdra (m73d1218b). He tags work tasks `#work` on a board and wants a Board view that hides them ("everything except #work").

Today `BoardFilters` (tandem/src/tui/board/mod.rs) holds one optional include `tag` plus kind/priority/delivered-untriaged. It cannot exclude a tag, and filters are per-session only; nothing is saved.

Smallest version:
- An exclude-tag Board filter ("not #work"), alongside the existing include filter, shown in the filter summary.
- A way to persist it as the workspace's default (or named) Board view, likely in workspace `.tandem/config.toml` next to `[board.badges]`.

Capture only: plan first and bring the plan to Herdra before building. Open questions for the plan: one excluded tag or a list; default view only or named views plus a picker; whether CLI `list` and the web view should honor the same config or stay TUI-only; how an active saved view is shown and cleared in-session.
