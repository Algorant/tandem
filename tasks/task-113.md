---
id: task-113
uid: 22818acd-136c-4769-aa57-f398219b9c59
type: task
title: "Preview option 2 with real tandem images before shipping"
state: "in-progress"
priority: "medium"
effort: "medium"
references: ["task-109", "task-111", "task-112"]
relatedFiles: ["docs/", "site/src/components/", "site/src/styles/", "site/astro.config.mjs", "site/scripts/sync-docs.mjs", "site/tsconfig.json", "docs/index.mdx", "site/public/images/"]
tags: ["ui", "docs"]
accord:
  status: "rework"
  acceptance: ["Option2’s actual landing layout includes useful real images of tandem in use, captured from the real app in an isolated nonpersonal sample workspace, with both themes where meaningful and established capture provenance.", "The revised image-bearing homepage is rendered at real dark/light desktop/phone viewports with the mocked Quickstart transition, and published as a clearly labelled revised preview in the existing tandem docs site session.", "Algorant explicitly approves that revised preview before any production merge/push/deploy; after approval, live landing and first docs page are verified in both themes/widths before deferred worker_finish."]
  claimedAt: "2026-11-13T17:42:35Z"
  deliveredAt: "2026-11-13T18:11:19Z"
  validation: ["$ just site-build", "$ cd site && bun run check:links", "$ git diff --check"]
  constraints: ["No production merge/push/deploy until Algorant approves the revised image-bearing preview; earlier no-second-acceptance is superseded.", "Preserve selected option2, normal docs transition, canonical wordmark and docs/brand sync exclusion; no unselected routes, framework/hosting change or new dependency campaign.", "Use real app/sample workspace, no personal data, fake UI or synthetic terminal screenshots; no restart/edit of live user boards or terminals."]
  summary: "Three real Starlight landing-page prototypes published as numbered choices with dark/light desktop/phone renders and the shared Quickstart transition. Source and command/lifecycle copy reviewed and corrected; branch remains isolated, unpushed and undeployed for Algorant's choice."
  evidence: ["Local prototype branch starlight-homepage-options has commits d6baa6e and fd6641c, clean. docs/homepage contains1 Markdown +2 MDX splash pages; scoped RepoBoard/QuietPage components and shared original-mark component; only minimal MDX copy support/@site alias added. docs/index, regular docs and verdigris.css remain untouched.", "One options post DpjbX4ohWcw in existing 'tandem docs site' session yf_6Zb7WQVc; currentversion13,13 imageframes: each option dark/light at real1440×900 desktop and390×844 phone, plus Quickstart docs transition. Exact viewport/theme/DPR1 recorded by native Chromium; HTTP200 session check.", "Parent independently reviewed all8 source files, source diffs and bounded desktop/phone renders for all3 layouts plus Quickstart. Source copy corrections: plan/claim/deliver-or-optional-review/archive are lifecycle steps, not4storedstates; owner visible rather than false no-overlap guarantee; Task CLI includes acceptance, no false command count. Real disposable native init/add/claim/deliver/complete sequence verified by manus; sandbox removed.", "Parent reran just site-build (11pages), check:links (221links/23HTML), git diff --check at fd6641c: allpass. Source was not merged/pushed/deployed; only task112current-wordmark source independently landed on main.", "Renderer correction preserved: Obscura screenshot DOES reflow during capture; the earlier opposite inference was wrong. Confirmed blockers are closed-dialog painting and dropped grid CSS. Bounded installed headless Chromium publication-capture exception used, no Playwright/Puppeteer/Selenium, no userprofile/visiblewindow/DOMsurgery/newdeps; browser and preview closed."]
  filesChanged: ["docs/homepage/option-1.md", "site/src/components/homepage/RepoBoard.astro", "site/src/components/homepage/HomepageMark.astro", "site/src/components/homepage/QuietPage.astro", "site/scripts/sync-docs.mjs", "site/tsconfig.json", "docs/homepage/option-2.mdx", "docs/homepage/option-3.mdx"]
  note: "Algorant chose option2 and authorized shipping via Herdra me96b22a7. This is the approved production phase after successful prototypes, not rejection of the mockups; deliver selected homepage/transition for starter deployment and live verification."
  updatedAt: "2026-11-13T18:22:39Z"
createdAt: "2026-11-13T17:42:20Z"
updatedAt: "2026-11-13T18:22:39Z"
assignee: "pi:starlight-homepage-options"
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

## Latest Algorant direction — real product imagery and a new approval gate
Algorant, verbatim (2026-11-13 via Herdra m68db7fed): "the current tandem landing page looks like shit. where are the images? are they still to come?"

The currently live page is the old docs-layout homepage with task-112’s new wordmark. Option2 is NOT deployed; its selected mockup contains no product screenshots. Add real images showing tandem in use within option2’s actual landing layout, in the spirit of mr-boxington/hokum.app: real TUI board/task/CLI screenshots or a short real animated demo, from the actual app on an isolated sample workspace without personal data; capture both themes where meaningful. No fake app or synthetic terminal presentation passed off as real output.

Show the revised landing in dark/light at desktop/phone widths, plus the ordinary Quickstart transition, in the same tandem docs site session yf_6Zb7WQVc. Publish an explicit new revised-preview milestone so the historical selected mockup is not confused with this new result. Algorant must approve the image-bearing preview before ANY production merge/push/deploy. This latest reaction OVERRIDES the earlier no-second-acceptance instruction. Deferred finish ordering still applies AFTER that approval: merge → normal deploy → real live checks/post → worker_finish. No main changes or deployment for this revision until Algorant says yes.