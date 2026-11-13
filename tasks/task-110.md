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
  deliveredAt: "2026-11-13T17:03:50Z"
  validation: ["$ git diff --exit-code"]
  summary: "Published four numbered middle-ground wordmark directions in one 'tandem wordmark' post, with current repo/site marks side by side and every alternative in README banner, docs header (light/dark) and small favicon contexts. Mockups only; no source/site changes or brand adoption."
  evidence: ["Session c4JYCfBnD8Q / post7tO59nEXJ3g, current version4. One post includes current source marks side by side, full current README banner, measured/reconstructed light/dark docs headers and the actual live header capture; current favicon included.", "Options1 Twin t (shared crossbar),2 Weave (two paths cross),3 Embrace (arcs around shared core),4 Yoke (two arrive, one continues). Each shown in1280×320 README banner,64px docs header in both themes and a16px browser tab/48px tile. Extra legibility sheet renders16/24/32px at6× without smoothing.", "Parent inspected bounded thumbnail renders of today, all4 option boards and pixel-legibility sheet. One correction pass added literal side-by-side current comparison and corrected palette/recommendation wording. Weave has two strand colours; ochre only where a third/shared element exists. Weave preference explicitly taste, not an exclusive16px legibility claim; yoke also survives.", "Typography remains a stand-in (Adwaita Sans600/-0.018em), not an adoption decision. Headers/favicon placements are reconstructions from live geometry/colours, clearly labelled; no light-mode banner variant. Final direction belongs to Algorant.", "rsvg-convert renders inspected; exact-size favicon variants inspected; one parent MagicDNS session HTTP200 check. git diff --exit-code clean; artifacts only under ignored .pi/proposals/tandem-wordmark/. No tracked code/assets, site build, deployment, account or runtime setting changes."]
  updatedAt: "2026-11-13T17:03:50Z"
createdAt: "2026-11-13T16:38:20Z"
updatedAt: "2026-11-13T17:03:50Z"
assignee: "pi:tandem-wordmark"
---

## Description

Algorant's words, verbatim (2026-11-13, relayed by Herdra m437ba28e):

"First is the discrpeancy between the wordmark on the doc site (trytandem.dev) and the repo. Those should agree somewhat. lets start shopping that in context and maybe even simplify it a bit. the cassette tape tandem one is a bit too basic but the trytandem one is a bit too complex. lets meet somewehre in the middle. the invocation should be something of collaborative effort btween two things (humans and agents, namely, but its abstract). this is one angle and something we can begin delegating independently. the next thing would be how I expect the docs site layout to be. If I go to any random documentation site for a project, say its hokum or mrboxington https://mr-boxington.jdx.dev/ https://hokum.app/ and they have a nice basic but eye catching landing page, and tehn extensive but simple documentation setup. what are they using, and how do they stand it up?"

## Track 1 — mockups only
Show both current marks side by side in their real contexts: the README's cassette-tape banner and trytandem.dev's header mark. Explore 3–5 numbered middle-ground options, simpler than the site's and richer than the cassette; carry the abstract idea of two things working together (humans and agents, without literal person/robot symbols). Show EACH option in a README banner, docs-site header, and at small favicon sizes. Use current repo-style rules and verdigris family, lowercase tandem. One post in a fresh Sideshow session named exactly 'tandem wordmark'.

Do not change or build anything into the tracked repo or live site. Put temporary proposals only under ignored .pi/proposals/tandem-wordmark/ or another explicitly disposable local artifact directory; never overwrite the real marks. No commits, pushes, deployment, dependency/tool installation or unrelated cleanup. Taste/final adoption remain Algorant's choice. Send the completed post and artifact evidence to the starter; starter reports to Herdra.
