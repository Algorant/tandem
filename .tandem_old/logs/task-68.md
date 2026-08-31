---
id: task-68
type: task
title: "Research and apply the official Astro Gruvbox theme"
priority: "low"
relatedFiles: ["site/package.json", "site/astro.config.mjs", "site/src/styles/custom.css", "justfile"]
tags: ["docs", "theme", "astro", "starlight", "gruvbox", "research"]
createdAt: "2026-06-29T23:04:11Z"
updatedAt: "2026-07-04T13:10:32Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-04T11:53:19Z"
  deliveredAt: "2026-07-04T12:54:55Z"
  deliverables: ["Added local Gruvbox Starlight CSS variables via site/src/styles/gruvbox.css and configured Starlight customCss.", "Vendored upstream Gruvbox Shiki/Expressive Code JSONC themes plus MIT license under site/src/styles/shiki/.", "Configured site/astro.config.mjs to load vendored Gruvbox code themes using ExpressiveCodeTheme.fromJSONString(...).", "Added direct compatible astro-expressive-code@^0.44.0 dependency because astro.config.mjs imports it directly.", "Updated docs/guides/docs-site.md to document the Astro 7-compatible workaround and why the incompatible package dependency is avoided."]
  validation:
    commands: ["Parent reran `cd site && npm run build`: passed, 11 pages built.", "Parent reran `cd site && npm audit --audit-level=low`: passed, 0 vulnerabilities.", "Parent reran `cd site && npm ls astro @astrojs/starlight astro-expressive-code --depth=0`: Astro 7.0.3, Starlight 0.41.1, astro-expressive-code 0.44.0.", "Parent loaded both vendored JSONC themes with ExpressiveCodeTheme.fromJSONString successfully.", "`git diff --check` passed for tracked changes."]
  summary: "User validated the docs site Gruvbox workaround/theme result and requested completion."
  evidence: ["git status shows only task-68-related files modified/untracked.", "site/package.json/package-lock only add astro-expressive-code; Astro and Starlight remain at ^7.0.3 and ^0.41.1.", "License file included for vendored upstream theme assets."]
  filesChanged: ["docs/guides/docs-site.md", "site/astro.config.mjs", "site/package.json", "site/package-lock.json", "site/src/styles/gruvbox.css", "site/src/styles/shiki/gruvbox-dark-medium.jsonc", "site/src/styles/shiki/gruvbox-light-medium.jsonc", "site/src/styles/shiki/starlight-theme-gruvbox.LICENSE"]
  reviewer: "Algorant"
  updatedAt: "2026-07-04T13:10:24Z"
completedAt: "2026-07-04T13:10:32Z"
completion:
  summary: "Completed Astro 7/Starlight-compatible Gruvbox docs-site workaround after user validation."
  filesChanged: ["docs/guides/docs-site.md", "site/astro.config.mjs", "site/package.json", "site/package-lock.json", "site/src/styles/gruvbox.css", "site/src/styles/shiki/gruvbox-dark-medium.jsonc", "site/src/styles/shiki/gruvbox-light-medium.jsonc", "site/src/styles/shiki/starlight-theme-gruvbox.LICENSE"]
  validation: "User validated task-68. Prior verification passed: docs build, npm audit with 0 vulnerabilities, dependency check preserving Astro 7/Starlight 0.41, Expressive Code theme loading, and git diff whitespace check."
  reviewer: "Algorant"
---

## Description

Investigate the official Astro/Starlight Gruvbox theme or integration, determine the recommended installation/configuration path for the Tandem documentation site, and apply it if compatible. Include validation of local docs build/preview, note any theme limitations or custom overrides needed, and avoid broad layout redesign unless separately approved.
