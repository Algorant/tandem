---
id: task-114
uid: 0fed0330-232d-4c11-aefc-4aa813888ff2
type: task
title: "Apply the eight README and docs-site tweaks"
priority: "low"
effort: "medium"
references: ["task-113"]
relatedFiles: ["README.md", "docs/quick-start/index.md", "docs/agents/index.md", "docs/tui/index.md", "docs/concepts/index.md", "docs/reference/", "site/public/images/", "site/src/components/homepage/RepoBoard.astro"]
tags: ["docs"]
accord:
  status: "accepted"
  acceptance: ["All eight approved README/docs tweaks are applied as described, and no other content or design changes are made.", "TUI and Concepts screenshots are current real captures from an isolated sample board; Concepts keeps one sync-status image, and unreferenced old images are removed.", "Site build, link check and git diff --check pass; after deployment the README on GitHub and the changed live pages are checked, with after-shots in session yf_6Zb7WQVc."]
  claimedAt: "2026-11-16T05:16:29Z"
  deliveredAt: "2026-11-16T05:26:59Z"
  validation: ["$ just site-build", "$ cd site && bun run check:links", "$ git diff --check"]
  summary: "All eight README/docs tweaks applied on branch readme-and-docs-tweaks at 90ed14e; TUI/web/sync images recaptured from tandem 0.17.7 on an isolated sample board; three unreferenced images removed. Awaiting starter merge, deploy and live after-shots."
  evidence: ["just site-build: 8 pages built, Complete", "cd site && bun run check:links: 173 internal links across 20 HTML files OK", "git diff --check: clean", "Local dist renders of /, /quick-start/, /agents/, /tui/, /concepts/, /cli/ at dark 1440x900 and light 390x844, DPR1: correct h1/theme, 0 broken images, expected images visible", "One sequential capture pass: one ttyd PTY + one tandem web on loopback + one headless Chromium; Chromium exited gracefully (code 0); no owned processes/profiles left"]
  filesChanged: ["README.md", "docs/agents/index.md", "docs/cli/index.md", "docs/concepts/index.md", "docs/quick-start/index.md", "docs/tui/index.md", "site/public/images/", "site/src/components/homepage/RepoBoard.astro", "site/src/styles/verdigris.css"]
  updatedAt: "2026-11-16T05:30:40Z"
createdAt: "2026-11-16T05:16:14Z"
updatedAt: "2026-11-16T05:30:40Z"
assignee: "pi:readme-and-docs-tweaks"
archivedAt: "2026-11-16T05:30:40Z"
resolution:
  outcome: "completed"
  note: "Merged by worker_finish: readme-and-docs-tweaks → main at 90ed14e368ead9ac11fc1205cd0b4e348ab707fd."
---

## Description

## Description

Algorant, verbatim (2026-11-15, via Herdra mf7f9782e, answering the light review post 61LDSrUHV1g in session yf_6Zb7WQVc): "yeah do all 8 go ahead".

Earlier request (via Herdra m863d3d85): "I don't need anything specific with tandem as far as readiness, I think I just wnated to review the readme a bit, and the docs site. I think we've already done most of it". Priority: below herdra and dotfiles.

## The eight approved tweaks
1. README: add the real dark/light Board screenshot (existing site/public/images/homepage/ assets) under the opening line, with GitHub picture selection.
2. README: replace "Do not clone or build it to use tandem … point your agent at them, or at one part of this repo." with roughly "You don't need this repo to use tandem: install the release, then follow the quickstart."
3. README: capitalise the language name, "Rust" (project names stay lowercase).
4. Quickstart: cut the init paragraph to about "Init adds six starter rules and points any existing agent instructions file at them. Read them with `tandem rules list`."; move the detail to the Agents page.
5. Reshoot the 0.17.0-era TUI/Concepts screenshots from the current installed binary on an isolated sample board; keep one sync-status image on Concepts instead of four. Delete replaced images that nothing references.
6. Homepage caption/alt text: "The tandem TUI · example workspace" — no "Actual", lowercase tandem, no version number.
7. TUI page: cut internal detail (provisional-ID selection mechanics; web-view /api/v1, headers, caching) to what a user does; move the server detail to Reference if worth keeping.
8. Quickstart: one consistent command-block style, commands to copy separate from sample output.

## Boundaries
Approved as a set; no second taste gate. Capture method: ONE sequential pass, one isolated sample board/HOME/XDG, one disposable browser, the same loopback ttyd method used for task-113; no parallel capture services, no new framework, no recolouring or fake output. Starter merges (Worktrunk --no-remove), pushes, verifies live, posts after-shots in session yf_6Zb7WQVc, then worker_finish. No other redesign.
