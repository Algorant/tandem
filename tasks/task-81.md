---
id: task-81
uid: 800443c6-c745-41db-9216-f70642139032
type: task
title: "Docs visuals: public Excalidraw diagrams and 0.17.0 captures"
state: "in-progress"
priority: "medium"
effort: "medium"
references: ["task-76", "task-73", "task-74"]
relatedFiles: ["site/public/images/"]
tags: ["docs", "ui"]
accord:
  status: "delivered"
  acceptance: ["Public Excalidraw D1 shared-board/how-it-works, D3 Task lifecycle and D4 source-versus-tandem/two-machine/conflict-versus-held diagrams are readable, accurate for protocol 0.5.0 and have editable sources retained in the authorized vault.", "Actual 0.17.0 Board, session chooser/activity/add-remove/full-TUI switch, sync-status pending/conflict/held examples, TUI conflict and single-workspace read-only web captures are produced on safe disposable replicas, without private data or synthetic output.", "Authorized PNG/capture assets under site/public/images render legibly at desktop/mobile widths and on dark/light pages; existing teaser is not rewritten.", "Parent receives exact artifact/source paths, native capture observations and preview surfaces for review in the existing docs Sideshow session; no public-page writing or integration is performed by the Worker."]
  claimedAt: "2026-10-13T13:47:07Z"
  deliveredAt: "2026-10-13T16:40:16Z"
  validation: ["$ git diff --check", "$ just site-build", "$ cd site && bun run check:links"]
  constraints: ["Plan first; explicit Luna/high. No drawing/capture until parent approves the bounded Worker plan.", "Use only existing Excalidraw vault/EA method: PNG renders may be copied to site/public/images and Sideshow, with editable plugin-created source and PNG kept together in the authorized vault folder. No new exporter, hand-written scenes or other diagram tool.", "Diagrams/captures for public readers: no private paths, hostnames or personal/internal names; use generic example repositories and agents.", "Only owned image assets; no page prose, site layout, Rust/CLI behavior or external repository code edits. Original teaser source remains read-only.", "One Obsidian command at a time; do not launch Obsidian if absent. Serialize vault/EA use with other diagram work.", "Deliver visuals to parent for independent verification/Herdra review in J0qH-JN7PqI before integration; no push/deploy. Temporary fixtures only.", "Herdra accepted captures and requested final D4 symmetry/manual-resolution corrections. Final set must be a new post in J0qH-JN7PqI; hold integration/acceptance/cleanup until Algorant accepts."]
  summary: "Final12-asset set posted for Algorant at Herdra direction. Captures/D1/D3 previously accepted and unchanged. D4 now symmetric: both machines connect to main and tandem and feed local When-you-sync band; conflict wording until you resolve it. Parent inspected native/390px image, verified EA vault/source PNG hashes, exact one-file revision, clean f8281fd and native site/996-link checks. Hold integration/acceptance/cleanup for Algorant."
  evidence: ["Final review: http://desktop.tail1cefc.ts.net:8228/session/J0qH-JN7PqI new post2QgnioNb-rI; HTTP200. All12 public assets published; no deployment or integration.", "Parent verified diff38d5c0e..f8281fd touches only source-and-tandem-sync.png. D1/D3 and all Herdra-accepted captures byte-unchanged. Both machine panels have same arrows to main and tandem, each feeds local outcomes band. Conflict text: unpublished until you resolve it; caption names tandem sync resolve.", "Parent read native EA scenes and hashes via authorized Obsidian API; D4 vault/site SHA256 e788f5c8f41fcdea718405dae0faad0800b4386039b7f1f3895a00ee18c7b164, pairs preserved. No human source overwrite, raw hand-written scenes or exporter introduced.", "Parent independently ran just site-build, site check:links and diff-check: passed,996links/21HTML. Native/390px D4 images inspected. Existing i18n/404 warnings remain. No prose/layout/Rust/teaser changes.", "Native Chromium web and authentic renderedANSI terminal captures retained with review-only provenance and no public banners. Source/archive pairs and raw /tmp/tandem-task81-captures evidence retained. Task82 false-positive cancellation permanent; no CSSworkaround.", "Explicit outcome: Task remains delivered, now validation for Algorant visual acceptance; no integration/Tandem completion/checkout cleanup/push/release/deploy before his acceptance. Worker clean at f8281fd retained."]
  filesChanged: ["site/public/images/shared-board.png", "site/public/images/task-lifecycle.png", "site/public/images/source-and-tandem-sync.png", "site/public/images/tui-board.png", "site/public/images/tui-session-chooser.png", "site/public/images/tui-session-activity.png", "site/public/images/tui-full-board-selection.png", "site/public/images/sync-status-pending.png", "site/public/images/sync-status-conflict.png", "site/public/images/sync-status-held.png", "site/public/images/tui-sync-conflict.png", "site/public/images/web-read-only.png"]
  updatedAt: "2026-10-13T16:40:16Z"
createdAt: "2026-10-13T13:46:56Z"
updatedAt: "2026-10-13T16:40:16Z"
assignee: "worker-task-81-c628664a"
---

## Description

Approved by Herdra within Algorant's task-76 restart. Visuals Worker first, then content/layout Worker. Required asset map: D1 shared board (Home), D3 lifecycle (Concepts), D4 sidecar storage/sync (Concepts dedicated section). D4 covers independent source Git and Tandem-managed tandem histories, two generic machines sharing a remote, compatible merges versus a true conflict, and held unpublishable local edits. Short public-reader labels; do not promise zero board conflicts.

Current 0.17.0 captures: Board; existing w chooser, Ctrl-A/Ctrl-D, recorded-assignee activity, local last checked, full-TUI selection; CLI tandem sync status with useful pending/conflict/held states; real conflict surfaced in TUI; current single-workspace loopback read-only web. Include caption/alt-text suggestions only as handoff metadata, not prose file edits. Source-output is real; faithfully rendered ANSI capture may be used if labelled, never hand-synthesized terminal output. Reproduce sync cases on disposable local Git replicas/bare origin; no real project/user boards or archived-record edits.

Herdra expressly approved using existing EA/vault-rendered PNGs for site/Sideshow assets despite the earlier self-check-only skill. Keep editable plugin-created Excalidraw source (its native .excalidraw.md format) alongside each PNG in Agentic/excalidraw via Obsidian APIs; no raw scene-file writing or new export tooling. The skill is /home/algorant/Projects/excalidraw/.pi/skills/excalidraw/SKILL.md. Obsidian must already be running; ask parent if not. API/global result-slot use must be serial, not concurrent with another drawing Worker.

Existing encoded teaser/poster are retained by the content scaffold; do not modify read-only /home/algorant/Projects/fframes/videos/tandem-teaser-v3/out/tandem-teaser-v3-4k.mp4. Asset names/placement are agreed in your plan before creation. Parent will verify images and publish review to J0qH-JN7PqI for Herdra before Algorant sees/integration. No content pages, code or deployment.
