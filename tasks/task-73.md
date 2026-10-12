---
id: task-73
uid: f1f67c9a-042d-4bd2-9e53-11e9579a634b
type: task
title: "Orchestrator mode: opt-in multi-workspace web selector"
state: todo
priority: "medium"
effort: "medium"
blockers: ["task-72"]
references: ["task-70"]
relatedFiles: ["tandem/src/web.rs", "tandem/src/web/app.js", "tandem/src/web/api.js", "tandem/src/web/index.html"]
tags: ["ui", "orchestrator"]
accord:
  status: "ready"
  acceptance: ["Without the opt-in flag tandem web retains its current single-workspace behaviour; with it a selector switches among explicitly configured boards.", "Per-board URLs preserve selected workspace identity and unknown aliases return 404; client input cannot open an unlisted filesystem path.", "Read-only methods, loopback binding, Host checks and CSP remain enforced across every workspace route.", "The served list is visible at startup and background sync does not unconditionally run for every configured board."]
  validation: ["just dev-check"]
  updatedAt: "2026-10-12T15:51:22Z"
createdAt: "2026-10-12T15:51:22Z"
updatedAt: "2026-10-12T16:16:24Z"
---

## Description

Approved task-70 follow-up. Keep ordinary tandem web single-workspace. Add an explicit opt-in flag for serving the hand-written workspace list, with a header selector and per-board URLs (/w/<alias>/...). Resolve only configured aliases; never accept arbitrary filesystem paths from a request. Preserve read-only, loopback binding, exact Host validation and CSP. Print the served list at startup; define a bounded active-workspace sync policy rather than fetching every listed board continuously. No all-workspaces overview in this slice. Kickoff approved by Algorant via Herdra: implement in order 71 → 75. Visual Tasks remain delivered for Algorant's review, with actual screenshots in Sideshow. Keep docs overhaul task-76 paused.
