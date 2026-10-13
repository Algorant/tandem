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
  deliveredAt: "2026-10-13T16:59:31Z"
  validation: ["$ git diff --check", "$ just site-build", "$ cd site && bun run check:links"]
  constraints: ["Plan first; explicit Luna/high. No drawing/capture until parent approves bounded plan.", "Use existing Excalidraw vault/EA PNG/source method; native editable sources beside PNGs, no hand-written scenes or new exporter.", "Public generic data only; only site/public/images assets, no prose/layout/Rust/teaser/product edits.", "Serialize Obsidian/EA commands and protect human edits.", "Herdra exact final D4 layout: two side-by-side machines, full-width shared remote below, exactly two straight vertical bidirectional arrows; three side-by-side local outcomes below with no arrows. D1/D3/captures unchanged.", "Post the corrected set for HERDRA recheck first; prior final post2QgnioNb-rI is superseded. No Algorant handoff until Herdra approves; no integration/acceptance/cleanup until Algorant accepts."]
  summary: "Implemented and independently checked Herdra's exact D4 three-row layout. New review set posted for Herdra first; hold Algorant handoff/integration/acceptance/cleanup pending recheck. Only D4 PNG changed; all accepted D1/D3/captures unchanged. Clean af0a878 retained."
  evidence: ["Current set: http://desktop.tail1cefc.ts.net:8228/session/J0qH-JN7PqI postSPXFbM9ETWU, HTTP200; native D4 and390px preview plus unchanged accepted set. Previous final post2QgnioNb-rI marked superseded.", "Parent read native EA scene: exactly2 arrow elements, each2points with same absolute endpointx180/540, y300→420, start/end arrowhead arrow. Endpoints lie between aligned machine panels and full-width remote top. No per-branch or outcomes arrows; no crossing lines. Both machines have same two-line body and ignored-sourceGit note; remote main/tandem side-by-side; outcomes side-by-side sized for text.", "Parent inspected D4 pixels native/390px; manual conflict text until you resolve it. Native vault/site D4 SHA2564c0c95188f4554d58f72ec44d11e957e76644b5f8233abf779c95c91aba1ad16. All3 editable source/PNG pairs retained/hashmatched. No raw scene writing or new exporter.", "Parent independently ran just site-build, site check:links and git diff --check main...HEAD: passed,996links/21HTML; existing i18n/404 warnings unchanged. Diff38d5c0e..HEAD only D4PNG; accepted D1/D3/8terminal/Chromiumweb byte-unchanged. Clean af0a878.", "Review provenance only: real0.17.0 HerdrANSI terminal renders, real unchanged nativeChromium152 read-onlyweb; rawinputs and native900/1280/1920 webPNGs retained. No publicbanner or injectedCSS/productchange. Teaser untouched.", "Explicit decisions: remains delivered/unintegrated. Herdra rechecks this geometry BEFORE Algorant acceptance; no cleanup/complete/push/release/deploy until authorized. Exact Worker and clean captured checkout retained."]
  filesChanged: ["site/public/images/source-and-tandem-sync.png", "site/public/images/shared-board.png", "site/public/images/task-lifecycle.png", "site/public/images/tui-board.png", "site/public/images/tui-session-chooser.png", "site/public/images/tui-session-activity.png", "site/public/images/tui-full-board-selection.png", "site/public/images/sync-status-pending.png", "site/public/images/sync-status-conflict.png", "site/public/images/sync-status-held.png", "site/public/images/tui-sync-conflict.png", "site/public/images/web-read-only.png"]
  updatedAt: "2026-10-13T16:59:31Z"
createdAt: "2026-10-13T13:46:56Z"
updatedAt: "2026-10-13T16:59:31Z"
assignee: "worker-task-81-c628664a"
---

## Description

Approved by Herdra within Algorant's task-76 restart. Visuals Worker first, then content/layout Worker. Required asset map: D1 shared board (Home), D3 lifecycle (Concepts), D4 sidecar storage/sync (Concepts dedicated section). D4 covers independent source Git and Tandem-managed tandem histories, two generic machines sharing a remote, compatible merges versus a true conflict, and held unpublishable local edits. Short public-reader labels; do not promise zero board conflicts.

Current 0.17.0 captures: Board; existing w chooser, Ctrl-A/Ctrl-D, recorded-assignee activity, local last checked, full-TUI selection; CLI tandem sync status with useful pending/conflict/held states; real conflict surfaced in TUI; current single-workspace loopback read-only web. Include caption/alt-text suggestions only as handoff metadata, not prose file edits. Source-output is real; faithfully rendered ANSI capture may be used if labelled, never hand-synthesized terminal output. Reproduce sync cases on disposable local Git replicas/bare origin; no real project/user boards or archived-record edits.

Herdra expressly approved using existing EA/vault-rendered PNGs for site/Sideshow assets despite the earlier self-check-only skill. Keep editable plugin-created Excalidraw source (its native .excalidraw.md format) alongside each PNG in Agentic/excalidraw via Obsidian APIs; no raw scene-file writing or new export tooling. The skill is /home/algorant/Projects/excalidraw/.pi/skills/excalidraw/SKILL.md. Obsidian must already be running; ask parent if not. API/global result-slot use must be serial, not concurrent with another drawing Worker.

Existing encoded teaser/poster are retained by the content scaffold; do not modify read-only /home/algorant/Projects/fframes/videos/tandem-teaser-v3/out/tandem-teaser-v3-4k.mp4. Asset names/placement are agreed in your plan before creation. Parent will verify images and publish review to J0qH-JN7PqI for Herdra before Algorant sees/integration. No content pages, code or deployment.
