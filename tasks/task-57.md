---
id: task-57
uid: 1a2fb43b-f4e7-4dc0-aaa8-00a925547947
type: task
title: "Show Standard, Research, and Papercuts lanes in tandem web"
state: todo
priority: "low"
effort: "medium"
blockers: ["task-55"]
references: ["task-55", "task-5"]
tags: ["ui", "taxonomy"]
accord:
  status: "ready"
  acceptance: ["The web Board shows Standard, Research, and Papercuts lanes derived from `kind`, each with a count; empty lanes collapse to a header.", "A research Task nested under another parent still appears in the Research lane with parent context.", "The layout is reviewed against the Sideshow mockup and checked in a browser."]
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-01T23:33:29Z"
createdAt: "2026-10-01T22:21:04Z"
updatedAt: "2026-10-01T23:33:29Z"
relatedFiles: ["tandem/src/web.rs", "tandem/src/web/ui.js", "tandem/src/web/app.js", "tandem/src/web/app.css"]
---

## Description

The lanes idea deliberately lives in the web view rather than the static TUI. Design seed: "Mockup A" in the first post at http://desktop-wsl.tail1cefc.ts.net:8228/session/urf4z8sWgj8 (three stacked lanes, counts, collapse, and nested research shown in its lane).
