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
  status: "claimed"
  acceptance: ["Public Excalidraw D1 shared-board/how-it-works, D3 Task lifecycle and D4 source-versus-tandem/two-machine/conflict-versus-held diagrams are readable, accurate for protocol 0.5.0 and have editable sources retained in the authorized vault.", "Actual 0.17.0 Board, session chooser/activity/add-remove/full-TUI switch, sync-status pending/conflict/held examples, TUI conflict and single-workspace read-only web captures are produced on safe disposable replicas, without private data or synthetic output.", "Authorized PNG/capture assets under site/public/images render legibly at desktop/mobile widths and on dark/light pages; existing teaser is not rewritten.", "Parent receives exact artifact/source paths, native capture observations and preview surfaces for review in the existing docs Sideshow session; no public-page writing or integration is performed by the Worker."]
  claimedAt: "2026-10-13T13:47:07Z"
  validation: ["$ git diff --check", "$ just site-build", "$ cd site && bun run check:links"]
  constraints: ["Plan first; explicit Luna/high. No drawing/capture until parent approves the bounded Worker plan.", "Use only existing Excalidraw vault/EA method: PNG renders may be copied to site/public/images and Sideshow, with editable plugin-created source and PNG kept together in the authorized vault folder. No new exporter, hand-written scenes or other diagram tool.", "Diagrams/captures for public readers: no private paths, hostnames or personal/internal names; use generic example repositories and agents.", "Only owned image assets; no page prose, site layout, Rust/CLI behavior or external repository code edits. Original teaser source remains read-only.", "One Obsidian command at a time; do not launch Obsidian if absent. Serialize vault/EA use with other diagram work.", "Deliver visuals to parent for independent verification/Herdra review in J0qH-JN7PqI before integration; no push/deploy. Temporary fixtures only."]
  updatedAt: "2026-10-13T13:47:07Z"
createdAt: "2026-10-13T13:46:56Z"
updatedAt: "2026-10-13T13:47:07Z"
assignee: "worker-task-81-c628664a"
---

## Description

Approved by Herdra within Algorant's task-76 restart. Visuals Worker first, then content/layout Worker. Required asset map: D1 shared board (Home), D3 lifecycle (Concepts), D4 sidecar storage/sync (Concepts dedicated section). D4 covers independent source Git and Tandem-managed tandem histories, two generic machines sharing a remote, compatible merges versus a true conflict, and held unpublishable local edits. Short public-reader labels; do not promise zero board conflicts.

Current 0.17.0 captures: Board; existing w chooser, Ctrl-A/Ctrl-D, recorded-assignee activity, local last checked, full-TUI selection; CLI tandem sync status with useful pending/conflict/held states; real conflict surfaced in TUI; current single-workspace loopback read-only web. Include caption/alt-text suggestions only as handoff metadata, not prose file edits. Source-output is real; faithfully rendered ANSI capture may be used if labelled, never hand-synthesized terminal output. Reproduce sync cases on disposable local Git replicas/bare origin; no real project/user boards or archived-record edits.

Herdra expressly approved using existing EA/vault-rendered PNGs for site/Sideshow assets despite the earlier self-check-only skill. Keep editable plugin-created Excalidraw source (its native .excalidraw.md format) alongside each PNG in Agentic/excalidraw via Obsidian APIs; no raw scene-file writing or new export tooling. The skill is /home/algorant/Projects/excalidraw/.pi/skills/excalidraw/SKILL.md. Obsidian must already be running; ask parent if not. API/global result-slot use must be serial, not concurrent with another drawing Worker.

Existing encoded teaser/poster are retained by the content scaffold; do not modify read-only /home/algorant/Projects/fframes/videos/tandem-teaser-v3/out/tandem-teaser-v3-4k.mp4. Asset names/placement are agreed in your plan before creation. Parent will verify images and publish review to J0qH-JN7PqI for Herdra before Algorant sees/integration. No content pages, code or deployment.
