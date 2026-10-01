---
id: task-57
uid: 1a2fb43b-f4e7-4dc0-aaa8-00a925547947
type: task
title: "Show Standard, Research, and Papercuts lanes in tandem web"
priority: "low"
effort: "medium"
blockers: ["task-55"]
references: ["task-55", "task-5"]
tags: ["ui", "taxonomy"]
accord:
  status: "accepted"
  acceptance: ["The web Board shows Standard, Research, and Papercuts lanes derived from `kind`, each with a count; empty lanes collapse to a header.", "A research Task nested under another parent still appears in the Research lane with parent context.", "The layout is reviewed against the Sideshow mockup and checked in a browser."]
  claimedAt: "2026-10-01T23:33:41Z"
  deliveredAt: "2026-10-01T23:54:55Z"
  validation: ["$ just dev-check"]
  summary: "The `tandem web` Board now shows three stacked lanes (Standard, Research, Papercuts), derived from `kind` alone with no tag fallback. Each lane has a count. An empty lane collapses to a header, and a non-empty lane can be collapsed by the reader and stays collapsed across the 3s poll. A research Task or Subtask nested under another parent appears in Research with \"under <id> · <parent title>\". Epics stay in Standard with an EPIC badge and \"+N children\" (which counts children in other lanes). The state columns were dropped; rows carry state badges instead, and the state/priority/text filters are unchanged. Decisions no longer appear on the Board (they have their own tab). The TUI and extensions/pi-tandem were not touched. Review post (published, version 2): http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8 (postId mTNuqHpdDDc; localhost URL http://localhost:8228/session/urf4z8sWgj8). The post carries Mockup A and the implemented lanes stacked at the same width, plus screenshots of the full page, the collapsed empty lane, nested research, a reader-collapsed lane, dark scheme and a 390px viewport, and it states the dropped state columns. Algorant's review of the post is part of acceptance and has not happened yet. A literal side-by-side did not fit legibly in Sideshow's ~800px column, so Mockup A sits above the implementation instead."
  evidence: ["The web Board shows Standard, Research, and Papercuts lanes derived from `kind`, each with a count; empty lanes collapse to a header.: ui.js laneOf() classifies on item.kind only. The browser DOM showed counts 5/3/3. Under priority=high the Papercuts section got data-empty=true with count 0 and rendered as a header with the label 'empty' (screenshot 2-empty-lane in the post). The board API test asserts that a papercut carrying a `research` tag still has kind papercut.", "A research Task nested under another parent still appears in the Research lane with parent context.: task-7 (parent Epic task-1) and task-2-2 (a research Subtask under task-2) both render in Research with 'under task-1 · Explore assignment workflows' and 'under task-2 · Assignment with a milestone'. The papercut task-10 under task-1 also shows its parent context in Papercuts. The board API test asserts parentId and parentRelationship 'epic-task' for a nested research Task.", "The layout is reviewed against the Sideshow mockup and checked in a browser.: Checked in real Chromium; screenshots are in the post at http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8 (post mTNuqHpdDDc), with Mockup A shown above the implemented lanes at the same width. The MagicDNS URL returned HTTP 200. Algorant's review of the post is still outstanding.", "$ just dev-check: Exit 0; 403 passed, 0 failed; dev smoke test printed PASS.", "Commit: All work is committed as 8ddce55 'feat(web): show Standard, Research, and Papercuts lanes on the Board, derived from kind'. The git status was clean afterwards, and no .tandem or runtime state was committed."]
  filesChanged: ["tandem/src/web/ui.js", "tandem/src/web/app.css", "tandem/src/web.rs"]
  updatedAt: "2026-10-01T23:54:57Z"
createdAt: "2026-10-01T22:21:04Z"
updatedAt: "2026-10-01T23:54:57Z"
relatedFiles: ["tandem/src/web.rs", "tandem/src/web/ui.js", "tandem/src/web/app.js", "tandem/src/web/app.css"]
assignee: "worker-task-57-cc6ba535"
archivedAt: "2026-10-01T23:54:57Z"
resolution:
  outcome: "completed"
---

## Description

The lanes idea deliberately lives in the web view rather than the static TUI. Design seed: "Mockup A" in the first post at http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8 (three stacked lanes, counts, collapse, and nested research shown in its lane).
