---
id: task-10
type: task
title: "Prototype Verdigris TUI theme"
priority: "medium"
relatedFiles: ["tandem-tui/src/tui/theme.rs", "tandem-tui/src/tui.rs", "tandem-tui/README.md", "tandem-tui/plan/spec.md"]
tags: ["tui", "theme", "verdigris", "visual-design"]
createdAt: "2026-06-27T15:02:47Z"
updatedAt: "2026-06-27T23:45:33Z"
subtasks:
  - id: task-10-1
    title: "Map Verdigris palette roles to Tandem TUI theme keys"
    completed: false
  - id: task-10-2
    title: "Add loadable built-in or workspace theme example"
    completed: false
  - id: task-10-3
    title: "Smoke visually in tdm tui and document usage"
    completed: false
accord:
  status: "accepted"
  assignee: "herd:tui-verdigris-theme"
  claimedAt: "2026-06-27T22:49:30Z"
  deliveredAt: "2026-06-27T23:04:03Z"
  deliverables: ["code:tandem-tui/src/tui/theme.rs:Verdigris theme or loader support", "docs:tandem-tui/README.md:Verdigris usage and palette notes"]
  validation:
    commands: ["cd tandem-tui && cargo fmt --check", "cd tandem-tui && cargo test", "cd tandem-tui && cargo build"]
  constraints: ["Use the local Verdigris/Pi sources in the task description as palette guidance.", "Keep terminal readability and no-color fallback intact."]
  summary: "Implemented selectable Verdigris TUI theme prototype in commit 4d4f5e8."
  evidence: ["Validation passed in /home/ivan/dev/projects/tandem-task10-verdigris: cargo fmt --check; cargo test; cargo build; git diff --check HEAD~1..HEAD.", "PTY smoke copied tandem-tui/examples/themes/verdigris.toml to .tandem/theme.toml and found both 'built-in verdigris' and NO_COLOR 'built-in terminal/no-color'."]
  filesChanged: ["tandem-tui/src/tui/theme.rs", "tandem-tui/examples/themes/verdigris.toml", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  reviewer: "orchestrator"
  note: "Reviewed and integrated Verdigris theme branch; final cargo validation and PTY smoke passed."
  updatedAt: "2026-06-27T23:45:33Z"
completedAt: "2026-06-27T23:45:33Z"
completion:
  summary: "Integrated selectable Verdigris TUI theme with example theme selector, docs, validation, and smoke coverage."
  filesChanged: ["tandem-tui/src/tui/theme.rs", "tandem-tui/examples/themes/verdigris.toml", "tandem-tui/README.md", "tandem-tui/plan/spec.md", "tandem-tui/plan/todo.md"]
  validation: "cargo fmt --check && cargo test && cargo build; git diff --check; Verdigris theme PTY smoke and NO_COLOR smoke"
  reviewer: "orchestrator"
---

## Description

Prototype a Verdigris-inspired Tandem TUI theme and make it easy to evaluate visually.

Research notes gathered before task creation:
- Pi Verdigris theme source: /home/ivan/.dotfiles/pi/.pi/agent/themes/verdigris.json
  - bg #1d2021, fg #ebdbb2, muted #928374, panel #222526, panelAlt #252829
  - accent/fern #8ec07c, accentDark/patina #689d6a, secondary/aqua #83a598, moss #70764a
  - warning/ochre #e6bf86, error/diff-red #e36f63, diffContext #dacba6
  - Pi mapping uses fern for accent/borders/list bullets, patina for darker accent/code borders, aqua for links/user-message emphasis, ochre for warnings.
- Verdigris project overview: /home/ivan/dev/projects/verdigris/README.md
  - Core palette: ashbronze #665c54, vellum #ebdbb2, fern #8ec07c, patina #689d6a, aqua #83a598, ochre #e6bf86.
  - Design language: oxidized copper, weathered bronze, warm vellum, patina greens, cool aqua, restrained brass; gruvbox/everforest adjacent.
- Neovim palette source: /home/ivan/dev/projects/verdigris.nvim/lua/verdigris/palette.lua
  - Neutral dark bases: bg0 #1d2021, bg1 #282828, bg2 #32302f, bg3 #3c3836, bg4 #504945, bg5 #665c54.
  - Text: fg0 #fbf1c7, fg1 #ebdbb2, fg2 #d5c4a1, fg3 #bdae93, fg4 #a89984, fg5 #928374.
  - Identity colors: fern #8ec07c, aqua/patina #689d6a, cyan #83a598, teal #458588, moss #70764a.
  - Warm accents: ochre #d8a65c / #e0b36f / #e6bf86, yellow #fabd2f, orange #d65d0e, red #fb4934, purple #d3869b.
- Verdigris Pi README: /home/ivan/dev/projects/verdigris/pi/README.md
  - Pi stays close to Gruvbox dark backgrounds for transcript readability.
  - It prioritizes muted panels and readable Markdown over showing the full palette at once.

Acceptance direction:
- Add either a built-in `verdigris` TuiTheme or a checked-in example `.tandem/theme.toml`/docs path that can be loaded immediately.
- Keep contrast/readability high in terminal panes.
- Map priority/accord/review badges intentionally using fern/patina/aqua/ochre/moss/diff-red.
- Preserve NO_COLOR/TANDEM_NO_COLOR fallback.
- Include a PTY/manual visual smoke note and update docs with exact keys/usage.
