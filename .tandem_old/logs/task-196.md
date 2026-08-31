---
id: task-196
type: task
kind: "epic"
title: "Site overhaul"
priority: "high"
tags: ["site", "docs", "ui"]
createdAt: "2026-08-04T23:33:36Z"
updatedAt: "2026-08-05T02:56:58Z"
completedAt: "2026-08-05T02:56:58Z"
completion:
  summary: "Completed the trytandem.dev site overhaul across all approved documentation pages, workflows, integrations, navigation, validation, and local preview setup."
  filesChanged: ["docs/", "site/astro.config.mjs", "README.md", "plan/papercuts.md"]
  validation: "just site-build; cd site && bun run check:links; just docs reached Astro ready state; 831 internal links checked"
  reviewer: "orchestrator"
---

## Description

Overhaul the trytandem.dev Astro documentation site page by page. Review each page direction with the project owner in Sideshow before implementation. Track approved page work as child Tasks and delegate implementation only after requirements are clear.
