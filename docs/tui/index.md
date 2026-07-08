---
title: TUI
description: Using the Tandem terminal user interface.
---
`tandem tui` opens the Ratatui-based terminal interface for a Tandem workspace.

## Views

The v0 TUI includes:

- **Board** — active tasks grouped by configured workflow state, including the Validation state/subview.
- **Logs** — completed task history.
- **Rules** — project coordination rules.
- **Decisions** — ADR-compatible `decision` documents.

## Navigation

Use `1` through `4` to switch top-level views, `j`/`k` or arrow keys to move selection, `h`/`l` for local pane or state navigation, `?` for help, and `q` to quit.

Board supports quick task creation, state movement, reloads, mouse selection/scrolling, inline preview expansion by clicking the selected row, and opening active task Markdown in `$EDITOR`. The Decisions view lists active decision records, displays metadata/body/path, and supports a basic title/body add prompt with `a`; include ADR sections such as Status, Context, Decision, Consequences, and Supersession in the body when recording architecture decisions. Mouse hit regions are intentionally bounded: top tabs switch views, Board state tabs switch states, Board/Logs/Decisions rows select items, visible footer actions reuse the matching keyboard behavior where safe, wheel events scroll the pane under the pointer, and non-action regions no-op. Drag/drop is out of v0.

## Themes

The TUI includes built-in themes and can load user TOML themes from `~/.config/tandem/themes/` or `$XDG_CONFIG_HOME/tandem/themes/`. Set your normal theme in `~/.config/tandem/config.toml` or `$XDG_CONFIG_HOME/tandem/config.toml`:

```toml
theme = "verdigris"
transparent_background = true
badge_style = "muted"
```

Use workspace `.tandem/theme.toml` only when a project should override the user's global theme preference. Use workspace `.tandem/config.toml` for project display semantics such as Board tag badge opt-ins.

Set `transparent_background = true` in a user theme, user config, or workspace `.tandem/theme.toml` to let the terminal default/transparent background show through for app and panel fills where practical. The default is `false`, so omitted themes preserve opaque rendering.

Set `badge_style` to control Board priority/status/tag chips. The default, `muted`, keeps the compact badge shape with a softer fill for transparent and image-backed terminals. Other options are:

```toml
badge_style = "muted" # muted chip, default
badge_style = "accent"  # small colored rail plus label
badge_style = "text"    # colored label, no chip fill
badge_style = "ghost"   # outlined/chip text with transparent fill
badge_style = "solid"   # legacy saturated filled block
```

The same key can also be written as `[badges] style = "ghost"` in user themes, user config, or `.tandem/theme.toml`. Rounded-edge badge rendering remains deferred.

Default Board badges are intentionally minimal: priority (`CRIT`, `HIGH`, `MED`, `LOW`), work-type tags (`RESEARCH`, `SPIKE`, `DELIVERABLE`), validation `VISUAL`, attention accord/review statuses, and subtask progress such as `2/5`. Project/domain tags like `tui`, `cli`, `docs`, `spec`, or `protocol` are opt-in rather than global defaults.

Configure extra tag badges or suppress badges in user config or workspace `.tandem/config.toml`:

```toml
[board.badges]
disabled = ["deliverable", "visual"]

[board.badges.tags.tui]
label = "TUI"
tone = "accent"

[board.badges.tags.docs]
# label defaults to "DOCS"
tone = "success"
```

`label` and `tone` are optional for configured tags. `label` defaults to the uppercase tag, and `tone` defaults to `accent`; supported tones are `accent`, `success`, `warning`, `error`, and `muted`. `disabled` is a simple list of built-in badge IDs or configured tag names to suppress, not a regex/rule engine. Legacy `[badges]` / `[badges.tags.*]` sections in theme files are still read during migration, but new project badge config should live in `.tandem/config.toml`.
