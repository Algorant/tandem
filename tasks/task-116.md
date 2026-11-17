---
id: task-116
uid: 1e919897-d6aa-4f49-be4f-f1727e10d57e
type: task
title: "Polish the current read-only web view"
state: "in-progress"
priority: "high"
effort: "medium"
references: ["task-75", "task-115"]
relatedFiles: ["tandem/src/web/app.css", "tandem/src/web/index.html", "tandem/src/web/ui.js", "site/public/images/web-read-only.png"]
tags: ["ui", "web"]
accord:
  status: "claimed"
  acceptance: ["Existing web view receives a bounded Verdigris visual polish with clearer spacing, typography and layout, without changing its read-only functionality or server/API behavior.", "One current real after still is captured from a nonpersonal isolated sample board; one same-session Sideshow post shows comparable actual before/after plus GitHub-dark Showcase-still context.", "Runnable checks and actual real-app Board/detail/navigation checks pass; preview link is delivered for Algorant’s pick before any publication. Full task-75 remains deferred."]
  claimedAt: "2026-11-17T21:42:24Z"
  validation: ["$ cargo check --manifest-path tandem/Cargo.toml", "$ git diff --check"]
  constraints: ["One bounded styling pass, preview within about one hour; no full web reimagining or new infrastructure/framework/dependencies.", "Use only isolated illustrative sample data; preserve loopback/read-only behavior and public-change hold until Algorant picks.", "No public README insertion, merge/push/release or live app changes before approval."]
  updatedAt: "2026-11-17T21:42:24Z"
createdAt: "2026-11-17T21:42:06Z"
updatedAt: "2026-11-17T21:42:24Z"
assignee: "pi:web-view-visual-polish"
---

## Description

## Request
Algorant, viaHerdra m48c4cecf (2026-11-17): "I have an ai meetup to go to and I want all my repos in decent shape before I go so I need to do all tehse polish tasks." Earlier the web view "looks very ugly".

Today authorizes ONLY one bounded visual polish of the existing single-workspace read-only web view: Verdigris palette, spacing, typography and layout, so a new Showcase still looks decent. Aim to deliver the preview within an hour. This standalone execution Task links task-75 rather than resuming/closing its much larger deferred multi-project orchestration reimagining.

## Scope
One manus pass in an isolated worktree. Inspect current CSS/markup, propose a short concrete styling plan, then implement the bounded candidate on its branch. Prefer CSS changes; markup/JS only minimally if presentation requires it. Keep current Board/Task/Rules/Decisions functionality and DOM behavior, no new product features. Replace the existing web-read-only PNG with a real still of the candidate from a fresh isolated, illustrative sample workspace; no personal/live-board data.

Show genuine BEFORE and AFTER from the real app against the same sample data in ONE Sideshow post in existing tandem docs site session yf_6Zb7WQVc. Also show the candidate still against GitHub-dark README context, clearly only a proposal: the published README deliberately excludes this image and must not be changed without Algorant selecting it. No new standalone sessions, no new demo/movie or broad documentation review.

## Exclusions and gate
No revival of full task-75, orchestration, multi-project/plugin architecture, API/data-model/Rust backend semantics, network/hosting/binding changes, frontend framework/dependency campaign, tool installs, tests/mocks/fixtures, TUI changes, binary release or live app restart. No public merge/push/release or README insertion until Algorant picks. Use one sequential finite capture pass and one disposable browser; no parallel capture services or repeated render investigations. Preserve existing loopback/read-only/security behavior; cleanup owned processes/sample/profile. Report blockers early rather than extending scope.
