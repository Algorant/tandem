---
id: task-24
type: task
title: "Research Tandem documentation platform and static site approach"
state: "in-progress"
priority: "medium"
relatedFiles: ["README.md", "plan/spec.md", "protocol/plan/spec.md", "tandem/plan/spec.md", "extensions/plan/spec.md", "plan/docs-platform-research.md"]
tags: ["docs", "research", "static-site", "github-pages"]
createdAt: "2026-06-28T03:45:11Z"
updatedAt: "2026-06-28T04:42:04Z"
subtasks:
  - id: task-24-1
    title: "Define evaluation criteria for simplicity, aesthetics, Markdown workflow, static output, and repo fit"
    completed: false
  - id: task-24-2
    title: "Compare a focused shortlist without assuming a tool bias"
    completed: false
  - id: task-24-3
    title: "Recommend one primary platform plus one fallback"
    completed: false
  - id: task-24-4
    title: "Propose initial Tandem docs information architecture"
    completed: false
  - id: task-24-5
    title: "Document a GitHub Pages-compatible deployment path"
    completed: false
  - id: task-24-6
    title: "Create follow-up implementation tasks if the recommendation is accepted"
    completed: false
accord:
  status: "claimed"
  assignee: "herd:task24-docs-platform-research"
  claimedAt: "2026-06-28T04:42:04Z"
  deliverables: ["docs:plan/docs-platform-research.md:recommendation memo comparing candidate documentation/static-site platforms", "proposal:docs information architecture covering project overview, protocol/spec, CLI, TUI, extensions, and usage guides", "deployment:GitHub Pages-compatible static site path with local preview and maintenance workflow", "tasks:follow-up implementation tasks if the recommendation is accepted"]
  validation:
    commands: ["test -f plan/docs-platform-research.md"]
  constraints: ["Research only; do not implement or add a docs generator in this task unless explicitly redirected.", "Do not assume a preferred ecosystem; evaluate best-fit options against Tandem's workflow.", "Keep recommendations simple, low-maintenance, Markdown-friendly, aesthetic, and compatible with static deployment."]
  updatedAt: "2026-06-28T04:42:04Z"
---

## Description

Research and recommend a simple, maintainable documentation platform for Tandem as the project matures.

User direction:
- Treat this as a research/spike task, not an implementation task.
- Produce a recommendation memo, not a broad landscape survey.
- Do not start with a tool bias; compare the best-fit options on their merits.
- Optimize for GitHub Pages-compatible static output.
- The docs system should be simple to maintain, aesthetically good, Markdown-friendly, and able to support project structure docs, protocol/spec docs, CLI/TUI docs, extensions docs, and user-facing usage guides.

Research questions:
1. Which documentation/static-site paradigm best fits Tandem's workflow and repo shape?
2. What shortlist of tools should be considered, and why? Include common options such as Docusaurus, VitePress, Astro Starlight, MkDocs Material, mdBook, and any clearly better fit discovered during research.
3. What docs information architecture should Tandem use initially?
4. How should generated/static docs deploy through a simple GitHub Pages-compatible path?
5. What should be deferred to avoid maintenance burden or premature automation?

Acceptance direction:
- Recommend one primary platform and one fallback.
- Include the reasoning, tradeoffs, maintenance workflow, and deploy model.
- Propose an initial docs tree/section structure for Tandem.
- Produce follow-up implementation tasks if the recommendation is accepted.
