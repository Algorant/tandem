---
id: task-64
type: task
title: "Polish docs site branding, homepage, and navigation"
priority: "low"
parentId: "task-59"
references: ["decision-1"]
relatedFiles: ["site/astro.config.mjs", "site/src", "docs/index.md"]
tags: ["docs", "site", "design"]
createdAt: "2026-06-29T20:49:37Z"
updatedAt: "2026-07-04T22:52:39Z"
accord:
  status: "accepted"
  assignee: "pi-docs-site-polish"
  claimedAt: "2026-07-04T21:38:22Z"
  deliveredAt: "2026-07-04T22:52:27Z"
  deliverables: ["Sideshow review surface: http://localhost:8228/session/F2iqcu5r07M/s/M6SpmnsVi5o", "Polished homepage: Starlight hero frontmatter, CTAs, feature cards, role entry cards, and current-status note.", "Grouped sidebar/nav: Start here, Core model, Interfaces, Guides, Reference.", "Branding/metadata: Starlight logo asset, refreshed favicon, social card, title delimiter/tagline, Open Graph/Twitter metadata, GitHub social label.", "Canonical edit links: Starlight route middleware points edit links back to `docs/` instead of generated content copies.", "Verdigris CSS polish: hero/card styles, logo glow, link/card transitions, and code block readability tweaks."]
  validation:
    commands: ["Parent inspected Sideshow review surface; user said it looks good, likes the logo and favicon, and accepts overview cards for now.", "Parent reran `git diff --check`: passed for tracked files.", "Parent reran whitespace checks for untracked `site/public/social-card.svg`, `site/src/assets/tandem-mark.svg`, and `site/src/starlight-route-data.ts`: passed.", "Parent reran `cd site && bun run check:docs`: passed; built 13 pages and checked 591 internal docs links.", "Sideshow API inspected surface `M6SpmnsVi5o` and confirmed it includes review mock, changed-files summary, branding assets, and validation summary."]
  summary: "Accepted task-64 based on user visual/product validation of the Sideshow review surface. User likes the logo and favicon and accepts the overview cards as fine for now."
  evidence: ["Working tree contains expected shared-main task-64 changes and no separate branch/worktree/commit was used.", "Current files changed: docs/index.md, site/astro.config.mjs, site/public/favicon.svg, site/src/styles/verdigris.css, site/public/social-card.svg, site/src/assets/tandem-mark.svg, site/src/starlight-route-data.ts."]
  filesChanged: ["docs/index.md", "site/astro.config.mjs", "site/public/favicon.svg", "site/src/styles/verdigris.css", "site/public/social-card.svg", "site/src/assets/tandem-mark.svg", "site/src/starlight-route-data.ts"]
  reviewer: "Algorant"
  updatedAt: "2026-07-04T22:52:31Z"
completedAt: "2026-07-04T22:52:39Z"
completion:
  summary: "Completed docs site polish after user visual validation. Polished the homepage into a launch-style landing page, grouped sidebar navigation, added/refined branding assets and social metadata, fixed canonical edit links to docs sources, and improved Verdigris homepage/code readability styling. User liked the logo and favicon and accepted overview cards as fine for now."
  filesChanged: ["docs/index.md", "site/astro.config.mjs", "site/public/favicon.svg", "site/src/styles/verdigris.css", "site/public/social-card.svg", "site/src/assets/tandem-mark.svg", "site/src/starlight-route-data.ts"]
  validation: "User visual/product validation from Algorant after inspecting Sideshow review surface: http://localhost:8228/session/F2iqcu5r07M/s/M6SpmnsVi5o. Parent verification passed: `git diff --check`, untracked whitespace checks for new SVG/TS files, and `cd site && bun run check:docs` building 13 pages and checking 591 internal links."
  reviewer: "Algorant"
---

## Description

Improve the Starlight site presentation so it feels launch-quality: sidebar grouping, hero/landing copy, theme colors, social/edit links, metadata, favicons/logo, code block readability, and any small custom CSS/components needed without overbuilding.
