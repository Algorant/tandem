---
id: task-76
uid: 5f373353-a0d5-4373-87fc-5c533f439c89
type: task
title: "Docs and site overhaul: approved seven-page shape, paused"
state: todo
priority: "medium"
effort: "large"
blockers: ["task-74"]
references: ["task-51", "task-67", "task-70", "task-71", "task-72", "task-73", "task-74", "task-75"]
relatedFiles: ["plan/docs-overhaul.md", "docs/", "site/", "README.md", "tandem/README.md", "protocol/README.md", "docs/concepts/index.md", "docs/cli/index.md"]
tags: ["docs"]
accord:
  status: "blocked"
  acceptance: ["The public site retains the approved seven-page revision-3 shape and incorporates Herdra's approved style guide.", "Content accurately explains the latest shipped features and how to use orchestrator mode, opt-in multi-workspace web, Pi /task project routing and agent setup, without claiming integration commands are Tandem CLI commands.", "Workspace layout, lifecycle/review claims, installation/version guidance and Decision examples from tasks 51/67 are verified and corrected across docs and READMEs.", "Real product captures, Excalidraw diagrams and teaser presentation are rendered and reviewed in the existing Sideshow session with before/after comparisons.", "Site build and link checks pass; final integration and deployment occur only after Algorant's approval.", "A dedicated board-storage/sidecar-sync section within the seven-page shape accurately explains source Git versus the Tandem-managed tandem branch, multi-machine use, pending, true conflicts and held edits at the latest shipped version, with the required Excalidraw diagram and real sync-status/TUI-conflict captures."]
  validation: ["$ just site-build", "$ cd site && bun run check:links"]
  constraints: ["Paused until Herdra task-19 style guide and revised TUI orchestrator tasks 73/74 are complete, with owner direction; deferred web task-75 is not a prerequisite.", "Keep docs-overhaul clean at 6f959a8 and unpushed while parked."]
  note: "Paused by Algorant via Herdra: resume after Herdra task-19 style guide and orchestrator mode tasks 71–75, with owner direction. Approved revision-3 seven-page branch remains parked unchanged and unpushed."
  updatedAt: "2026-10-13T12:23:52Z"
createdAt: "2026-10-12T15:51:52Z"
updatedAt: "2026-10-13T12:23:52Z"
---

## Description

## Direction and pause

Algorant accepted revision 3's seven-page shape via Herdra: Home, Quickstart, Concepts, Agents, TUI and web, CLI, Configuration. PAUSED: resume only after Herdra's style guide (Herdra task-19, tracked externally) and orchestrator mode (revised TUI tasks 73/74; web task-75 deferred) are complete, with owner direction. No rewrite, merge, push or deployment while paused.

Parked branch: docs-overhaul, clean at 6f959a8, unpushed. Leave the branch exactly as is. Main separately contains the /task-routing description front matter in README.md (738fc92); preserve it when eventually integrating. Approved shape preview: http://desktop.tail1cefc.ts.net:8228/session/J0qH-JN7PqI (revision 3). Plan: plan/docs-overhaul.md.

On resumption, incorporate Herdra's style guide and document the latest shipped features and practical usage, including orchestrator-mode workspace list/switcher/overview, opt-in multi-workspace web, /task routing via project description front matter (attribute this to the Pi integration, not a Tandem CLI command), and agent setup. Keep the agreed compact seven-page structure, concise headings and minimal chrome.

## Absorbed scope

Replaces task-51: workspace layout descriptions must match the current protocol (tasks/, decisions/, rules/, logs/, per-actor events/).
Replaces task-67: verify and correct stale review-status badges/config claims, Validation wording, install/version pins and unsupported Decision date/supersession examples across public docs and READMEs. Keep delivery separate from explicit validation; verify claims against the current executable/protocol, not stale docs.

Finish concise CLI/configuration reference, actual TUI/web captures and Excalidraw diagrams using that project's skill; preserve read-only teaser source. Old-URL handling and final media/export/deployment choices still need resolution before launch. Publish review revisions in the existing docs Sideshow session with main-versus-branch comparisons and checked MagicDNS access. No deployment without approval.

## Required dedicated section: board storage and sidecar sync

Algorant added this requirement via Herdra after the 0.17.0 release. The overhaul remains PAUSED; this is scope/audit capture only, not permission to rewrite, capture assets or create diagrams now.

Within revision 3's agreed seven-page shape, give board storage/sync its own clearly titled section (prefer Concepts → Board storage and sync; a page is acceptable only without expanding the agreed shape). Explain that the ignored local .tandem/ board is versioned on the repository's separate tandem branch and synchronized by Tandem itself, independently of source Git history, staging, pull and push. Explain why this removes board/source merge friction and supports multiple machines without promising that contradictory board edits cannot conflict.

Cover normal automatic publish/refresh, fresh clones and shared worktree boards, offline pending/local-only status and local safety copy, automatic record/field merges versus true conflicts, and held unpublishable local edits. Distinguish conflict resolution (local/remote/edited) from validation-hold repair (keep remote), explain unaffected records continue, and include a short practical two-machine example. Clarify the 0.17.0 session working set: inactive chooser summaries read local snapshots only; ongoing background sync follows the active full-TUI board. Local path selection does not monitor remote-only machines.

Required visuals on resumption:
- An Excalidraw diagram, using the Excalidraw project's skill, showing source branch versus tandem board branch, two machines syncing through the shared remote, one conflicting record and one held local edit; show their different outcomes and do not imply Git/source conflict markers are used.
- Real captures of tandem sync status (including pending/conflict/held distinctions where useful) and a real conflict as surfaced in the TUI, generated safely on disposable Git-backed replicas, never by damaging project/user boards. Label commands/version and review in the existing docs Sideshow session.

Read-only audit at 0.17.0:
- main docs/workspace/index.md:64–76 explains the sidecar branch, automatic sync, multiple machines, fresh clones/worktrees and safety copy, but gives no dedicated held-edit/pending walkthrough or visuals.
- main docs/cli/index.md:273–297 is the strongest accurate operational reference: pending/local-only, conflict resolution, held edits and keep-remote repair. docs/guides/agents-and-adapters.md:95–102 also states the authority boundary.
- main upgrading-to-independent-sync guide has useful day-to-day prose but stale sysup mise/~/.dotfiles/checkpoint rollout instructions; its held-edit 'fix the file' wording conflicts with its own Never section and misses the newer validation-hold/shared-read repair distinction. Do not use it as current 0.17.0 setup guidance.
- parked docs-overhaul concepts page reduces storage to one sentence and sync to one conflict sentence; workspace and migration guide were removed. The detailed CLI sync section survives as an explicitly uncondensed placeholder, but the short reader-facing flow omits pending, held edits, multi-machine explanation and visuals. 'Two machines change the same Task differently' overstates conflicts: compatible changes/independent fields merge automatically.
