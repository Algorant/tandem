---
id: task-113
uid: 22818acd-136c-4769-aa57-f398219b9c59
type: task
title: "Prototype tandem homepages with Starlight's landing layout"
state: "in-progress"
priority: "medium"
effort: "medium"
references: ["task-109", "task-111", "task-112"]
relatedFiles: ["docs/", "site/src/components/", "site/src/styles/", "site/astro.config.mjs", "site/scripts/sync-docs.mjs", "site/tsconfig.json"]
tags: ["ui", "docs"]
accord:
  status: "delivered"
  acceptance: ["Three or four numbered, distinct options run through real Astro/Starlight landing pages on an isolated prototype branch, not generic static HTML approximations of the docs layout.", "Each option is shown in dark/light desktop and narrow-phone renders plus the first docs page transition, using the selected existing tandem site wordmark.", "A short feature/override explanation accompanies each option in one new post within session 'tandem docs site'; nothing is pushed or deployed, and final homepage implementation waits for Algorant's choice."]
  claimedAt: "2026-11-13T17:42:35Z"
  deliveredAt: "2026-11-13T18:11:19Z"
  validation: ["$ just site-build", "$ cd site && bun run check:links", "$ git diff --check"]
  constraints: ["No framework migration, hosting changes, new dependencies or production deployment.", "No generated docs edits or DOM-replacement screenshot mockups; use real source pages and normal Starlight layouts.", "No alternative wordmark; use the existing site mark and coordinate source assets with task-112."]
  summary: "Three real Starlight landing-page prototypes published as numbered choices with dark/light desktop/phone renders and the shared Quickstart transition. Source and command/lifecycle copy reviewed and corrected; branch remains isolated, unpushed and undeployed for Algorant's choice."
  evidence: ["Local prototype branch starlight-homepage-options has commits d6baa6e and fd6641c, clean. docs/homepage contains1 Markdown +2 MDX splash pages; scoped RepoBoard/QuietPage components and shared original-mark component; only minimal MDX copy support/@site alias added. docs/index, regular docs and verdigris.css remain untouched.", "One options post DpjbX4ohWcw in existing 'tandem docs site' session yf_6Zb7WQVc; currentversion13,13 imageframes: each option dark/light at real1440×900 desktop and390×844 phone, plus Quickstart docs transition. Exact viewport/theme/DPR1 recorded by native Chromium; HTTP200 session check.", "Parent independently reviewed all8 source files, source diffs and bounded desktop/phone renders for all3 layouts plus Quickstart. Source copy corrections: plan/claim/deliver-or-optional-review/archive are lifecycle steps, not4storedstates; owner visible rather than false no-overlap guarantee; Task CLI includes acceptance, no false command count. Real disposable native init/add/claim/deliver/complete sequence verified by manus; sandbox removed.", "Parent reran just site-build (11pages), check:links (221links/23HTML), git diff --check at fd6641c: allpass. Source was not merged/pushed/deployed; only task112current-wordmark source independently landed on main.", "Renderer correction preserved: Obscura screenshot DOES reflow during capture; the earlier opposite inference was wrong. Confirmed blockers are closed-dialog painting and dropped grid CSS. Bounded installed headless Chromium publication-capture exception used, no Playwright/Puppeteer/Selenium, no userprofile/visiblewindow/DOMsurgery/newdeps; browser and preview closed."]
  filesChanged: ["docs/homepage/option-1.md", "site/src/components/homepage/RepoBoard.astro", "site/src/components/homepage/HomepageMark.astro", "site/src/components/homepage/QuietPage.astro", "site/scripts/sync-docs.mjs", "site/tsconfig.json", "docs/homepage/option-2.mdx", "docs/homepage/option-3.mdx"]
  updatedAt: "2026-11-13T18:11:19Z"
createdAt: "2026-11-13T17:42:20Z"
updatedAt: "2026-11-13T18:11:19Z"
assignee: "pi:starlight-homepage-options"
---

## Description

Algorant, verbatim (2026-11-13, relayed by Herdra m9e1a94f6): "for tandem docs: yes keep astra and starlight, just imitate teh form and simplicity plus styling the homepage in a new way with mockup options"

Decision from task-111: keep Astro and Starlight; no VitePress port. This Task is prototype/mockups only. Final homepage implementation/deployment happens only after Algorant chooses one.

## Required options and contexts
Create 3–4 numbered, genuinely distinct homepage directions, each a REAL Starlight landing page using its splash/landing layout and custom Astro components only where needed. Aim at mr-boxington's form and simplicity: a basic, eye-catching landing page leading into plain, extensive docs. Do not copy its bespoke animation/code volume, wording or brand. Use tandem's EXISTING site wordmark as chosen, high-resolution dark/light; none of the four exploratory replacement marks.

Show each option rendered in dark and light, at desktop and a narrow phone width, plus the first docs page behind it to expose the landing-to-docs transition. Keep product claims truthful and existing documentation accurate; if layout uses illustrative content, label it. Include a short per-option account of the first-party Starlight features, components or explicit overrides it needs, not hacks or browser DOM surgery. Distinct layouts, not only colour swaps.

Use an isolated branch/worktree; no push, production deploy, main-branch edit, new framework, dependency upgrade campaign or hosting change. Source prototype pages live under docs/ and are built normally; do not hand-edit generated site/src/content/docs/. If MDX is needed, minimally extend the existing sync step to copy MDX source as well rather than bypassing the source ownership boundary. Keep overrides/components scoped to prototypes so docs pages retain their ordinary layout.

Publish ONE new homepage-options post in the EXISTING 'tandem docs site' session yf_6Zb7WQVc (the neutral research post remains intact). Four required render contexts per option, and a docs transition view, with short comparison notes. Report to starter, who verifies and forwards the link to Herdra. No second manus under this manus; bounded helpers/Workers are allowed. Wordmark adoption(task-112) runs separately; coordinate its canonical asset paths through the starter rather than independently redrawing/copying mark geometry.


## Internal verification approval
Renderer evidence correction: the earlier inference from post-capture innerWidth was wrong—Obscura screenshot capture DOES reflow at a requested width, then restores its prior viewport. The confirmed Obscura0.2.6 gaps are painting closed Starlight search dialogs and dropping the theme’s auto-fit/minmax/min grid declaration. Those concrete rendering errors—not an alleged lack of phone reflow—justify the bounded Chromium publication captures. A finite installed-Chromium headless capture run is approved under the documented pixel-exact publication/rendering-gap exception: fresh disposable profile, Node built-in WebSocket/CDP for real1440×900 and390×844 viewport/media settings,13 requested views, loopback-only debugging, actual viewport/DPR/theme recorded, no DOM surgery or screenshot substitution, no automation dependencies/visible browser/user profile. Close/reap browser and stop preview on completion.

Minimal MDX source-copy support and @site alias in site/tsconfig.json are allowed source wiring for the prototype components; generated docs remain untouched. No hosting, framework or production change.