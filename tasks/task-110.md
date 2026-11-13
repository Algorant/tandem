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
  deliveredAt: "2026-11-13T16:59:15Z"
  validation: ["$ git diff --exit-code"]
  summary: "Mockup-only wordmark study: both current marks shown in their real contexts, four numbered middle-ground marks (twin t, weave, embrace, yoke) each shown as README banner, docs-site header (dark+light) and favicon with true 16/24/32 px legibility renders. One Sideshow post in a new 'tandem wordmark' session; tracked repo, docs site and deployment unchanged."
  evidence: ["Sideshow post: http://desktop.tail1cefc.ts.net:8228/session/c4JYCfBnD8Q (post 7tO59nEXJ3g)", "One bounded reachability check: curl --noproxy '*' --max-time 3 on the MagicDNS session URL returned HTTP 200; localhost:8228 returned HTTP 200", "Artifacts (ignored, not committed): .pi/proposals/tandem-wordmark/ with generate.py, legibility.py, svg/ sources and png/ boards", "git diff --exit-code: clean; git status: no tracked or untracked changes", "Live header measured from trytandem.dev: 64px tall, light #f3ebcf / dark #222526, title 24px/600 in theme accent; one favicon file per theme"]
  updatedAt: "2026-11-13T16:59:15Z"
createdAt: "2026-11-13T16:38:20Z"
updatedAt: "2026-11-13T16:59:15Z"
assignee: "pi:tandem-wordmark"
---

## Description

Algorant's words, verbatim (2026-11-13, relayed by Herdra m437ba28e):

"First is the discrpeancy between the wordmark on the doc site (trytandem.dev) and the repo. Those should agree somewhat. lets start shopping that in context and maybe even simplify it a bit. the cassette tape tandem one is a bit too basic but the trytandem one is a bit too complex. lets meet somewehre in the middle. the invocation should be something of collaborative effort btween two things (humans and agents, namely, but its abstract). this is one angle and something we can begin delegating independently. the next thing would be how I expect the docs site layout to be. If I go to any random documentation site for a project, say its hokum or mrboxington https://mr-boxington.jdx.dev/ https://hokum.app/ and they have a nice basic but eye catching landing page, and tehn extensive but simple documentation setup. what are they using, and how do they stand it up?"

## Track 1 — mockups only
Show both current marks side by side in their real contexts: the README's cassette-tape banner and trytandem.dev's header mark. Explore 3–5 numbered middle-ground options, simpler than the site's and richer than the cassette; carry the abstract idea of two things working together (humans and agents, without literal person/robot symbols). Show EACH option in a README banner, docs-site header, and at small favicon sizes. Use current repo-style rules and verdigris family, lowercase tandem. One post in a fresh Sideshow session named exactly 'tandem wordmark'.

Do not change or build anything into the tracked repo or live site. Put temporary proposals only under ignored .pi/proposals/tandem-wordmark/ or another explicitly disposable local artifact directory; never overwrite the real marks. No commits, pushes, deployment, dependency/tool installation or unrelated cleanup. Taste/final adoption remain Algorant's choice. Send the completed post and artifact evidence to the starter; starter reports to Herdra.
