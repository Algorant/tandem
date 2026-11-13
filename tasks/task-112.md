---
id: task-112
uid: dde15def-367b-424f-b30e-986b08beb544
type: task
title: "Use the current tandem site wordmark consistently in README and header"
state: todo
priority: "medium"
effort: "medium"
references: ["task-110"]
relatedFiles: ["README.md", "docs/banner.svg", "docs/index.md", "site/astro.config.mjs", "site/src/assets/", "site/public/", "site/src/styles/verdigris.css", "site/scripts/"]
tags: ["ui", "docs"]
accord:
  status: "ready"
  acceptance: ["The current site mark remains unchanged in identity, with one authoritative SVG geometry source and derived dark/light assets shared by README and docs header; no cassette banner or independently redrawn replacement remains in those public surfaces.", "GitHub README uses dark/light picture selection and docs header renders the matching current wordmark in both themes; favicon is geometrically consistent or explicitly retained if already on-brand.", "After commit/push and the normal docs deploy, real github.com README and live trytandem.dev header are checked in dark and light, and their before/after appears in the existing tandem wordmark Sideshow session."]
  validation: ["$ just site-build", "$ cd site && bun run check:links", "$ git diff --check"]
  constraints: ["No alternate wordmark, site redesign, new hosting, new runtime service or unrelated cleanup.", "No second human approval gate: Algorant approved this exact current-mark adoption.", "No screenshot enlargement as an asset; keep text/name treatment faithful and SVG output crisp."]
  updatedAt: "2026-11-13T17:41:05Z"
createdAt: "2026-11-13T17:41:05Z"
updatedAt: "2026-11-13T17:41:05Z"
---

## Description

Algorant’s exact words, via Herdra m4b71009c (2026-11-13): "for tandem wordmark: high def version of the docs site wordmark, can you show me that in dark and light?" and "I just want the repo and the docs site to match with that, basically".

Decision: preserve the EXISTING trytandem.dev mark (interlaced strands), not any new option from task-110. The exploration closes separately. This specific implementation, commit/push, normal docs deployment and live dark/light verification are authorized; no second taste-acceptance gate.

## Scope
Make one authoritative high-resolution SVG source of the current site wordmark with dark/light variants. Reuse that source in the GitHub README banner via a real dark/light <picture> and in the docs header; favicon from the same geometry if the current one is off-brand. Do not maintain separately redrawn geometry copies. A small build-time asset derivation is acceptable if needed; no new general asset framework, dependency campaign or runtime service. Existing site mark is already SVG; do not upscale a screenshot. Preserve the current shape/type treatment, not a typography redesign. If the site uses SVG mark plus HTML name, account for that explicitly and match the rendered current lockup.

Cover remaining cassette-banner uses on the docs landing page as needed so the public faces do not retain the discrepancy. Do not redesign the docs layout/content or change hosting. Read-only artifacts from wordmark study are in the main checkout’s ignored .pi/proposals/tandem-wordmark/ and session c4JYCfBnD8Q; do not rely on them being copied into a worktree.

Capture before and after for real GitHub README and live trytandem.dev header in BOTH dark/light themes; add one adoption before/after post to the SAME "tandem wordmark" session c4JYCfBnD8Q (a new post for this meaningful step is fine). Clearly label reconstructed local previews versus live browser captures. No screenshots of other projects/private apps.

Use an isolated implementation worktree; report a concise internal plan to the starter before editing. Do not ask Algorant again for this approved change. Deliver verified scoped work to starter for integration/push/deploy; starter owns main and final acceptance/completion.
