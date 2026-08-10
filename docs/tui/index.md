---
title: TUI
description: Using the Tandem terminal user interface.
---
`tandem tui` opens Tandem's Ratatui terminal interface for a workspace. Use it for the daily task board, validation work, project rules, decisions, completed work logs, and the secondary read-only Papercuts inbox.

## Screen gallery

The following placeholders mark the screens that should be represented by future renders, GIFs, or screenshots. Each screen is usable from `tandem tui`.

| Screen placeholder | What it shows |
| --- | --- |
| **Board / State Board**<br>*[render or screenshot placeholder]* | Active tasks grouped by `todo`, `in-progress`, and `validation`. Use it to select tasks, inspect hierarchy, filter work, and move through the workflow. |
| **Epic Board**<br>*[render or screenshot placeholder]* | Epics with their direct Tasks and leaf Subtasks. Use it to see the full delivery tree and state labels in one view. |
| **Logs**<br>*[render or screenshot placeholder]* | Completed and canceled work. Use it to search history and inspect archived task details. |
| **Rules**<br>*[render or screenshot placeholder]* | Workspace rules grouped by category. Use it to browse a compact list and open a full rule preview. |
| **Decisions**<br>*[render or screenshot placeholder]* | ADR-compatible decisions with metadata, body, and path. Use it to inspect or add durable project choices. |
| **Papercuts utility panel**<br>*[render or screenshot placeholder]* | A temporary open-Papercut list and detail pane over the current main view. It is a read-only secondary inbox, not a fifth view or Board column. |
| **Help and interaction states**<br>*[render or GIF placeholder]* | Key hints, selected-row previews, drawers, prompts, validation warnings, and mouse focus states. |

## Views and navigation

The top-level views are Board, Logs, Rules, and Decisions. Press `1` through `4` to switch views, `j`/`k` or the arrow keys to move selection, `h`/`l` for local pane or state navigation, `?` for help, and `q` to quit. Press `Tab`/`BackTab` to cycle focus where a view has multiple panes. Press `e` in Board to edit the selected active task in `$EDITOR`.

Every main view shows a global `Papercuts N` header indicator for the open count. A zero count stays muted. Press `P`, or click the indicator, to open the temporary Papercuts utility panel. Press `P` or `Esc` to close it and restore the current main view with its selection, focus, filters, arrangement, and scroll unchanged. Use `j`/`k`, arrows, `g`/`G`, page keys, `h`/`l`, `Enter`, or `Tab` for local list/detail navigation. Mouse row selection, pane focus, and wheel scrolling mirror those keys.

The panel lists valid open Papercuts and renders ID, title, status, optional tags and references, created/updated timestamps, path, and styled Markdown body. It is read-only. Add, edit, resolve, reopen, delete, and task-promotion actions stay in the CLI or integration tools. A missing `.tandem/papercuts/` directory is an empty inbox. A malformed record appears as a panel warning without blocking Board, Logs, Rules, or Decisions. Normal manual and watched reloads include Papercut files.

Board supports quick task creation, state movement, reloads, mouse selection and scrolling, inline hierarchy, row previews, and task editing. Press `b` to switch between the State Board and Epic Board. Press `Enter` (or click an already-selected parent row) to expand or collapse children one level at a time. Press `Space` to toggle the inline preview on any row. On a leaf row, `Enter` keeps the normal preview behavior.

Logs supports list/detail focus and `/` filtering. Rules supports category navigation and a full-width preview drawer: press `Enter` to open or close the selected rule. Decisions supports list/detail focus and an `a` prompt for a title and body. Include ADR sections such as Status, Context, Decision, Consequences, and Supersession when recording an architecture decision.

Mouse hit regions are bounded and mirror safe keyboard actions. Click top tabs to switch views, the global Papercuts count to open or close its panel, Board state tabs to switch states, and Papercut/Board/Logs/Decisions rows to select items. Click an already-selected Board row to toggle its preview. Footer actions use the matching keyboard behavior where safe. The wheel scrolls the pane under the pointer; other regions are no-ops. Drag and drop is not supported in v0.

## Board, hierarchy, and filtering

The State Board treats `parentId` links between tasks as a hierarchy rather than flat peers. Root tasks and tasks whose parent is a decision or custom document remain normal rows. Collapsed parents show concise active and logged descendant rollups. Expanded descendants use aligned tree guides and disclosure markers. Exact identity remains in the Selected header and detail context, not redundant `SUB` labels in State Board rows.

Children keep their own workflow state. An expanded child from another state gets a compact state label; a same-state child reserves that column for an aligned title. State-tab counts count active documents by their own state, even when a document is collapsed under a parent in another tab. Expanding a tree changes visible rows, not tab counts.

Board tag and priority filters reveal each matching descendant with its required active ancestor path in the descendant's own state tab. A match is never hidden only because its parent is collapsed. Completed descendants stay in Logs and contribute `logged` rollups without becoming active rows.

