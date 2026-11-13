---
id: task-111
uid: 84b237b4-8a35-4b0d-8891-1b6777b2a92d
type: task
kind: "research"
title: "Research landing pages and docs stacks for tandem"
state: "in-progress"
priority: "medium"
effort: "medium"
references: ["task-109"]
relatedFiles: ["site/", "docs/"]
tags: ["docs", "research"]
accord:
  status: "delivered"
  acceptance: ["Both reference sites' framework/theme/hosting/deploy and landing/docs split are evidenced from live source and public repos, with source commits/versions and uncertainty explicitly stated.", "The current tandem stack is accurately compared with one or two realistic routes, including estimated effort and source-checked free/self-hosted or hosted-tier constraints.", "One neutral findings post is published in a new 'tandem docs site' Sideshow session, with no implementation changes."]
  claimedAt: "2026-11-13T16:38:34Z"
  deliveredAt: "2026-11-13T16:54:44Z"
  validation: ["$ git diff --exit-code"]
  summary: "Read-only docs-site research published as one neutral post (version2) in session 'tandem docs site': source-verified stacks/deploys for mr boxington and hokum, current tandem comparison, two realistic routes with estimated effort and vendor-checked hosting constraints. No implementation or tracked file changes."
  evidence: ["Post H3w7tC9cb9I / session yf_6Zb7WQVc (version2,5 surfaces), checked MagicDNS session URL HTTP200; git diff --exit-code passes.", "mr-boxington: live VitePress1.6.4; public jdx/mr-boxington commit25d6ed35b6d6bbd74cf6d833c0ff05c7fc335375 confirms home-layout frontmatter/custom Vue slots, docs pages, workflow GitHub Pages deployment. hokum: anomalyco/hokum deve2e31b146792da21b9c6c691703433a6d2d72bbe confirms custom Bun HTML/Markdown build and Cloudflare Worker with run_worker_first:true; documented manual deploy/no site-deploy CI in inspected repo, private automation unknown.", "Current tandem a356a45: Astro7.3.3/Starlight0.41.11,7 docs pages, Pagefind, current homepage in docs shell, Actions Pages deployment behind Cloudflare. Latest upstream versions inspected/cited (Astro7.4.11, Starlight0.42.11, VitePress stable1.6.4/pre-release2). No update performed.", "Routes described neutrally: keep Astro/Starlight and use existing landing layout (estimate half-day initial +1–2 days taste rounds), or port7 pages/theme integration to VitePress (estimate2–4 days). Estimates labelled, not benchmarks; framework-independent palette/artwork/content reuse distinguished from framework-specific CSS/config ports.", "Parent verification read pinned package/workflow files for both reference repos, hokum's wrangler config/worker source, Starlight pages docs and Cloudflare static-assets billing docs. One correction pass removed subjective/absolute claims, added clickable commit/doc sources and explicit per-route free/self-host constraints; run_worker_first invocation quota now stated accurately. Third-party issue claims labelled unreproduced.", "Artifacts in ignored .pi/research/tandem-docs-site/; no repo/site implementation, commits, pushes, accounts or infrastructure changes. Final product direction remains Algorant's decision."]
  updatedAt: "2026-11-13T16:54:44Z"
createdAt: "2026-11-13T16:38:22Z"
updatedAt: "2026-11-13T16:54:44Z"
assignee: "pi:tandem-docs-site"
---

## Description

Algorant's words, verbatim (2026-11-13, relayed by Herdra m437ba28e):

"First is the discrpeancy between the wordmark on the doc site (trytandem.dev) and the repo. Those should agree somewhat. lets start shopping that in context and maybe even simplify it a bit. the cassette tape tandem one is a bit too basic but the trytandem one is a bit too complex. lets meet somewehre in the middle. the invocation should be something of collaborative effort btween two things (humans and agents, namely, but its abstract). this is one angle and something we can begin delegating independently. the next thing would be how I expect the docs site layout to be. If I go to any random documentation site for a project, say its hokum or mrboxington https://mr-boxington.jdx.dev/ https://hokum.app/ and they have a nice basic but eye catching landing page, and tehn extensive but simple documentation setup. what are they using, and how do they stand it up?"

## Track 2 — read-only research
Determine what https://mr-boxington.jdx.dev/ and https://hokum.app/ are built with (framework, theme, hosting, deploy) from their live page source AND public repositories, not appearance guesses. Explain how landing page and docs are split. Record the current repo/main commit or version read, sources and any uncertainty; inspect relevant upstream releases/issues where documentation matters.

Then establish trytandem.dev's current stack from this repo and live page. Present 1–2 realistic ways to achieve a basic, eye-catching landing page followed by extensive but simple documentation. For each, give estimated implementation/maintenance effort and concrete free/self-hosted limitations; verify vendor docs for any hosted tier mentioned. No recommendation unless evidence makes it obvious, and label inference versus observed facts.

One concise Sideshow post in a fresh session named exactly 'tandem docs site'; findings first, evidence/source links after. No tracked repo, site, infrastructure, account, dependency or config changes. No build implementation or committing a migration. Research artifacts may be saved only under ignored .pi/research/tandem-docs-site/; no secrets or private app content. Send the completed post and evidence to starter for the separate report to Herdra.
