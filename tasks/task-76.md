---
id: task-76
uid: 5f373353-a0d5-4373-87fc-5c533f439c89
type: task
title: "Docs and site overhaul: approved seven-page shape, paused"
state: todo
priority: "medium"
effort: "large"
blockers: ["task-75"]
references: ["task-51", "task-67", "task-70", "task-71", "task-72", "task-73", "task-74", "task-75"]
relatedFiles: ["plan/docs-overhaul.md", "docs/", "site/", "README.md", "tandem/README.md"]
tags: ["docs"]
accord:
  status: "blocked"
  acceptance: ["The public site retains the approved seven-page revision-3 shape and incorporates Herdra's approved style guide.", "Content accurately explains the latest shipped features and how to use orchestrator mode, opt-in multi-workspace web, Pi /task project routing and agent setup, without claiming integration commands are Tandem CLI commands.", "Workspace layout, lifecycle/review claims, installation/version guidance and Decision examples from tasks 51/67 are verified and corrected across docs and READMEs.", "Real product captures, Excalidraw diagrams and teaser presentation are rendered and reviewed in the existing Sideshow session with before/after comparisons.", "Site build and link checks pass; final integration and deployment occur only after Algorant's approval."]
  validation: ["$ just site-build", "$ cd site && bun run check:links"]
  constraints: ["Paused until Herdra task-19 and Tandem orchestrator-mode tasks 71–75 are complete; no implementation starts from this backlog filing.", "Keep docs-overhaul clean at 6f959a8 and unpushed while parked."]
  note: "Paused by Algorant via Herdra: resume after Herdra task-19 style guide and orchestrator mode tasks 71–75, with owner direction. Approved revision-3 seven-page branch remains parked unchanged and unpushed."
  updatedAt: "2026-10-12T16:29:43Z"
createdAt: "2026-10-12T15:51:52Z"
updatedAt: "2026-10-12T16:29:43Z"
---

## Description

## Direction and pause

Algorant accepted revision 3's seven-page shape via Herdra: Home, Quickstart, Concepts, Agents, TUI and web, CLI, Configuration. PAUSED: resume only after Herdra's style guide (Herdra task-19, tracked externally) and orchestrator mode (Tandem tasks 71–75) are complete, with owner direction. No rewrite, merge, push or deployment while paused.

Parked branch: docs-overhaul, clean at 6f959a8, unpushed. Leave the branch exactly as is. Main separately contains the /task-routing description front matter in README.md (738fc92); preserve it when eventually integrating. Approved shape preview: http://desktop.tail1cefc.ts.net:8228/session/J0qH-JN7PqI (revision 3). Plan: plan/docs-overhaul.md.

On resumption, incorporate Herdra's style guide and document the latest shipped features and practical usage, including orchestrator-mode workspace list/switcher/overview, opt-in multi-workspace web, /task routing via project description front matter (attribute this to the Pi integration, not a Tandem CLI command), and agent setup. Keep the agreed compact seven-page structure, concise headings and minimal chrome.

## Absorbed scope

Replaces task-51: workspace layout descriptions must match the current protocol (tasks/, decisions/, rules/, logs/, per-actor events/).
Replaces task-67: verify and correct stale review-status badges/config claims, Validation wording, install/version pins and unsupported Decision date/supersession examples across public docs and READMEs. Keep delivery separate from explicit validation; verify claims against the current executable/protocol, not stale docs.

Finish concise CLI/configuration reference, actual TUI/web captures and Excalidraw diagrams using that project's skill; preserve read-only teaser source. Old-URL handling and final media/export/deployment choices still need resolution before launch. Publish review revisions in the existing docs Sideshow session with main-versus-branch comparisons and checked MagicDNS access. No deployment without approval.
