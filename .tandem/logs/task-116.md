---
id: task-116
type: task
title: "Rewrite README to reflect the current Tandem project"
priority: "medium"
relatedFiles: ["README.md"]
tags: ["docs", "readme"]
createdAt: "2026-07-10T10:58:06Z"
updatedAt: "2026-07-10T12:31:22Z"
subtasks:
  - id: task-116-1
    title: "Summarize Tandem's current purpose and capabilities"
    completed: false
  - id: task-116-2
    title: "Link the project website and key documentation"
    completed: false
  - id: task-116-3
    title: "Document installation, getting started, and common usage"
    completed: false
  - id: task-116-4
    title: "Update repository/status guidance and retain appropriate links to plans/specs"
    completed: false
accord:
  status: "accepted"
  assignee: "shep-readme"
  claimedAt: "2026-07-10T11:07:18Z"
  deliveredAt: "2026-07-10T12:31:10Z"
  deliverables: ["Current Tandem overview, website/docs links, repository layout, installation, getting-started, workflow, TUI, documentation, and extensions sections.", "Intentional placeholders for future visual assets now tracked separately by task-117.", "Rendered Sideshow preview reviewed and validated by the user: http://localhost:8228/session/j4aBWw2HQkM/s/En4ZlusZ-3w"]
  validation:
    commands: ["User reviewed the updated README through Sideshow and explicitly marked task-116 validated.", "git diff --check passed after removing three trailing-whitespace artifacts without changing copy.", "README changes are committed on main; no branch/worktree merge is required."]
  summary: "Accepted after the user reviewed the current README through Sideshow and explicitly validated the final manually refined copy."
  evidence: ["Initial worker commit: 2e92532 Rewrite project README.", "User refinement commit: 2eb59ad Refine project README.", "Sideshow surface En4ZlusZ-3w version 2 in session j4aBWw2HQkM."]
  filesChanged: ["README.md"]
  reviewer: "user"
  updatedAt: "2026-07-10T12:31:17Z"
completedAt: "2026-07-10T12:31:22Z"
completion:
  summary: "Completed the root README rewrite with the user's final copy, current project framing, installation and usage guidance, documentation links, and intentional visual placeholders tracked by task-117."
  filesChanged: ["README.md"]
  validation: "User reviewed and approved the current README through Sideshow surface En4ZlusZ-3w; git diff --check passed; commits 2e92532 and 2eb59ad are preserved on main."
  reviewer: "user"
---

## Description

Replace the initialized greenfield/planning-oriented root README with a polished project README that accurately represents Tandem today. Include the project website, a concise product overview, links to the most useful documentation, installation/getting-started and everyday usage guidance, and an accurate summary of the current repository/project state. Keep deeper plan/spec files discoverable without presenting them as the primary user entry point.
