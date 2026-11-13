---
id: task-111
uid: 84b237b4-8a35-4b0d-8891-1b6777b2a92d
type: task
kind: "research"
title: "Research landing pages and docs stacks for tandem"
state: todo
priority: "medium"
effort: "medium"
references: ["task-109"]
relatedFiles: ["site/", "docs/"]
tags: ["docs", "research"]
accord:
  status: "ready"
  acceptance: ["Both reference sites' framework/theme/hosting/deploy and landing/docs split are evidenced from live source and public repos, with source commits/versions and uncertainty explicitly stated.", "The current tandem stack is accurately compared with one or two realistic routes, including estimated effort and source-checked free/self-hosted or hosted-tier constraints.", "One neutral findings post is published in a new 'tandem docs site' Sideshow session, with no implementation changes."]
  validation: ["$ git diff --exit-code"]
  updatedAt: "2026-11-13T16:38:22Z"
createdAt: "2026-11-13T16:38:22Z"
updatedAt: "2026-11-13T16:38:22Z"
---

## Description

Algorant's words, verbatim (2026-11-13, relayed by Herdra m437ba28e):

"First is the discrpeancy between the wordmark on the doc site (trytandem.dev) and the repo. Those should agree somewhat. lets start shopping that in context and maybe even simplify it a bit. the cassette tape tandem one is a bit too basic but the trytandem one is a bit too complex. lets meet somewehre in the middle. the invocation should be something of collaborative effort btween two things (humans and agents, namely, but its abstract). this is one angle and something we can begin delegating independently. the next thing would be how I expect the docs site layout to be. If I go to any random documentation site for a project, say its hokum or mrboxington https://mr-boxington.jdx.dev/ https://hokum.app/ and they have a nice basic but eye catching landing page, and tehn extensive but simple documentation setup. what are they using, and how do they stand it up?"

## Track 2 — read-only research
Determine what https://mr-boxington.jdx.dev/ and https://hokum.app/ are built with (framework, theme, hosting, deploy) from their live page source AND public repositories, not appearance guesses. Explain how landing page and docs are split. Record the current repo/main commit or version read, sources and any uncertainty; inspect relevant upstream releases/issues where documentation matters.

Then establish trytandem.dev's current stack from this repo and live page. Present 1–2 realistic ways to achieve a basic, eye-catching landing page followed by extensive but simple documentation. For each, give estimated implementation/maintenance effort and concrete free/self-hosted limitations; verify vendor docs for any hosted tier mentioned. No recommendation unless evidence makes it obvious, and label inference versus observed facts.

One concise Sideshow post in a fresh session named exactly 'tandem docs site'; findings first, evidence/source links after. No tracked repo, site, infrastructure, account, dependency or config changes. No build implementation or committing a migration. Research artifacts may be saved only under ignored .pi/research/tandem-docs-site/; no secrets or private app content. Send the completed post and evidence to starter for the separate report to Herdra.
