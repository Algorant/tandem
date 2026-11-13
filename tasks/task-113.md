---
id: task-113
uid: 22818acd-136c-4769-aa57-f398219b9c59
type: task
title: "Ship the selected Starlight homepage and docs transition"
state: "validation"
priority: "medium"
effort: "medium"
references: ["task-109", "task-111", "task-112"]
relatedFiles: ["docs/", "site/src/components/", "site/src/styles/", "site/astro.config.mjs", "site/scripts/sync-docs.mjs", "site/tsconfig.json"]
tags: ["ui", "docs"]
accord:
  status: "delivered"
  acceptance: ["Production homepage matches Algorant's selected option2 on real Astro/Starlight, using the chosen current wordmark and the mocked landing-to-Quickstart docs transition; options1/3 and preview-only routes are not deployed.", "After merge/push and normal docs deployment, LIVE landing and first docs page are checked in dark/light at real desktop and phone viewport widths, with actual running theme/layout confirmed and no renderer substitutions or DOM surgery.", "The same tandem docs site session contains the live after-shots; Task completion follows live verification using the approved deferred worker_finish ordering."]
  claimedAt: "2026-11-13T17:42:35Z"
  deliveredAt: "2026-11-13T18:11:19Z"
  validation: ["$ just site-build", "$ cd site && bun run check:links", "$ git diff --check"]
  constraints: ["Only selected option2 may become production; remove preview-only routes/components and production noindex flags.", "Preserve the wordmark task112 assets and docs/brand sync exclusion; no framework migration, hosting changes or new dependencies.", "No second human acceptance needed for the approved design; verify actual published result before archive."]
  summary: "Three real Starlight landing-page prototypes published as numbered choices with dark/light desktop/phone renders and the shared Quickstart transition. Source and command/lifecycle copy reviewed and corrected; branch remains isolated, unpushed and undeployed for Algorant's choice."
  evidence: ["Local prototype branch starlight-homepage-options has commits d6baa6e and fd6641c, clean. docs/homepage contains1 Markdown +2 MDX splash pages; scoped RepoBoard/QuietPage components and shared original-mark component; only minimal MDX copy support/@site alias added. docs/index, regular docs and verdigris.css remain untouched.", "One options post DpjbX4ohWcw in existing 'tandem docs site' session yf_6Zb7WQVc; currentversion13,13 imageframes: each option dark/light at real1440×900 desktop and390×844 phone, plus Quickstart docs transition. Exact viewport/theme/DPR1 recorded by native Chromium; HTTP200 session check.", "Parent independently reviewed all8 source files, source diffs and bounded desktop/phone renders for all3 layouts plus Quickstart. Source copy corrections: plan/claim/deliver-or-optional-review/archive are lifecycle steps, not4storedstates; owner visible rather than false no-overlap guarantee; Task CLI includes acceptance, no false command count. Real disposable native init/add/claim/deliver/complete sequence verified by manus; sandbox removed.", "Parent reran just site-build (11pages), check:links (221links/23HTML), git diff --check at fd6641c: allpass. Source was not merged/pushed/deployed; only task112current-wordmark source independently landed on main.", "Renderer correction preserved: Obscura screenshot DOES reflow during capture; the earlier opposite inference was wrong. Confirmed blockers are closed-dialog painting and dropped grid CSS. Bounded installed headless Chromium publication-capture exception used, no Playwright/Puppeteer/Selenium, no userprofile/visiblewindow/DOMsurgery/newdeps; browser and preview closed."]
  filesChanged: ["docs/homepage/option-1.md", "site/src/components/homepage/RepoBoard.astro", "site/src/components/homepage/HomepageMark.astro", "site/src/components/homepage/QuietPage.astro", "site/scripts/sync-docs.mjs", "site/tsconfig.json", "docs/homepage/option-2.mdx", "docs/homepage/option-3.mdx"]
  updatedAt: "2026-11-13T18:13:43Z"
createdAt: "2026-11-13T17:42:20Z"
updatedAt: "2026-11-13T18:13:43Z"
assignee: "pi:starlight-homepage-options"
validation.criterion: "A short feature/override explanation accompanies each option in one new post within session 'tandem docs site'; nothing is pushed or deployed, and final homepage implementation waits for Algorant's choice."
validation.note: "Technical source/render/build/link checks verified. Only the product/taste choice between3 homepage directions remains Algorant's; prototype branch is intentionally not merged or deployed. Posted options and transition frames are ready for that choice."
validation.requestedAt: "2026-11-13T18:11:20Z"
validation.reviewer: "Algorant"
---