The Epic Board groups each active Epic with its direct global-ID Tasks and their parent-derived Subtasks. It keeps the ancestor path visible when a filter matches a Subtask. Direct Task rows use compact state labels such as `TODO`, `WIP`, and `VAL`; only leaf Subtask rows use `SUB`. Stable `<parentId> → <childId>` context remains visible where useful.

The canonical hierarchy is strict: an Epic has global-ID Tasks, and a Task can have parent-derived leaf Subtasks. Invalid nested Epics, children beneath Subtasks, hierarchical IDs directly beneath Epics, and global-ID Subtasks fail workspace validation instead of using compatibility rendering. Completed and canceled Tasks/Subtasks stay in Logs. Canceled logs are labeled and excluded from successful-completion rollups.

## Validation and workflow

The Board presents the active `todo`, `in-progress`, and `validation` states. `validation` is the review queue; it is not a permanent completion state. Papercuts do not join these states or the Board hierarchy. Accord status and review metadata remain separate from workflow state. The TUI surfaces structural validation warnings and errors so invalid hierarchy or unresolved required references can be corrected before work is trusted. Completion archives a task to Logs rather than creating a persistent `done` state.

## Themes and badges

Theme loading uses this order: built-in defaults, a user TOML theme, user config, then workspace overrides. User themes live in `$XDG_CONFIG_HOME/tandem/themes/*.toml` or, when `XDG_CONFIG_HOME` is unset, `~/.config/tandem/themes/*.toml`. User config is `$XDG_CONFIG_HOME/tandem/config.toml` or `~/.config/tandem/config.toml`. A workspace can override the selection in `.tandem/theme.toml`; use `.tandem/config.toml` for project Board display settings. Invalid user or workspace configuration is non-fatal and appears as a warning in the TUI status line.

### Select a theme

`theme` selects a built-in theme or a user theme by name. The default built-in theme is `default-dark`; `verdigris` is also supported. Put the selection in user config for a normal preference:

```toml
# ~/.config/tandem/config.toml
theme = "verdigris"
```

Use a workspace selector only for a project-specific override:

```toml
# .tandem/theme.toml
theme = "default-dark"
```

### Background and badge style

`transparent_background` is `false` by default. Set it to `true` in a user theme, user config, or `.tandem/theme.toml` to let the terminal or compositor background show through app and panel fills where practical:

```toml
transparent_background = true
```

`badge_style` controls priority, status, and tag chips. The default is `muted`. Supported styles are `muted` (soft fill), `accent` (colored rail), `text` (colored label without chip fill), `ghost` (transparent outlined chip), and `solid` (legacy saturated fill):

```toml
badge_style = "ghost"
```

The compatibility spelling `[badges] style` is also accepted in user themes, user config, and `.tandem/theme.toml`:

```toml
[badges]
style = "ghost"
```

Rounded-edge badge rendering is not supported in v0.

### Board badge configuration

Default Board badges stay minimal: priority (`CRIT`, `HIGH`, `MED`, `LOW`), common repository work tags (`BUG`, `FEAT`, `CHORE`, `RESEARCH`, `SPIKE`, `DELIVERABLE`), validation `VISUAL`, attention accord/review statuses, and Subtask progress such as `2/5`. `BUG`, `FEAT`, and `CHORE` are built in because they describe common work kinds across repositories. Project/domain tags such as `tui`, `cli`, `docs`, `spec`, and `protocol` remain opt-in rather than global defaults.

Use `.tandem/config.toml` for project badge choices:

```toml
[board.badges]
# Suppress built-in badge IDs or configured tag names.
disabled = ["deliverable", "visual"]

[board.badges.tags.tui]
label = "TUI"
tone = "accent"

[board.badges.tags.docs]
# label defaults to the uppercase tag: DOCS.
tone = "success"
```

`[board.badges] disabled` is a list, not a pattern or rule engine. It can suppress a built-in tag by name, such as `"bug"`. For each `[board.badges.tags.<tag>]` entry, `label` is optional and defaults to the uppercase tag. A configured entry overrides the label or tone of a built-in tag without rendering a duplicate. `tone` is optional. It keeps the built-in tone for a built-in tag and otherwise defaults to `accent`. Existing tones remain supported: `accent`, `success`, `warning`, `error`, and `muted`. Theme-owned work-tag tones add `orange`, `sand` (alias `beige`), and `purple`. The built-in `BUG`, `FEAT`, and `CHORE` badges use those three tones respectively.

Themes can customize these named palette roles without changing Board rendering or project badge selection:

```toml
[badges.tones]
orange = "#fb923c"
sand = "#d6b98c"
purple = "#c084fc"
```

The built-in `default-dark` palette uses rework orange, warm sand, and delivered purple. Verdigris uses burnt copper (`#c96f3d`), ready sand (`#e6bf86`), and validation heather (`#ad8294`). All tones use the selected `muted`, `accent`, `text`, `ghost`, or `solid` badge style. They also preserve labels in terminal/no-color mode.

For compatibility, legacy `[badges] disabled` and `[badges.tags.<tag>]` settings are still read from theme/config files. New project badge settings should use `[board.badges]` and live in `.tandem/config.toml`; the newer sections make project display choices distinct from theme styling.
