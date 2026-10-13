---
id: task-76
uid: 5f373353-a0d5-4373-87fc-5c533f439c89
type: task
title: "Docs and site overhaul: refreshed 0.17.0 seven-page plan"
state: "todo"
priority: "medium"
effort: "large"
references: ["task-51", "task-67", "task-70", "task-71", "task-72", "task-73", "task-74", "task-75"]
relatedFiles: ["plan/docs-overhaul.md", "docs/", "site/", "README.md", "tandem/README.md", "protocol/README.md", "docs/concepts/index.md", "docs/cli/index.md"]
tags: ["docs"]
accord:
  status: "ready"
  acceptance: ["The public site retains the approved seven-page shape, starts with minimal reader-facing prose and incorporates Herdra's approved style guide when available.", "Content accurately explains shipped 0.17.0 session boards, add/remove paths, full-TUI switching, chooser activity, active-board sync versus inactive local snapshots, and agent setup; optional Pi /task routing is not presented as a Tandem CLI command. Web stays single-workspace/read-only.", "Workspace layout, lifecycle/review claims, installation/version guidance and Decision examples from tasks 51/67 are verified and corrected across docs and READMEs.", "Real 0.17.0 product captures, authorized Excalidraw diagram exports and existing teaser presentation are legible and reviewed in the existing docs Sideshow session with main-versus-branch comparisons.", "Site build and link checks pass; public pages contain no placeholders; final integration/deployment occurs only after Algorant's approval.", "A dedicated board-storage/sidecar-sync section within the seven-page shape accurately explains source Git versus the Tandem-managed tandem branch, multi-machine use, pending, true conflicts and held edits, with the required Excalidraw diagram and real sync-status/TUI-conflict captures."]
  validation: ["$ just site-build", "$ cd site && bun run check:links"]
  constraints: ["Planning resumed; no prose/assets/implementation Workers until Herdra approves the refreshed plan and Excalidraw export/use path.", "Interim writing: minimal, for the real reader, no narrator voice or padding; apply Herdra task-19 style guide when available.", "Preserve rebased docs-overhaul scaffold at 57381c9; no push or deployment without approval.", "Only shipped 0.17.0 features: manual ephemeral session set and local summaries; no permanent workspaces.toml, Herdr auto-discovery or multi-workspace web."]
  note: "Algorant restarted planning via Herdra after 0.17.0 landed. Refreshed plan awaits sign-off; no page writing or implementation Worker yet. Rebased scaffold onto main; Excalidraw export/use decision surfaced."
  updatedAt: "2026-10-13T13:16:39Z"
createdAt: "2026-10-12T15:51:52Z"
updatedAt: "2026-10-13T13:16:39Z"
---
## Restart: planning for 0.17.0

Algorant restarted task-76 via Herdra. Planning is unpaused; no page writing, asset creation, implementation Worker launch, push or deployment until Herdra approves the refreshed plan. Until Herdra task-19 style guide lands, use minimal reader-facing prose: no narrator voice, padding or speculative features. Incorporate the guide when available.

Rebased, not restarted: docs-overhaul moved from 6f959a8 to 57381c9 on current main 1f8b59a, preserving all four scaffold/revision commits with no conflicts. Dedicated checkout: /home/algorant/Projects/worktrees/tandem/docs-overhaul. Branch is clean and unpushed; main remains clean at the released 0.17.0 commit. Rebase preserves main's external-dependency clarification, optional archive notes, README routing description, and shipped TUI work.

## Refreshed seven-page plan (awaiting sign-off)

