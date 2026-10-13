---
id: task-82
uid: 36a8474a-8588-4f23-8009-4f9e3c56a7b7
type: task
kind: "papercut"
title: "Web action-button labels wrap outside their button bounds"
state: "todo"
priority: "low"
effort: "small"
references: ["task-75", "task-81"]
relatedFiles: ["tandem/src/web/app.css"]
tags: ["ui", "web", "plan-first"]
accord:
  status: "ready"
  acceptance: ["Refresh view and Apply filters labels stay readable inside their button bounds at 900px, 1280px and 1920px viewport widths without layout overlap."]
  claimedAt: "2026-10-13T15:47:18Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Herdra explicitly authorized this small fix; plan before editing. CSS only in app.css, no JS/HTML/Rust behavior change or deferred task-75 redesign.", "Verify actual browser layout at 900/1280/1920px, preferably both themes, on disposable boards; no injected capture-only CSS workaround.", "Temporary checks only; no permanent tests/fixtures. No push/release/deploy; parent integrates verified technical fix then task-81 recaptures."]
  note: "discarded: Herdra explicitly authorized canceling task-82 as an Obscura capture false positive, not a product CSS bug. Parent inspected actual paired screenshots; unchanged Chromium fits Refresh/Apply labels at 900/1280/1920 in both themes. No source edits; clean checkout independently confirmed. Owned temporary servers/drivers/board cleaned and ports 18782/9515 closed; seven evidence PNGs preserved and published in docs session J0qH-JN7PqI post MdEg0IfN_cs. Discard only this disposable no-source attempt; parent then cancels Task with permanent rendering-gap note."
  updatedAt: "2026-10-13T16:15:02Z"
createdAt: "2026-10-13T15:35:10Z"
updatedAt: "2026-10-13T16:15:02Z"
---
## Investigation outcome — direction pending

Herdra authorized a CSS-only fix after task-81's image showed button-label wrapping. Plan-first/read-only Worker reproduced the pixel issue in Obscura at 1280px, but genuine installed Chromium with native WebDriver sizes at 900/1280/1920 and explicit CDP light/dark media paints both labels on one line inside their boxes, on the same unchanged app build/data. Refresh text 98.72×17 fits button124.72×40.80; Apply text92.47×17 fits button118.47×40.80. Obscura's rectangle APIs were misleading for painted text. Parent inspected paired PNGs: /tmp/pi-codemode-6ccacd6a7a768d37.png and /tmp/task82-chrome-{light,dark}-{900,1280,1920}.png.

No source edits. Evidence published in existing docs session J0qH-JN7PqI; propose closing this false-positive product papercut and using genuine Chromium for task-81 public web recapture, not an engine-specific CSS workaround. Await Herdra direction before cleanup/cancel or task-81 web gate release. Task-75 remains deferred/unstarted.