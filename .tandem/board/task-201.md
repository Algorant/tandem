---
id: task-201
type: task
title: "Overhaul CLI Reference page"
state: "validation"
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/cli/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-05T01:50:01Z"
updatedAt: "2026-08-05T02:45:32Z"
accord:
  status: "delivered"
  assignee: "worker-task-201-4c9c417d"
  claimedAt: "2026-08-05T02:26:09Z"
  deliveredAt: "2026-08-05T02:45:32Z"
  deliverables: ["docs/cli/index.md"]
  validation:
    commands: ["git diff --check passed", "just site-build passed", "cd site && bun run check:links passed: 831 internal links across 19 HTML files"]
  summary: "Merged the feature-complete CLI Reference including upgrade, command families, options, transitions, JSON output, and examples."
  filesChanged: ["docs/cli/index.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-08-05T02:45:32Z"
assignee: "worker-task-201-4c9c417d"
---
## Approved CLI Reference page direction

- Make this a technical, feature-complete reference for every command and option defined by the Tandem CLI specification.
- Keep the visible title `CLI Reference`.
- Organize the page by command family, with one section for every command and subcommand.
- Document every supported option, argument, default, state transition, and important behavior.
- Include a concrete example command and short explanatory text for every command and option.
- Cover at minimum: `init`, `list`, `show`, `add`, `move`, `update`, `complete`, `cancel`, `search`, `log`, `accord`, `rules`, `decision`, `tui`, `version`, and `--version`.
- Include all accord actions: `claim`, `deliver`, `accept`, `rework`, `block`, and `fail`.
- Include all Rules actions: `list`, `add`, `edit`, and `delete`.
- Include all Decision actions: `list`, `show`, and `add`.
- Explain human-readable output versus `--json` reads and show JSON examples where supported.
- Keep installation and end-to-end onboarding links connected to Quickstart, but do not reduce this page to a quickstart.
- Preserve technical accuracy and update the reference when the CLI specification changes.

Implementation remains pending until the page review is complete.