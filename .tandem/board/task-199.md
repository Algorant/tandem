---
id: task-199
type: task
title: "Overhaul Workspace page"
state: "validation"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/protocol/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T00:14:04Z"
updatedAt: "2026-08-05T02:45:24Z"
accord:
  status: "delivered"
  assignee: "worker-task-199-87c7ef87"
  claimedAt: "2026-08-05T02:26:50Z"
  deliveredAt: "2026-08-05T02:45:24Z"
  deliverables: ["docs/workspace/index.md", "site/astro.config.mjs", "README.md"]
  validation:
    commands: ["git diff --check passed", "cd site && bun install --frozen-lockfile && bun run check:docs passed in Worker validation", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Merged the Workspace page, /protocol/ redirect, and approved Overview navigation."
  filesChanged: ["docs/workspace/index.md", "site/astro.config.mjs", "README.md", "docs/protocol/index.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:45:24Z"
assignee: "worker-task-199-87c7ef87"
---
## Approved Workspace page direction

- Rename the visible page from `Spec` to `Workspace`.
- Move the page route from `/protocol/` to `/workspace/`; preserve `/protocol/` with a redirect for existing links.
- Describe the `.tandem/` workspace rather than presenting this page as the normative protocol specification.
- Open with the `.tandem/` file tree, as on the live site.
- Keep the opening layout section technical and reference-oriented.
- Show representative document filenames for Tasks, Subtasks, and Decisions.
- Do not include a separate Work hierarchy diagram.
- Add a friendly tooltip or inline guidance link to Concepts for visitors who do not recognize terms such as Tasks, Epics, Rules, or Decisions.
- After Layout, use plain headings with short text, lists, or small renders describing what each folder or file contains.
- Cover `tandem.md`, `board/`, `logs/`, `actor-id`, `events/`, and the legacy `events.jsonl` file.
- Give `tandem.md` its own dedicated Rules section with representative `always` and `never` examples.
- Show what a log document looks like with a representative Markdown/YAML example or a clear `PLACEHOLDER: log screenshot` block.
- For the events section, use this plain description: `A timestamped log of everything that happens within a Tandem project.` Remove the per-actor/legacy explanation and the event file example from the page mockup.
- Update the Overview sidebar order and labels to: `Workspace`, `Concepts`, `CLI Reference`, `TUI`.
- Preserve the existing Verdigris styling and readable code-block presentation.

Implementation remains pending until the page review is complete.