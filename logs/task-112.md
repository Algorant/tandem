---
id: task-112
uid: dde15def-367b-424f-b30e-986b08beb544
type: task
title: "Use the current tandem site wordmark consistently in README and header"
priority: "medium"
effort: "medium"
references: ["task-110"]
relatedFiles: ["README.md", "docs/brand/", "docs/banner.svg", "docs/index.md", "site/astro.config.mjs", "site/src/assets/", "site/public/", "site/src/styles/verdigris.css", "site/scripts/sync-docs.mjs"]
tags: ["ui", "docs"]
accord:
  status: "accepted"
  acceptance: ["The existing site mark is the current identity, with authoritative dark/light SVG assets consistently used in README, docs header, favicon and social preview; no active cassette usage or proposed replacement is adopted.", "A plain docs/brand/ folder retains current dark/light source SVGs and high-res renders plus the original comparison and four exploration boards, with short file descriptions/current-versus-historical labels and no brand-generator scripts.", "GitHub README uses dark/light picture selection and the docs header renders the matching existing wordmark in both themes; other visible repository artifacts share the docs styling.", "After commit/push and normal docs deployment, real github.com README and live trytandem.dev header are checked in dark/light and their before/after appears in the existing tandem wordmark Sideshow session."]
  claimedAt: "2026-11-13T17:41:11Z"
  deliveredAt: "2026-11-13T18:23:55Z"
  validation: ["$ just site-build", "$ cd site && bun run check:links", "$ git diff --check"]
  constraints: ["No alternate wordmark, site redesign, new hosting, new runtime service or unrelated cleanup.", "No second human approval gate: Algorant approved this exact current-mark adoption.", "No screenshot enlargement as an asset; keep text/name treatment faithful and SVG output crisp."]
  summary: "Approved existing site identity adopted across README/site banners, favicon/social artwork and retained references; merged/pushed/deployed, real dark/light before/after checked, and Algorant's manual GitHub preview upload independently verified."
  evidence: ["Main commits a568a0b,29c19e3,dc0aecd,3676f64; normal docs deployments43479403288 and43481032582 bothsuccess. No new generator/dependency/framework/hosting.", "Parent source/geometry/reference-byte review passed; independent build176links/20HTML and diff checks passed. Original current renders/comparison/four exploration PNGs retained; canonical SVG pair unchanged; docs/brand excluded from public collection.", "Genuine LIVE GitHub README light/dark loaded matching1280×320 banner currentSrc; live trytandem.dev header resolved light/dark canonicalmarks,24px600name, unchanged headerpixels. Parent inspected all4boundedAFTERframes;8before/afterframes posted in5omIQ5rDcnU, sessionc4JYCfBnD8Q.", "Algorant uploaded the supplied1280×640/90989B socialPNG himself. Parent publicHTML og:image and twitter:image agree with GraphQL openGraphImageUrl; usesCustomOpenGraphImage=true. Downloadedpubliccard pixel comparison AE=0. Parent revised same adoptionpost with that fact and corrected oldlandingvsoption2attribution.", "Origin updated socialPNG confirmed; CloudflarecanonicalURL still caches prior1200×630 same-artworkexport temporarily(max-age14400). No cache/infrastructure changes. Current wordmark identity correct either export. Headless capture ownedprocesses andObscura closed per delivery."]
  updatedAt: "2026-11-13T18:24:02Z"
createdAt: "2026-11-13T17:41:05Z"
updatedAt: "2026-11-13T18:24:02Z"
assignee: "pi:matching-site-wordmark"
archivedAt: "2026-11-13T18:24:02Z"
resolution:
  outcome: "completed"
  note: "Merged by worker_finish: matching-site-wordmark → main at 3676f6413cc895e6adc8bce02df3481b6239d388."
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

## Integration ordering approved by Herdra
Herdra m7a25d684 explicitly authorizes starter to merge with Worktrunk --no-remove after source checks, then push/normal deploy/live dark+light verification, and only then worker_finish to accept/archive/close/remove the already-integrated child. If worker_finish refuses that state, stop and report; no workaround. This keeps Task completion after the actual requested outcome.

Existing sync-docs.mjs may exclude reference-only docs/brand/ from the public documentation collection; that is needed so its reference README does not become a docs page, not a new brand-generator pipeline.

## GitHub social-preview operator boundary and final verification
Algorant chose manual upload, verbatim via Herdra m0fc4e0ea/ma7b94122: "ok yes I will upload it put the file I'm going to upload in a place I can easliy get to and then tell me the instructions". Starter exported the same artwork at1280×640 (uniform scale, outer-background-only crop), updated source PNG/reference dimensions in3676f64, and copied90,989bytes to ~/Downloads/tandem-social-preview.png. No agent login/upload.
Algorant then confirmed via Herdra m2f989d16: "Ok looks like its done". Parent verified public og:image and twitter:image point to https://repository-images.githubusercontent.com/1281279955/fec1e144-0325-4bca-968a-589b699e95d6; GraphQL usesCustomOpenGraphImage=true. Downloaded1280×640 card is pixel-identical (ImageMagick AE=0) to the suppliedPNG. Normal docs deploy3676f64/run43481032582 succeeded. Origin serves the updated1280PNG; canonical Cloudflarecache still carries the earlier1200×630 export of the SAME artwork (max-age14400), not an old identity. No infrastructure/configuration changes made.
Parent reviewed genuine dark/light GitHub README and live header frames and corrected adoption-post attribution: the current old docs-layout landing has only task112’s banner swap; selected task113homepage is NOT deployed. Post5omIQ5rDcnU in existing sessionc4JYCfBnD8Q includes8before/afterframes plus the verified uploadedcard.