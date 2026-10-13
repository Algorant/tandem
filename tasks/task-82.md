---
id: task-82
uid: 36a8474a-8588-4f23-8009-4f9e3c56a7b7
type: task
kind: "papercut"
title: "Web action-button labels wrap outside their button bounds"
state: "in-progress"
priority: "low"
effort: "small"
references: ["task-75", "task-81"]
relatedFiles: ["tandem/src/web/app.css"]
tags: ["ui", "web", "plan-first"]
accord:
  status: "claimed"
  acceptance: ["Refresh view and Apply filters labels stay readable inside their button bounds at 900px, 1280px and 1920px viewport widths without layout overlap."]
  claimedAt: "2026-10-13T15:47:18Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Herdra explicitly authorized this small fix; plan before editing. CSS only in app.css, no JS/HTML/Rust behavior change or deferred task-75 redesign.", "Verify actual browser layout at 900/1280/1920px, preferably both themes, on disposable boards; no injected capture-only CSS workaround.", "Temporary checks only; no permanent tests/fixtures. No push/release/deploy; parent integrates verified technical fix then task-81 recaptures."]
  updatedAt: "2026-10-13T15:47:18Z"
createdAt: "2026-10-13T15:35:10Z"
updatedAt: "2026-10-13T15:47:18Z"
assignee: "worker-task-82-e7773445"
---

## Description

Observed in native Obscura captures of released Tandem 0.17.0 during task-81: Refresh view and Apply filters wrap/overflow the fixed button bounds at tested 900/1280/1920 widths. Authentic web-read-only PNG retained rather than patched browser CSS or product code in the asset-only Task. Capture only, unstarted; do not start deferred task-75's web reimagining or broaden task-81 to fix this.

## Authorized narrow fix

Herdra reviewed task-81 and explicitly asked to fix this first, CSS only, then recapture rather than document a known product bug. Existing stylesheet is tandem/src/web/app.css. Inspect actual computed layout/label bounds before proposing the smallest CSS rule change; retain existing responsiveness/button behavior. New root Worker is Luna/high and plan-first. Parent independently verifies native browser pixels/bounds and integrates this technical fix; task-81 web capture then uses that real updated build. No web orchestration/plugin/source changes.