## Description

Algorant, verbatim (2026-11-13, relayed by Herdra m9e1a94f6): "for tandem docs: yes keep astra and starlight, just imitate teh form and simplicity plus styling the homepage in a new way with mockup options"

Decision from task-111: keep Astro and Starlight; no VitePress port. The initial phase was prototype/mockups only. Algorant has now chosen option2 and explicitly authorized its production implementation/deployment; the exact decision and latest scope are recorded below.

## Required options and contexts
Create 3–4 numbered, genuinely distinct homepage directions, each a REAL Starlight landing page using its splash/landing layout and custom Astro components only where needed. Aim at mr-boxington's form and simplicity: a basic, eye-catching landing page leading into plain, extensive docs. Do not copy its bespoke animation/code volume, wording or brand. Use tandem's EXISTING site wordmark as chosen, high-resolution dark/light; none of the four exploratory replacement marks.

Show each option rendered in dark and light, at desktop and a narrow phone width, plus the first docs page behind it to expose the landing-to-docs transition. Keep product claims truthful and existing documentation accurate; if layout uses illustrative content, label it. Include a short per-option account of the first-party Starlight features, components or explicit overrides it needs, not hacks or browser DOM surgery. Distinct layouts, not only colour swaps.

Use an isolated branch/worktree; no push, production deploy, main-branch edit, new framework, dependency upgrade campaign or hosting change. Source prototype pages live under docs/ and are built normally; do not hand-edit generated site/src/content/docs/. If MDX is needed, minimally extend the existing sync step to copy MDX source as well rather than bypassing the source ownership boundary. Keep overrides/components scoped to prototypes so docs pages retain their ordinary layout.

Publish ONE new homepage-options post in the EXISTING 'tandem docs site' session yf_6Zb7WQVc (the neutral research post remains intact). Four required render contexts per option, and a docs transition view, with short comparison notes. Report to starter, who verifies and forwards the link to Herdra. No second manus under this manus; bounded helpers/Workers are allowed. Wordmark adoption(task-112) runs separately; coordinate its canonical asset paths through the starter rather than independently redrawing/copying mark geometry.


## Internal verification approval
Renderer evidence correction: the earlier inference from post-capture innerWidth was wrong—Obscura screenshot capture DOES reflow at a requested width, then restores its prior viewport. The confirmed Obscura0.2.6 gaps are painting closed Starlight search dialogs and dropping the theme’s auto-fit/minmax/min grid declaration. Those concrete rendering errors—not an alleged lack of phone reflow—justify the bounded Chromium publication captures. A finite installed-Chromium headless capture run is approved under the documented pixel-exact publication/rendering-gap exception: fresh disposable profile, Node built-in WebSocket/CDP for real1440×900 and390×844 viewport/media settings,13 requested views, loopback-only debugging, actual viewport/DPR/theme recorded, no DOM surgery or screenshot substitution, no automation dependencies/visible browser/user profile. Close/reap browser and stop preview on completion.

Minimal MDX source-copy support and @site alias in site/tsconfig.json are allowed source wiring for the prototype components; generated docs remain untouched. No hosting, framework or production change.

## Algorant selected option 2 — production authorized
Algorant, verbatim (2026-11-13, via Herdra me96b22a7): "for tandem: I like landing option 2 and go on the new docs transition".

Implement only option2 as the production landing page on Astro/Starlight, with the current site wordmark from task-112 and the normal Quickstart first-docs transition as mocked. None of the other prototypes is selected. Remove preview-only routes/components and noindex flags from the production result; preserve the normal docs page/layout rather than inventing a new transition. Retain the existing sync helper’s docs/brand/ exclusion alongside MDX copying, and keep current production wordmark/social/reference assets intact.

This exact design is approved; no second acceptance gate. The original no-main/no-deploy restriction applied to the mockup phase and is now superseded ONLY for the selected option. Deliver clean source to starter; starter merges with Worktrunk --no-remove, pushes/deploys, verifies LIVE trytandem.dev landing and first docs page in dark/light at desktop/phone widths, posts live after-shots to the SAME session yf_6Zb7WQVc, and then worker_finish accepts/closes/removes the already-integrated child. If finish refuses that state, stop/report. No framework migration, hosting change, new dependency, unselected prototype deployment or unrelated cleanup.