1. Home: one-line purpose, existing teaser, primary install link, minimal shared-board explanation.
2. Quickstart: install and verify; one real Task from init through claim/deliver/complete to Logs, with verified 0.17.0 commands/output.
3. Concepts: Task/Accord lifecycle, Decisions/Rules, and a dedicated Board storage and sync section explaining source Git versus the Tandem-managed tandem branch, two-machine use, pending, conflicts and held edits.
4. Agents: short tool-neutral AGENTS.md snippet and example request; optional Pi task-routing description note clearly attributed to that integration, not a Tandem CLI command.
5. TUI and web: current Board and editable session working set (w, Ctrl-A add, Ctrl-D remove inactive, full-TUI switching), chooser activity/recorded assignees, active-board sync versus inactive local snapshots; only current single-workspace read-only web.
6. CLI: compact command/key-flag reference, repeatable --add-workspace, JSON envelope and links to the storage/sync explanation rather than duplicated prose.
7. Configuration: compact supported theme/badge settings and precedence; no permanent workspace roster file.

Changes from revision 3: retain the seven-page shape and minimal chrome; replace outdated orchestration/web expectations with actually shipped 0.17.0 behavior; expand the one-line sync explanation into a dedicated Concepts section; keep delivered needing triage distinct from explicit Validation and blocked Accord; explain that compatible board edits merge automatically, while true conflicts still require resolution. Source Git independence is not a promise of zero board conflicts. Fold stale scope from tasks 51/67 into the rewrite, including README/maintainer guide claims where applicable.

## Visuals

Excalidraw: retain the short shared-board/how-it-works diagram (D1) and lifecycle diagram (D3); add the required sidecar-sync diagram (D4), covering source versus tandem histories, two machines, a true conflict and a held edit. Keep it legible with short labels; no Mermaid/SVG substitute or hand-written Excalidraw scene.

Actual 0.17.0 captures on disposable Git boards: Board (C3); session chooser/activity and add/remove/full-TUI transition; sync status showing useful pending/conflict/held distinctions; conflict surfaced in the TUI; current read-only web (C10). Quickstart uses real text output, not screenshot tiles. Existing encoded teaser/poster are retained; original fframes source stays read-only.

Export gate: the current Excalidraw skill explicitly permits vault drawings and in-memory PNG agent self-checks, not site/share exports. Before any asset Worker, obtain an approved Excalidraw export/use path from Herdra/the Excalidraw owner. Do not add an extension/exporter or silently reuse self-check PNGs for public assets. PNG exports, if approved, avoid unresolved SVG-font handling; inspect light-page readability without inventing a second palette.

## Proposed Worker split

After plan and export gate approval:
- One standalone root Task for visuals: only approved diagram/capture assets under site/public/images (editable drawings via the authorized vault workflow). No prose/style/code implementation, no external repository code edits. Release captures use disposable replicas; no permanent test fixtures.
- One root Task-76 Worker for all seven pages, README corrections and site layout/config, seeded from the rebased scaffold. Keep coupled content/layout in one Worker; integrate/review visual assets first to avoid parallel asset-reference churn.
Both are isolated, explicit Luna/high, plan-first. Do not create an Epic or independently delegate checklist Subtasks. File the visuals Task only after sign-off. Parent verifies actual assets/rendering and the completed site; final visible acceptance remains Algorant's.

## Checks and review

Verify against released tandem 0.17.0 --help/native output and protocol 0.5.0. Runnable site build/link checks, responsive dark/light renders, image legibility/alt text and no placeholders in public pages. Capture normal main versus branch side by side and publish each meaningful revision as a new post in existing Sideshow session J0qH-JN7PqI, with page map/cut list and bounded MagicDNS check. No deployment without approval. Resolve old-URL handling before deployment.

## Absorbed requirements

Task-51: current tasks/decisions/rules/logs/per-actor-events layout.
Task-67: lifecycle/review wording, supported badge/config claims, current installer/version guidance and real Decision metadata.
Dedicated storage/sync section: automatic publish/refresh, fresh clone/shared worktrees, offline pending/local-only/safety copy, compatible merges versus true conflicts, and unpublishable held edits. Distinguish conflict resolution local/remote/edited from validation-hold keep-remote repair; unaffected records continue. Include required Excalidraw and real status/TUI conflict captures. Do not revive stale sysup/~/.dotfiles/checkpoint rollout instructions.
