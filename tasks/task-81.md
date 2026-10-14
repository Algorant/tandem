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
  acceptance: ["One polished public Excalidraw workflow diagram replaces D1 and D3: Agents via CLI and People via TUI enter one Board, delegate/deliver work, agent test/review/validate or human visual verify/feedback, then approve to Logs or rework looping to deliver again. Calm layout, no crossing lines, state names secondary at most; existing accepted D4 remains unchanged. Editable native source/PNG pairs retained.", "Actual 0.17.0 Board, session chooser/activity/add-remove/full-TUI switch, sync-status pending/conflict/held examples, TUI conflict and single-workspace read-only web captures are produced on safe disposable replicas, without private data or synthetic output.", "Authorized PNG/capture assets under site/public/images render legibly at desktop/mobile widths and on dark/light pages; existing teaser is not rewritten.", "Parent receives exact artifact/source paths, native capture observations and preview surfaces for review in the existing docs Sideshow session; no public-page writing or integration is performed by the Worker."]
  claimedAt: "2026-10-13T13:47:07Z"
  deliveredAt: "2026-10-14T02:00:23Z"
  validation: ["$ git diff --check", "$ just site-build", "$ cd site && bun run check:links"]
  constraints: ["Plan first; explicit Luna/high. No drawing/capture until parent approves bounded plan.", "Use existing Excalidraw vault/EA PNG/source method; native editable sources beside PNGs, no hand-written scenes or new exporter.", "Public generic data only; only site/public/images assets, no prose/layout/Rust/teaser/product edits.", "Serialize Obsidian/EA commands and protect human edits.", "Herdra exact final D4 layout: two side-by-side machines, full-width shared remote below, exactly two straight vertical bidirectional arrows; three side-by-side local outcomes below with no arrows. D1/D3/captures unchanged.", "Post the corrected set for HERDRA recheck first; prior final post2QgnioNb-rI is superseded. No Algorant handoff until Herdra approves; no integration/acceptance/cleanup until Algorant accepts."]
  summary: "Combined Work and Review diagram replaces rejected D1+D3 per Algorant structure, published for Herdra FIRST. D4 and all9 accepted captures byte-unchanged. New native source/PNG pair and prior drafts preserved; exact3-file revision, native checks and protectedhashes verified. Clean11d5165 held, not integrated."
  evidence: ["New replacement: http://desktop.tail1cefc.ts.net:8228/session/J0qH-JN7PqI postX will be read from publish output; HTTP200. Herdra checks combined diagram before Algorant, no integration.", "Parent inspected full/390px PNG and native EA geometry: People/TUI and Agents/CLI converge sharedBoard, delegation then delivery,2optionalchecking cards plusdirect asneededdecision path, approveLogs/sendbackRework. Single return polyline x680 outside rightmostcardsx620; no crossing lines.", "Parent verified work-and-review.png vault/site SHA25617f8547da11c10b8d5500c0e166aa5c2dfcd3fa15b8197a27d8952db0595cdab; native Work and Review.excalidraw.md/PNG retained. Old SharedBoard/TaskLifecycle sources+PNGs unchanged (prior recorded hashes match); no source overwrite.", "Parent independently ran protectedhashcheckD4+9captures allOK, gitdiffcheck,sitebuild,996linkcheck/21HTML allpassed. Exact revision af0a878→11d5165 addswork-and-review.png andremovesonlyownedshared-board/task-lifecycle PNGs; no prose/layout/Rust/teaser changes.", "All accepted captures retain authentic0.17ANSI/nativeChromium provenance inreviewnotesonly, no publicbanners/injectedstyles. Current delivered boundary is11publicPNGs (two diagrams+ninecaptures), notold12.", "Explicit outcome: held clean at11d5165 forHerdra thenAlgorant acceptance; no worker_finish/integration/cleanup/push/deploy/release."]
  filesChanged: ["site/public/images/work-and-review.png", "site/public/images/source-and-tandem-sync.png", "site/public/images/tui-board.png", "site/public/images/tui-session-chooser.png", "site/public/images/tui-session-activity.png", "site/public/images/tui-full-board-selection.png", "site/public/images/sync-status-pending.png", "site/public/images/sync-status-conflict.png", "site/public/images/sync-status-held.png", "site/public/images/tui-sync-conflict.png", "site/public/images/web-read-only.png"]
  updatedAt: "2026-10-14T02:00:23Z"
createdAt: "2026-10-13T13:46:56Z"
updatedAt: "2026-10-14T02:00:23Z"
assignee: "worker-task-81-c628664a"
---
## Latest direction — Algorant via Herdra (supersedes D1/D3 asset map)

Replace D1+ D3 with ONE polished workflow diagram following Algorant's structure: Agents → CLI → one Board and People → TUI → same Board; delegate/deliver work; agent can test/review/validate while human can visually verify/give feedback; approve → Logs OR send back to rework looping to delivery. Clean calm no-crossing layout, main labels describe work not Accord states, palette consistent with accepted D4. Same authorized native EA/vault methods; preserve old editable sources rather than delete/overwrite human source. D4 af0a878 and all accepted captures stand byte-unchanged. Worker must return a concrete new diagram layout/arrow-route/file plan before edits; no extra diagram/tool/graph machinery. Parent/Herdra review first before Algorant. No integration/cleanup/deploy pending Algorant acceptance.

## Prior scope (D1/D3 map superseded above)

## Description

Approved by Herdra within Algorant's task-76 restart. Visuals Worker first, then content/layout Worker. Required asset map: D1 shared board (Home), D3 lifecycle (Concepts), D4 sidecar storage/sync (Concepts dedicated section). D4 covers independent source Git and Tandem-managed tandem histories, two generic machines sharing a remote, compatible merges versus a true conflict, and held unpublishable local edits. Short public-reader labels; do not promise zero board conflicts.

Current 0.17.0 captures: Board; existing w chooser, Ctrl-A/Ctrl-D, recorded-assignee activity, local last checked, full-TUI selection; CLI tandem sync status with useful pending/conflict/held states; real conflict surfaced in TUI; current single-workspace loopback read-only web. Include caption/alt-text suggestions only as handoff metadata, not prose file edits. Source-output is real; faithfully rendered ANSI capture may be used if labelled, never hand-synthesized terminal output. Reproduce sync cases on disposable local Git replicas/bare origin; no real project/user boards or archived-record edits.

Herdra expressly approved using existing EA/vault-rendered PNGs for site/Sideshow assets despite the earlier self-check-only skill. Keep editable plugin-created Excalidraw source (its native .excalidraw.md format) alongside each PNG in Agentic/excalidraw via Obsidian APIs; no raw scene-file writing or new export tooling. The skill is /home/algorant/Projects/excalidraw/.pi/skills/excalidraw/SKILL.md. Obsidian must already be running; ask parent if not. API/global result-slot use must be serial, not concurrent with another drawing Worker.

Existing encoded teaser/poster are retained by the content scaffold; do not modify read-only /home/algorant/Projects/fframes/videos/tandem-teaser-v3/out/tandem-teaser-v3-4k.mp4. Asset names/placement are agreed in your plan before creation. Parent will verify images and publish review to J0qH-JN7PqI for Herdra before Algorant sees/integration. No content pages, code or deployment.
