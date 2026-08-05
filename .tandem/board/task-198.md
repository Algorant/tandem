---
id: task-198
type: task
title: "Overhaul Quickstart page"
state: todo
priority: "high"
parentId: "task-196"
relatedFiles: ["docs/quick-start/index.md", "site/astro.config.mjs", "site/src/styles/verdigris.css"]
tags: ["site", "docs", "ui"]
createdAt: "2026-08-04T23:41:12Z"
updatedAt: "2026-08-05T00:46:10Z"
---
## Approved Quickstart review notes

- Keep the Quickstart as one continuous page.
- Do not use tabs, modals, panes, or step-navigation UI.
- Start with a simple `Install Tandem` section.
- Show exactly three installation choices as plain text with a header and command:
  1. Installer — `curl -fsSL https://trytandem.dev/install.sh | sh`
  2. Rust — `cargo install --git https://github.com/Algorant/tandem.git --tag tandem-v0.4.0 --path tandem --locked`
  3. AUR — `paru -S tandem-bin`
- Continue directly into `Initialize a workspace` and all remaining Quickstart sections on the same page.
- Initialize with `tandem init`.
- Under it, show only the optional title comment: `--title "my tandem project"`.
- For adding tasks, point users to the forthcoming `Guidance for agents` page and tell them to direct their agent to create a task.
- Explain that the agent understands the fields and can fill them from natural-language requests, with examples such as `help me research the best static site generators` and `build me a simple terminal todo app`.
- Show that users can run `tandem list` to view tasks in the terminal or ask their agent to show them.
- Step 4 should be inviting and actionable. Show an example prompt such as `Please begin work on task-4`, with alternatives to delegate it or ask what to do next.
- Step 5 should be titled `Verify the result`. Show an agent-style completion message such as `Task done! Please review the result: http://localhost:4321` and explain that the user should open the URL, inspect the changes, or review the deliverables.
- Step 6 should be titled `Give feedback`. Show friendly examples such as `The UX needs some work` and `The “Create task” button is not functioning`, then explain that the agent can make fixes. Also show the positive path: tell the agent the work is validated and move to the next task.
- Link the Logs page of `tandem tui` for more information.
- End the Quickstart after these six steps; do not add a separate TUI step.
- Preserve technical accuracy, copyable commands, responsive behavior, and light/dark theme support.

The Sideshow proposal reflects this direction. Implementation remains pending until the page review is complete.