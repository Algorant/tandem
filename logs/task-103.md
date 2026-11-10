---
id: task-103
uid: 3d65151f-efba-4842-83bb-f39ad0b9f5b3
type: task
title: "Make tandem list cheap and complete for display callers"
priority: "medium"
effort: "small"
tags: ["cli", "sync"]
accord:
  status: "accepted"
  acceptance: ["`tandem list --no-sync` performs no board sync: a stale-board read spawns zero Git processes and returns the same records as today.", "`tandem list --json` items include state, kind, role, type, priority, effort, assignee, parentId/relationship, tags, accordStatus and location in one call, keeping id/title unchanged.", "Default `tandem list` (and other read commands) keep their current refresh behavior; docs describe the display-caller shape."]
  claimedAt: "2026-11-10T15:48:15Z"
  deliveredAt: "2026-11-10T16:00:16Z"
  summary: "Added `tandem list --no-sync` (local-only read) and richer one-call list JSON for display callers, documented, verified end-to-end on a real Git board, and released as0.17.6."
  evidence: ["Disposable real Git board with local bare remote, forced stale sync state: `list --no-sync --json` spawned zero Git children and returned the same 3 records; default `list --json` still refreshed (26 Git children including fetch) and returned identical data.", "JSON items carry id/type/kind/title/location/state/priority/effort/assignee/parentId/tags/accordStatus/updatedAt; fields matched `show --json` for the same records; id/title unchanged.", "Docs updated in docs/cli/index.md; cargo fmt, strict clippy, dev-check (12 suites) and site build passed. No new tests, mocks or real-board writes; verification sandbox removed.", "Released tandem-v0.17.6: GitHub Release non-draft with13 verified assets/checksums, live installer smoke-matched; desktop and homelab installed0.17.6 with binary SHA256 52192b8c…; x1nano offline, tracked by standing needs-x1nano task-105. AUR job failure is a separate Arch openssl4.1 packaging issue (papercut task-104)."]
  updatedAt: "2026-11-10T16:00:17Z"
createdAt: "2026-11-10T15:48:03Z"
updatedAt: "2026-11-10T16:00:17Z"
assignee: "tandem"
archivedAt: "2026-11-10T16:00:17Z"
resolution:
  outcome: "completed"
  note: "0.17.6 released and verified on desktop/homelab; Herdra's mod can use list --no-sync --json. x1nano tracked separately."
---

## Description

Requested by Herdra maee5dcd1 while fixing Herdr window idle load (herdra task-112; Algorant's goal of machines that basically almost idle). Herdra's window reads 12 boards once per minute; each `tandem list` runs the board refresh when the last fetch is older than READ_FRESHNESS (60s), so a window refresh costs 23 tandem processes plus ~59 git children.

Measured here on 0.17.5 in a disposable local-bare-remote board: `list` with a stale sync state spawned 26–28 git descendants per call; a fresh call spawned 0, and each call returned the same 3 records. `list --json` returns only id and title, so callers make several calls per board to get state/accord/tags.

Plan (plan-first, awaiting Herdra approval):
1. `--no-sync` on `tandem list`: skip `refresh_for_read` entirely; local files only, zero Git. Display callers poll at their own cadence; board owners (TUI already syncs 2s/30s) remain the sync source. No new interval framework.
2. `list --json` items become the existing DocumentSummaryDto projection used by `show --json` (id, type, kind, role, title, location, state, priority, effort, assignee, parentId, parentRelationship, tags, accordStatus). Additive: id/title unchanged. Reuse `app::queries::load_read` + `dto::summary`; no second shape.
3. Docs: one line on `--no-sync` and the richer list JSON. No protocol change; patch release 0.17.6 after approval.
Out of scope: changing default refresh behavior, multi-board calls, a cache/daemon, or an interval-sync framework.
