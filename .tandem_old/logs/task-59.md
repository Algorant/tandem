---
id: task-59
type: task
title: "Reset Tandem docs site to a barebones useful first draft"
priority: "medium"
relatedFiles: ["docs/index.md", "site/astro.config.mjs", "site/src", "site/src/content/docs"]
tags: ["docs", "site"]
createdAt: "2026-06-29T20:49:04Z"
updatedAt: "2026-07-10T12:37:08Z"
kind: "epic"
completedAt: "2026-07-10T12:37:08Z"
completion:
  summary: "Closed the barebones docs-site reset epic. The sidebar and Home were simplified and human-approved, the existing pared-down secondary pages were retained, and the rejected Quickstart rewrite was discarded and restored to its prior state. Future visual placeholders/assets are tracked separately in task-117."
  filesChanged: ["site/astro.config.mjs", "docs/index.md", "docs/quick-start/index.md", "docs/skills/index.md"]
  validation: "User explicitly approved closing the epic after reconciling all remaining children. Task-96, task-97, task-98, and task-99 are completed logs; task-98 records that its attempted content was rejected and discarded. Current docs checks passed with 15 pages built and 593 internal links checked."
  reviewer: "user"
---

## Description

Parent epic for simplifying the current docs site before adding more material.

Direction:
- Keep the current top bar/logo treatment.
- Start sparse: no filler, no jargon-heavy overview copy, no exhaustive reference pages yet.
- Use placeholders only for intentional future images, tables, and diagrams.
- Prefer a small usable navigation/content skeleton over a launch-sized docs set.
