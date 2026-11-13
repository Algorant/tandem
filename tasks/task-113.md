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
  deliveredAt: "2026-11-13T18:05:26Z"
  validation: ["$ just site-build", "$ cd site && bun run check:links", "$ git diff --check"]
  constraints: ["No framework migration, hosting changes, new dependencies or production deployment.", "No generated docs edits or DOM-replacement screenshot mockups; use real source pages and normal Starlight layouts.", "No alternative wordmark; use the existing site mark and coordinate source assets with task-112."]
  summary: "Three real Starlight splash options are built, captured and published; corrected lifecycle/CLI copy, recaptured affected views, and verified the displayed command sequence."
  evidence: ["just site-build: passed; Astro built 11 pages.", "cd site && bun run check:links: passed; 221 internal links across 23 HTML files.", "git diff --check: passed.", "Disposable native CLI verification: tandem init; tandem add task \"Add a greeting\" --acceptance \"README contains the greeting\"; claim; deliver with summary/evidence; complete; task-1 appeared in Logs.", "Corrected lifecycle copy: plan → claim (recorded owner visible) → deliver, with optional review for an unresolved criterion → archive. Logs is a location, not a Task state. No absolute no-overlap guarantee.", "Corrected Obscura finding: the earlier screenshot-width-only inference was wrong; actual viewport emulation does reflow (390×844 and phone media query true). Obscura blockers are its closed-dialog paint and dropped grid declaration; affected option 1/2 views were recaptured in approved headless Chromium.", "Bounded Chromium rerun: 8 affected option 1/2 views, real built pages, each actual viewport/theme/DPR matched requested 1440×900 or 390×844, dark/light, DPR 1; no broken images. Browser and preview server stopped, disposable profile/temp captures removed.", "Sideshow post updated in place in session yf_6Zb7WQVc: DpjbX4ohWcw. Original research post unchanged.", "Branch starlight-homepage-options; commits d6baa6e and fd6641c. No push, merge or deploy."]
  filesChanged: ["docs/homepage/option-1.md", "site/src/components/homepage/RepoBoard.astro", "site/src/components/homepage/HomepageMark.astro", "site/src/components/homepage/QuietPage.astro", "site/scripts/sync-docs.mjs", "site/tsconfig.json", "docs/homepage/option-2.mdx", "docs/homepage/option-3.mdx"]
  updatedAt: "2026-11-13T18:05:26Z"
createdAt: "2026-11-13T17:42:20Z"
updatedAt: "2026-11-13T18:05:26Z"
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
Renderer evidence from manus m3774208b: Obscura0.2.6 screenshot width390 only crops while innerWidth1280/innerHeight720 and phone mediaquery remain unchanged; it also paints closed Starlight search dialogs over the header. A finite installed-Chromium headless capture run is approved under the documented pixel-exact publication/rendering-gap exception: fresh disposable profile, Node built-in WebSocket/CDP for real1440×900 and390×844 viewport/media settings,13 requested views, loopback-only debugging, actual viewport/DPR/theme recorded, no DOM surgery or screenshot substitution, no automation dependencies/visible browser/user profile. Close/reap browser and stop preview on completion.

Minimal MDX source-copy support and @site alias in site/tsconfig.json are allowed source wiring for the prototype components; generated docs remain untouched. No hosting, framework or production change.