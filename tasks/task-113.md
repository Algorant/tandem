---
id: task-113
uid: 22818acd-136c-4769-aa57-f398219b9c59
type: task
title: "Prototype tandem homepages with Starlight's landing layout"
state: todo
priority: "medium"
effort: "medium"
references: ["task-109", "task-111", "task-112"]
relatedFiles: ["docs/", "site/src/components/", "site/src/styles/", "site/astro.config.mjs", "site/scripts/sync-docs.mjs"]
tags: ["ui", "docs"]
accord:
  status: "ready"
  acceptance: ["Three or four numbered, distinct options run through real Astro/Starlight landing pages on an isolated prototype branch, not generic static HTML approximations of the docs layout.", "Each option is shown in dark/light desktop and narrow-phone renders plus the first docs page transition, using the selected existing tandem site wordmark.", "A short feature/override explanation accompanies each option in one new post within session 'tandem docs site'; nothing is pushed or deployed, and final homepage implementation waits for Algorant's choice."]
  validation: ["$ just site-build", "$ cd site && bun run check:links", "$ git diff --check"]
  constraints: ["No framework migration, hosting changes, new dependencies or production deployment.", "No generated docs edits or DOM-replacement screenshot mockups; use real source pages and normal Starlight layouts.", "No alternative wordmark; use the existing site mark and coordinate source assets with task-112."]
  updatedAt: "2026-11-13T17:42:20Z"
createdAt: "2026-11-13T17:42:20Z"
updatedAt: "2026-11-13T17:42:20Z"
---

## Description

Algorant, verbatim (2026-11-13, relayed by Herdra m9e1a94f6): "for tandem docs: yes keep astra and starlight, just imitate teh form and simplicity plus styling the homepage in a new way with mockup options"

Decision from task-111: keep Astro and Starlight; no VitePress port. This Task is prototype/mockups only. Final homepage implementation/deployment happens only after Algorant chooses one.

## Required options and contexts
Create 3–4 numbered, genuinely distinct homepage directions, each a REAL Starlight landing page using its splash/landing layout and custom Astro components only where needed. Aim at mr-boxington's form and simplicity: a basic, eye-catching landing page leading into plain, extensive docs. Do not copy its bespoke animation/code volume, wording or brand. Use tandem's EXISTING site wordmark as chosen, high-resolution dark/light; none of the four exploratory replacement marks.

Show each option rendered in dark and light, at desktop and a narrow phone width, plus the first docs page behind it to expose the landing-to-docs transition. Keep product claims truthful and existing documentation accurate; if layout uses illustrative content, label it. Include a short per-option account of the first-party Starlight features, components or explicit overrides it needs, not hacks or browser DOM surgery. Distinct layouts, not only colour swaps.

Use an isolated branch/worktree; no push, production deploy, main-branch edit, new framework, dependency upgrade campaign or hosting change. Source prototype pages live under docs/ and are built normally; do not hand-edit generated site/src/content/docs/. If MDX is needed, minimally extend the existing sync step to copy MDX source as well rather than bypassing the source ownership boundary. Keep overrides/components scoped to prototypes so docs pages retain their ordinary layout.

Publish ONE new homepage-options post in the EXISTING 'tandem docs site' session yf_6Zb7WQVc (the neutral research post remains intact). Four required render contexts per option, and a docs transition view, with short comparison notes. Report to starter, who verifies and forwards the link to Herdra. No second manus under this manus; bounded helpers/Workers are allowed. Wordmark adoption(task-112) runs separately; coordinate its canonical asset paths through the starter rather than independently redrawing/copying mark geometry.
