---
id: task-112
uid: dde15def-367b-424f-b30e-986b08beb544
type: task
title: "Use the current tandem site wordmark consistently in README and header"
state: "in-progress"
priority: "medium"
effort: "medium"
references: ["task-110"]
relatedFiles: ["README.md", "docs/brand/", "docs/banner.svg", "docs/index.md", "site/astro.config.mjs", "site/src/assets/", "site/public/", "site/src/styles/verdigris.css"]
tags: ["ui", "docs"]
accord:
  status: "claimed"
  acceptance: ["The existing site mark is the current identity, with authoritative dark/light SVG assets consistently used in README, docs header, favicon and social preview; no active cassette usage or proposed replacement is adopted.", "A plain docs/brand/ folder retains current dark/light source SVGs and high-res renders plus the original comparison and four exploration boards, with short file descriptions/current-versus-historical labels and no brand-generator scripts.", "GitHub README uses dark/light picture selection and the docs header renders the matching existing wordmark in both themes; other visible repository artifacts share the docs styling.", "After commit/push and normal docs deployment, real github.com README and live trytandem.dev header are checked in dark/light and their before/after appears in the existing tandem wordmark Sideshow session."]
  claimedAt: "2026-11-13T17:41:11Z"
  validation: ["$ just site-build", "$ cd site && bun run check:links", "$ git diff --check"]
  constraints: ["No alternate wordmark, site redesign, new hosting, new runtime service or unrelated cleanup.", "No second human approval gate: Algorant approved this exact current-mark adoption.", "No screenshot enlargement as an asset; keep text/name treatment faithful and SVG output crisp."]
  updatedAt: "2026-11-13T17:46:46Z"
createdAt: "2026-11-13T17:41:05Z"
updatedAt: "2026-11-13T17:51:52Z"
assignee: "pi:matching-site-wordmark"
---

## Description

Algorant’s exact words, via Herdra m4b71009c (2026-11-13): "for tandem wordmark: high def version of the docs site wordmark, can you show me that in dark and light?" and "I just want the repo and the docs site to match with that, basically".

Decision: preserve the EXISTING trytandem.dev mark (interlaced strands), not any new option from task-110. The exploration closes separately. This specific implementation, commit/push, normal docs deployment and live dark/light verification are authorized; no second taste-acceptance gate.

## Scope
Make one authoritative high-resolution SVG source of the current site wordmark with dark/light variants. Reuse that source in the GitHub README banner via a real dark/light <picture> and in the docs header; favicon from the same geometry if the current one is off-brand. Do not maintain separately redrawn geometry copies. No tracked brand-generator scripts are wanted: keep the source SVG variants and rendered reference files directly, with no new asset framework, dependency campaign or runtime service. Existing site mark is already SVG; do not upscale a screenshot. Preserve the current shape/type treatment, not a typography redesign. If the site uses SVG mark plus HTML name, account for that explicitly and match the rendered current lockup.

Cover remaining cassette-banner uses on the docs landing page as needed so the public faces do not retain the discrepancy. Do not redesign the docs layout/content or change hosting. Read-only artifacts from wordmark study are in the main checkout’s ignored .pi/proposals/tandem-wordmark/ and session c4JYCfBnD8Q; do not rely on them being copied into a worktree.

Capture before and after for real GitHub README and live trytandem.dev header in BOTH dark/light themes; add one adoption before/after post to the SAME "tandem wordmark" session c4JYCfBnD8Q (a new post for this meaningful step is fine). Clearly label reconstructed local previews versus live browser captures. No screenshots of other projects/private apps.

Use an isolated implementation worktree; report a concise internal plan to the starter before editing. Do not ask Algorant again for this approved change. Deliver verified scoped work to starter for integration/push/deploy; starter owns main and final acceptance/completion.


## Expanded cohesion and reference scope — latest instruction
Algorant, verbatim (2026-11-13, via Herdra m56479c4d): "yes I like the tandem direction please move the readme or repo artifacts over to be cohesive with the documentation and save those images in some place on the repo for future reference"

Besides the README/header, align other visible repo artifacts with the existing docs look: README colours/badges if present, GitHub/social preview image, favicon, and any remaining ACTIVE cassette artwork. Keep one current wordmark source/identity used everywhere, not four proposed replacements. Preserve existing docs typography/palette rather than redesigning the brand.

Keep reference assets together in a plain tracked folder, docs/brand/: current dark/light source SVGs, high-resolution current renders, and task-110’s comparison board showing today’s two marks side by side plus all four alternative boards. Include a short README with one-line file descriptions and an unambiguous statement that the existing site mark is current and the alternatives are historical/not adopted. No generator scripts in the repo for this. Historical comparison boards may depict the cassette as explicitly requested reference; remove active cassette assets/usages, not history from the retained comparison.

Commit/push and the normal docs deployment are explicitly authorized, with real GitHub README and live header checked in both themes and before/after in the existing wordmark session. Do not broaden into homepage redesign (that is prototype-only task-113).

## Approved minimal implementation contract
Internal plan approved via mc2183b9d, no extra human gate. Existing site/src/assets/tandem-mark.svg and tandem-mark-light.svg remain the canonical header geometry family. Active banner pair exists ONLY at site/public/banner-dark.svg and banner-light.svg: README picture references those very repo files, docs landing references their served URLs. Do not add duplicate docs/banner-dark/light copies. Remove the two old cassette banner files.

The plain docs/brand/ folder keeps byte-identical ORIGINAL task-110 current-lockup dark/light PNGs (not substituted new banner renders), original comparison board and four alternative boards; source SVG reference copies are explicitly reference-only. No brand generators/package hooks.

Live capture constraint: Obscura was observed painting CLOSED GitHub/Starlight dialogs as open over the target area. The existing browsing-skill one-shot headless Chromium CLI exception is approved solely for faithful before/after publication images with real dark/light media preferences, fresh disposable profiles and immediate process cleanup. No user's browser/profile, visible window, installs, Playwright/Puppeteer/Selenium or DOM-hidden fake captures. This is a concrete renderer-gap check, not a general new browser workflow.