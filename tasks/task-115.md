---
id: task-115
uid: 7b17f57b-91be-4c84-9851-ec6c76e68224
type: task
title: "Add a demo clip and Screenshots section to the README"
state: "in-progress"
priority: "low"
effort: "medium"
references: ["task-114"]
relatedFiles: ["README.md", "docs/brand/", "site/public/images/"]
tags: ["docs"]
accord:
  status: "claimed"
  acceptance: ["A looping WebP demo of about 10–12 s at quality 90, recorded headlessly from the real tandem on an isolated demo board, sits full width right after the README intro.", "A ## Screenshots section near the end shows four real stills in a 2-column table with small captions; License remains last.", "A GitHub-dark Sideshow mock is shown first; only after Algorant picks is it committed, pushed and verified rendering on GitHub, and the link sent to Herdra."]
  claimedAt: "2026-11-17T15:40:13Z"
  validation: ["$ git diff --check"]
  updatedAt: "2026-11-17T15:40:13Z"
createdAt: "2026-11-17T15:40:03Z"
updatedAt: "2026-11-17T17:57:30Z"
assignee: "pi:readme-demo-clip"
---

## Description

## Description

Algorant, verbatim (2026-11-17, via Herdra mbd8b8f6d): "yes for herdra to establish A, the demo sells it first. algOS and some other repos should also follow this pattern... the web-ps would have to be created for some of those"

The README standard (Algorant board task-25, owned by the Algorant Ceph):
- one looping WebP demo, about 10–12 s, quality 90, full width, right after the intro
- stills in a `## Screenshots` 2-column table with small captions near the end
- License last

## Scope
Make a headless demo of tandem in use, e.g. VHS (installed) of `tandem tui` on a demo board in an isolated scratch workspace (temporary HOME/XDG, illustrative records only); the content choice is the owner's, picking what sells tandem. Convert to looping WebP with installed ffmpeg. Put the four current stills (reuse the existing real 0.17.7 captures) in the Screenshots table. Keep the banner and intro as they are; License stays last.

## Gate
Mock the README in Sideshow as GitHub dark renders it, in the tandem docs site session, with the demo playing. Algorant picks before anything is committed to main, pushed or published. After his pick: commit, push, verify the GitHub README renders (demo loads, table, captions), then report the link to Herdra.

## Boundaries
No live boards, private records, personal paths or machine names in frame. One sequential capture; no parallel capture services, no new tools installed without asking, no new framework. Demo asset size reasonable for a README (state the size).


## Consolidated review gate — latest coordination
Algorant Ceph m7b87f16a reports Algorant approved ONE README/media review hub on portfolio task-25. Existing posts are supporting previews/history, not parallel asks. No new sessions, captures or render investigations; no public changes until approval. Layout is now case-by-case: Showcase section or compact3–4-image row under banner; demo-first only where useful, not a universal mandate.

Submitted tandem recommendation: demo after intro (it shows live TUI/CLI coordination) plus2×2stills near end. Currentstatus: isolated mock/branch7fdd8bf only. Decision: approveclip/layout. Existing supportingpreview: http://desktop.tail1cefc.ts.net:8228/p/Z2jkMT07Gl0. Await hubURL/selection; no new parallel review request or implementation.