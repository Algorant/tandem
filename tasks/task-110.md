---
id: task-110
uid: 57e52edc-3962-437d-8649-6fbcc495dea8
type: task
title: "Reconcile tandem's repo and site wordmarks in context"
state: "in-progress"
priority: "medium"
effort: "medium"
references: ["task-109"]
relatedFiles: ["docs/banner.svg", "README.md", "site/src/assets/", "site/public/"]
tags: ["ui", "wordmark"]
accord:
  status: "delivered"
  acceptance: ["Both current marks are shown side by side in their actual README and docs-site contexts.", "Three to five numbered alternatives communicate two abstract things collaborating and each is shown as a README banner, a docs-site header and a favicon at small sizes.", "One Sideshow post in the new 'tandem wordmark' session contains the comparison and options, with tracked repo and live site unchanged."]
  claimedAt: "2026-11-13T16:38:36Z"
  deliveredAt: "2026-11-13T17:42:51Z"
  validation: ["$ git diff --exit-code"]
  summary: "Algorant chose the EXISTING docs-site mark (two interlaced loops + lowercase tandem) and wants repo and site to match it. Read-only handoff complete: high-def dark/light renders of that lockup drawn from the site's own SVG source at 7x, with SVG-vs-HTML split, fonts and colours named; the choice is recorded in the same post. The four explored options stay as record only, no option adopted. Adoption (README banner rebuilt from the site mark) moves to a separate implementation track; nothing tracked changed here."
  evidence: ["Sideshow post http://desktop.tail1cefc.ts.net:8228/session/c4JYCfBnD8Q (post 7tO59nEXJ3g, v9): decide surface now records the choice; high-def CURRENT cards sit after the served-header screenshot", "Mark SVG unmodified: site/src/assets/tandem-mark.svg (strands #689d6a + #ebdbb2) and tandem-mark-light.svg (#689d6a + #504945), 96x96 viewBox, ink 78 x 41.8 units; both attached as downloads", "Name is separate HTML text: Starlight title 24px/600, 0.36px tracking, 88px wide, colour #427b58 light / #d5e7d0 dark; declared system sans, Arial-metric face (Liberation Sans) used in the render", "Served geometry: 64px header band, 40x40px logo box, 27.75px ink-to-ink gap; cards are 1600x720 PNGs rendered from SVG at 7x", "git diff --exit-code clean; git status shows no tracked or untracked changes; MagicDNS session URL HTTP 200"]
  updatedAt: "2026-11-13T17:42:51Z"
createdAt: "2026-11-13T16:38:20Z"
updatedAt: "2026-11-13T17:42:51Z"
assignee: "pi:tandem-wordmark"
---

## Description

Algorant's words, verbatim (2026-11-13, relayed by Herdra m437ba28e):

"First is the discrpeancy between the wordmark on the doc site (trytandem.dev) and the repo. Those should agree somewhat. lets start shopping that in context and maybe even simplify it a bit. the cassette tape tandem one is a bit too basic but the trytandem one is a bit too complex. lets meet somewehre in the middle. the invocation should be something of collaborative effort btween two things (humans and agents, namely, but its abstract). this is one angle and something we can begin delegating independently. the next thing would be how I expect the docs site layout to be. If I go to any random documentation site for a project, say its hokum or mrboxington https://mr-boxington.jdx.dev/ https://hokum.app/ and they have a nice basic but eye catching landing page, and tehn extensive but simple documentation setup. what are they using, and how do they stand it up?"

## Track 1 — mockups only
Show both current marks side by side in their real contexts: the README's cassette-tape banner and trytandem.dev's header mark. Explore 3–5 numbered middle-ground options, simpler than the site's and richer than the cassette; carry the abstract idea of two things working together (humans and agents, without literal person/robot symbols). Show EACH option in a README banner, docs-site header, and at small favicon sizes. Use current repo-style rules and verdigris family, lowercase tandem. One post in a fresh Sideshow session named exactly 'tandem wordmark'.

Do not change or build anything into the tracked repo or live site. Put temporary proposals only under ignored .pi/proposals/tandem-wordmark/ or another explicitly disposable local artifact directory; never overwrite the real marks. No commits, pushes, deployment, dependency/tool installation or unrelated cleanup. Taste/final adoption remain Algorant's choice. Send the completed post and artifact evidence to the starter; starter reports to Herdra.


## Additional current-wordmark view
Algorant, verbatim (2026-11-13, via Herdra m4d4d7d8b): "for tandem wordmark: high def version of the docs site wordmark, can you show me that in dark and light?"

Add large, crisp renders of the EXISTING docs-site wordmark from its actual source SVG, on the real dark/light site backgrounds, to the same wordmark post/session. This is not a proposed replacement or adoption. Show both complete lockups; if the name is separate HTML, reproduce current type/weight and label that fact. No screenshot upscaling and no tracked repo/site changes.

## Algorant’s choice — exploration concluded
Herdra m4b71009c (2026-11-13) relays the exact follow-up: "I just want the repo and the docs site to match with that, basically". The EXISTING trytandem.dev mark wins; none of the four new options is adopted. The high-resolution dark/light request folds into applying that current mark consistently. Close the exploration after its final artifact handoff; implementation is separately scoped and authorized, not an extra design-acceptance gate.