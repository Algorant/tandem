---
id: task-115
uid: 7b17f57b-91be-4c84-9851-ec6c76e68224
type: task
title: "Add a demo and three-still Showcase to the README"
state: "in-progress"
priority: "low"
effort: "medium"
references: ["task-114", "task-75"]
relatedFiles: ["README.md", "docs/brand/", "site/public/images/"]
tags: ["docs"]
accord:
  status: "claimed"
  acceptance: ["A looping WebP demo of about 10–12 s at quality 90, recorded headlessly from the real tandem on an isolated demo board, sits full width right after the README intro.", "A single ## Showcase immediately after intro contains the existing demo and three real stills (Board, Task detail, Session boards) in a clean row/grid with small captions; no web-view still; License remains last.", "A GitHub-dark Sideshow mock is shown first; only after Algorant picks is it committed, pushed and verified rendering on GitHub, and the link sent to Herdra."]
  claimedAt: "2026-11-17T15:40:13Z"
  validation: ["$ git diff --check"]
  updatedAt: "2026-11-17T19:49:47Z"
createdAt: "2026-11-17T15:40:03Z"
updatedAt: "2026-11-17T19:50:31Z"
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

## One matching layout revision for the hub
Algorant Ceph m8e94fed4 requests one bounded layout revision: render demo after intro plus2×2 **Showcase** near end, using existing media only. Update supporting postZ2jkMT07Gl0 IN PLACE (no new session/post/captures/media, full reread or render loops). Keep sourceREADME and mock matching, including Showcaseheading. Publicchanges remain held. Return exact supporting /p/link+version+whatrendernowshows toAlgorantCeph for replacementrowinsolehub iMTCezlsbAo/post8t9eEWa7nIY. Preserveprivacyholds; no cutover/networkwork.

## Opening Showcase placement — latest chosen wording
Algorant, via Algorant Ceph m53fd0752: "lets just go with showcase as the word to be applied everywhere." In-place current-draft revision only, no public changes. Algorant Ceph clarified m58228a40: one ## Showcase immediately after intro containing the existing demo followed directly by the existing2×2stills table and same captions. Install/Core/Development follow; License last. No duplicate Showcase or stray headerless media table. Same supportingpostZ2jkMT07Gl0 updatedinplace, no new media/captures/renderinvestigation.

## Remove web-view still — latest Algorant direction
Algorant, verbatim viaHerdra mae37f1eb (2026-11-17): "for tandem: remove teh web view, we will have to revisit that. it looks very ugly. filed as a task in tandem itself and for now just remove the picture from showcase".
Remove ONLY the web-view still from the draft; retain demo and Board/Taskdetail/Sessionboards in a clean3-image row/grid. Keep sharedwebimage used bydocs; no redesign/re-capture/newmedia. Update existingpostZ2jkMT07Gl0 inplace and notifysolehubowner t21y. PublicREADMEremainshelduntilapproval. Link Algorant’s separateweb-viewTask onceIDconfirmed (notfoundinactivesearch; Herdraasked, no duplicatecreated).

Web-view follow-up resolved: existing deferred task-75 covers the web reimagining and absorbs archived/canceled task-5. Recorded Algorant’s visual complaint there at lowpriority, retaining blocked/not-started deferral; no duplicate created or redesign started. README draft7b9753b and supportingpostZ2jkMT07Gl0v13 omit only the web still, retaindemo+3stills. Hubowner notified.