---
id: task-93
type: task
title: "Explore Verdigris-inspired docs theme direction with Sideshow mockups"
priority: "low"
references: ["decision-1", "decision-3", "task-68"]
relatedFiles: ["site/src/styles/gruvbox.css", "site/astro.config.mjs", "docs/guides/docs-site.md", "site/src/styles/shiki", "README.md"]
tags: ["docs", "theme", "research", "verdigris", "design", "sideshow"]
createdAt: "2026-07-04T14:16:03Z"
updatedAt: "2026-07-04T15:39:09Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-04T14:17:32Z"
  deliveredAt: "2026-07-04T15:36:30Z"
  deliverables: ["New Sideshow URL: http://localhost:8228/s/pRcDGGYSsE4", "New surface ID: pRcDGGYSsE4", "New session ID: Z7uBLjoadR8", "Files changed in this pass: none.", "Existing dirty files observed but not created by this republish pass: site/src/styles/gruvbox.css, docs/guides/docs-site.md, site/README.md."]
  validation:
    commands: ["Republished due Sideshow server restart/session loss.", "Mockup includes standalone Verdigris docs/site theme direction, H1-H6, body text, links/tags, sidebar/nav states, cards/callouts/blockquotes, and intentional Verdigris code block/syntax treatment.", "Included palette/role mapping and short recommendation for future standalone vendorable Starlight/Expressive Code implementation.", "No new Sideshow feedback at publish time."]
  summary: "Accepted after user visual/product approval. User reviewed the republished Sideshow mockup and approved the standalone Verdigris docs/site theme direction, including intentional code block styling."
  evidence: ["http://localhost:8228/s/pRcDGGYSsE4", "surfaceId: pRcDGGYSsE4", "sessionId: Z7uBLjoadR8"]
  filesChanged: ["site/src/styles/gruvbox.css"]
  reviewer: "parent/user"
  updatedAt: "2026-07-04T15:38:54Z"
completedAt: "2026-07-04T15:39:09Z"
completion:
  summary: "Completed visual direction exploration after explicit user approval. Approved direction is a standalone Verdigris docs/site theme with colorful heading scale, Verdigris links/tags/sidebar/cards/callouts/blockquotes, and intentional Verdigris code block styling. Final republished Sideshow mockup: http://localhost:8228/s/pRcDGGYSsE4"
  validation: "User reviewed the republished Sideshow mockup and explicitly approved proceeding to official implementation."
  reviewer: "parent/user"
---

## Description

Investigate and prototype a Verdigris-inspired visual direction for the Astro/Starlight docs site before implementing any durable theme changes.

Expected approach:
- Start with a short investigation of the current docs theme implementation, including the Gruvbox workaround in `site/src/styles/gruvbox.css`, Expressive Code theme configuration, and available Verdigris palette sources.
- Produce Sideshow renders/mockups first, per project rule, showing a few plausible Verdigris-style variants for the current docs site look and feel.
- Compare variants for readability, brand fit, light/dark behavior, accessibility/contrast, code block treatment, navigation/sidebar appearance, and maintainability.
- Identify decisions needed before adoption, such as palette source of truth, whether Verdigris replaces Gruvbox or becomes a hybrid/variant, code-theme strategy, vendoring policy, and whether the docs theme should define Tandem's broader brand identity.
- After review/selection, implement the chosen direction as a scoped Astro/Starlight theme adjustment using supported customization paths such as `customCss` and Expressive Code configuration.

Acceptance direction:
- Do not skip the Sideshow/mockup phase.
- Keep implementation small and reversible unless a design decision explicitly expands scope.
- Validate with the docs build and appropriate visual review.
