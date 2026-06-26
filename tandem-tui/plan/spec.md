# Tandem CLI/TUI Spec Draft

Status: draft  
Date: 2026-06-26  
Working name: Tandem  
Implementation target: CLI design first, then Rust + Ratatui TUI

Naming:

- Product/protocol: **Tandem**
- CLI/TUI source directory: `tandem-tui/`
- CLI binary: `tdm`
- CLI design precedes TUI design/implementation
- Likely TUI invocation: `tdm tui` initially, with `tdm-tui` possible as a standalone binary later
- Future extension/tool prefixes: `td` or `tdm`

This document describes the user-facing `tdm` CLI and terminal UI for the Tandem protocol described in `../../protocol/plan/spec.md`.

The CLI/TUI baseline is broad feature parity with the live Brainfile project: keep the general command/workflow shape, then intentionally improve the flawed parts. The intent is not to port the current Brainfile Ink TUI directly. The intent is to design the CLI first, then build a more capable, responsive, themeable, mouse-aware TUI.

## Baseline inputs

- Live Brainfile CLI behavior from `brainfile --help` and subcommand help.
- Installed Brainfile implementation under `/usr/lib/node_modules/@brainfile/cli` for feature discovery.
- Current Brainfile TUI behavior and shortcomings.
- Tandem protocol draft in `../../protocol/plan/spec.md`.
- Local Brainfile v3 direction in `/home/ivan/.dotfiles/pi/.pi/plan/brainfile_v3_spec.md`.

The first CLI/TUI planning deliverable should be a feature parity matrix: keep, rename, improve, omit, open.

## Current Brainfile TUI issues to avoid

Observed/known pain points:

- Progress is tied to a literal `done` column, which does not fit v2 archive/log completion.
- Theming is effectively hardcoded.
- Mouse support is missing.
- Logs exist but are not rich enough to feel like a first-class completed-work view.
- Review/validation/accord status is not prominent enough.
- Many actions are keyboard-only and modal in ways that feel bolted on.
- UI is constrained by the current Ink/React implementation rather than designed around the protocol.

## Product goal

A fast CLI and terminal workspace for managing project work, agent accords, reviews, decisions, rules, and logs.

It should feel closer to a local-first Linear/kanban/logbook hybrid than a simple task list.

## Design principles

- **CLI first:** design `tdm` command workflows before the interactive TUI.
- **Feature parity baseline:** map live Brainfile features before deciding what Tandem keeps, renames, improves, or omits.
- **Logs are real:** completed work is browsable, searchable, inspectable, restorable, and useful.
- **Review is central:** delivered work should naturally flow to review, validation, acceptance, rework, and completion.
- **Agent state is visible:** accord status, evidence, validation, and blockers should be visible without opening raw files.
- **Fast scanning:** compact cards, good color hierarchy, clear badges, and useful filtering.
- **Keyboard-first, not keyboard-only:** vim-style and arrow navigation plus real mouse interactions.
- **Themeable from day one:** no hardcoded palette-only implementation.
- **Small-screen aware:** usable in narrow terminals and inside split panes.
- **File-native:** edits should reflect on disk; external edits should hot reload cleanly.

## CLI planning scope

The CLI is still being designed. Do not lock in implementation dependencies or crate layout yet.

Initial CLI questions:

- Which Brainfile commands map directly to `tdm`?
- Which commands should be renamed for Tandem vocabulary, especially `contract` → `accord`?
- Which Brainfile commands are out of scope for v0?
- What output modes are needed: human-readable only, `--json`, or both?
- Which operations should require confirmation or review policy checks?
- How should `tdm tui` launch the interactive TUI?

Likely command families to evaluate for parity:

```text
init, list, show, add, move, complete, log, search, accord, rules, decision, tui
```

These are planning inputs, not settled implementation commitments.

## Core views

### 1. Board view

Primary view for active work.

Default states:

```text
backlog | todo | active | review
```

Projects may configure state names. The TUI should not assume `done` exists.

Board view should support:

- state/column tabs or columns depending on layout width
- task/work cards
- compact and expanded card modes
- priority, type, tags, parent, blocker, assignee, due date badges
- accord status badges
- review status badges
- selection and multi-select later
- drag or click actions when mouse mode is enabled

Card example:

```text
▌ HIGH [work] Implement Ratatui theme system       [A:claimed] [2/5]
    #tui #rust · @pi · child of epic_01j...        work_01j...
```

Delivered/review item example:

```text
▌ MED  Add decision view                           [A:delivered] [review]
    validation pending · 3 files changed           work_01j...
```

### 2. Detail view

A focused pane or full-screen view for the selected document.

Sections:

- title and metadata
- Markdown description preview
- subtasks
- related files
- accord/assignment
- validation commands/results
- review status and notes
- event timeline
- raw file path

Detail view should make PM validation easy:

```text
Actions: [accept] [request changes] [complete] [reopen] [edit] [copy id]
```

### 3. Review queue

A dedicated view showing items needing attention:

- accord delivered
- review pending
- validation failed
- blocked items
- accepted but not completed

This should answer: “What needs me?”

Suggested sections:

```text
Delivered        3
Validation failed 1
Blocked          2
Ready to complete 4
```

### 4. Logs view

A first-class completed-work browser.

Logs should show:

- completed timestamp
- summary
- files changed
- reviewer
- validation result
- accord status/evidence
- original item body
- event timeline

Actions:

- search logs
- inspect completion details
- restore/reopen
- copy summary
- open files changed
- permanently delete only with strong confirmation

### 5. Rules view

A view for project rules:

- `always`
- `never`
- `prefer`
- `context`

Actions:

- add rule
- edit rule
- delete rule
- copy rule
- maybe promote decision to rule later

### 6. Decisions/notes view

If protocol document types include `decision` and `note`, the TUI should allow browsing them outside normal task flow.

Decision states may be separate from work states:

```text
draft | accepted | superseded
```

## Layout modes

### Wide layout

For terminals >= ~120 columns:

```text
┌ Project title ──────────────── health/status/search ┐
├ Board | Review | Logs | Rules | Decisions | Search ─┤
│ backlog       todo          active        review     │
│ ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌────────┐│
│ │ cards   │   │ cards   │   │ cards   │   │ cards  ││
│ └─────────┘   └─────────┘   └─────────┘   └────────┘│
├──────────────── selected detail / status ───────────┤
│ key hints / command mode / status messages           │
└──────────────────────────────────────────────────────┘
```

### Medium layout

For ~80-119 columns:

- state tabs at top
- single list for selected state
- right or lower detail pane if enough room

### Narrow layout

For ~50-79 columns:

- stacked global list grouped by state
- detail opens full-screen or as expandable cards
- no horizontal board assumptions

### Tiny terminal behavior

For terminals below minimum:

- show clear minimum size message
- avoid panics or corrupt terminal state

## Progress and health widgets

Do not compute progress from a `done` column.

Useful metrics:

- active count
- review count
- blocked count
- delivered needing review
- completed today/week from logs/events
- accord statuses
- validation failures
- stale active items

Potential header:

```text
My Project  active 4 · review 3 · blocked 1 · completed this week 7
```

Optional progress bars:

- epic progress: completed children / total children
- review queue: accepted / delivered
- validation: passed / total delivered
- milestone progress if milestones exist

## Theming

Theme support is required in MVP, not a later polish task.

### Built-in themes

Suggested built-ins:

- `default-dark`
- `default-light`
- `rose-pine`
- `catppuccin-mocha`
- `gruvbox-dark`
- `nord`
- `terminal` / no-truecolor fallback

### Theme file

Possible config path:

```text
.tandem/theme.toml
~/.config/tandem/themes/*.toml
```

Example:

```toml
name = "rose-pine-custom"

[colors]
background = "#191724"
panel = "#1f1d2e"
text = "#e0def4"
muted = "#6e6a86"
accent = "#c4a7e7"
success = "#9ccfd8"
warning = "#f6c177"
error = "#eb6f92"
border = "#403d52"
selected_bg = "#26233a"

[priority]
critical = "#eb6f92"
high = "#f6c177"
medium = "#31748f"
low = "#6e6a86"

[badges.accord]
ready = "#f6c177"
claimed = "#31748f"
delivered = "#c4a7e7"
accepted = "#9ccfd8"
rework = "#ebbcba"
blocked = "#eb6f92"
```

### Theme requirements

- Support truecolor terminals.
- Support 256-color fallback where possible.
- Support no-color mode.
- Keep semantic color names separate from concrete colors.
- Make priority and status badges configurable.
- Avoid relying only on color; include glyphs/text for status.

## Mouse support

Mouse support should be built into the event model.

Required interactions:

- click tabs/views
- click cards to select
- double-click or enter/click to expand detail
- scroll lists with mouse wheel
- click action buttons in detail/review/log views
- click column/state picker in move mode
- click confirmation buttons

Stretch interactions:

- drag card between states
- drag to reorder within a state
- right-click/context menu
- mouse text selection compatibility toggle

### Hit map architecture

During render, widgets register interactive regions:

```rust
struct HitRegion {
    rect: Rect,
    action: UiAction,
    z_index: u16,
    label: String,
}
```

Mouse events resolve against the topmost matching region.

This avoids scattering coordinate math through the app.

## Keyboard model

Support both vim-like and conventional keys.

Global:

| Key | Action |
| --- | --- |
| `q` | quit |
| `?` | help |
| `:` | command palette / command line |
| `/` | search current view |
| `r` | reload |
| `1..6` | switch major view |
| `tab` / `shift-tab` | next/previous section |
| `esc` | close modal/clear filter |

Navigation:

| Key | Action |
| --- | --- |
| `j/k` or arrows | move selection |
| `h/l` or left/right | move column/tab |
| `g/G` | top/bottom |
| `ctrl-d/u` | half-page down/up |
| `enter` | expand/open |

Work actions:

| Key | Action |
| --- | --- |
| `n` | new item quick add |
| `N` | new item in editor |
| `e` | edit selected item in `$EDITOR` |
| `m` | move/change state |
| `p` | change priority |
| `a` | accord action menu (assign/claim/deliver) |
| `v` | validation/review action menu |
| `c` | complete/archive, if allowed |
| `R` | reopen/restore in logs |
| `d` | delete with confirmation |
| `y` | copy ID/link |

All keybindings should be configurable eventually.

## Command palette

The command palette should expose every action so users do not have to memorize keys.

Examples:

```text
:new work
:move review
:complete
:accord deliver
:review accept
:log search theme system
:open file
:copy id
:theme catppuccin-mocha
```

## Search and filters

Search should support plain text and structured filters.

Examples:

```text
theme
state:review
accord:delivered
review:pending
priority:high tag:tui
assignee:pi blocked:true
file:src/theme.rs
```

Saved views should be possible via workspace config later:

```yaml
views:
  needs-review:
    query: "state:review OR accord:delivered"
  mine:
    query: "assignee:ivan OR assignee:pi"
```

## Editing model

The TUI should support two editing styles:

1. Quick inline forms for common fields.
2. `$EDITOR` for full Markdown/frontmatter editing.

Inline edits should call core mutation APIs that preserve unknown fields and minimize diffs.

External editor flow must:

- temporarily restore terminal state
- suspend mouse/raw mode
- wait for editor
- resume TUI cleanly
- reload changed files
- report parse/validation errors clearly

## File watcher and reload

The app should watch:

- workspace config
- `board/`
- `logs/`
- `events.jsonl`
- theme files

Hot reload behavior:

- debounce changes
- preserve selection when possible
- show reload flash/status
- detect selected item deletion/move
- surface parse errors without crashing

## Implementation boundaries (open)

The implementation layout is not settled. Do not assume a root Rust workspace, `crates/`, a standalone core crate, or a specific CLI parsing library until explicitly decided.

Even before folder/crate layout is chosen, the behavioral boundaries should stay clear:

### Protocol behavior responsibilities

- discover `.tandem/` workspaces
- parse config and documents
- expose typed projections for commands/views
- preserve raw documents for minimal patches
- list/filter/query work documents
- mutate fields/states/accords/reviews
- complete/archive/reopen
- append events

### CLI responsibilities

- expose scriptable `tdm` commands
- map command inputs to protocol behavior
- provide predictable human-readable output and optional structured output where useful
- report clear errors and policy failures
- launch the TUI through `tdm tui` if that remains the chosen invocation

### TUI responsibilities

- app state
- event loop
- rendering widgets
- keyboard/mouse mapping
- command palette
- forms/modals
- theme loading
- file watching integration

## Possible dependency areas (not settled)

Potential implementation dependencies should be chosen later and kept minimal:

- `ratatui` for rendering
- terminal input/backend support such as `crossterm`
- focused serialization/frontmatter/event formats
- theme/config parsing if needed
- file watching if it is in MVP
- ID/timestamp helpers only after protocol decisions require them

Need to choose later:

- CLI parser strategy
- direct terminal event loop vs thin internal event abstraction
- text input widgets vs simple custom forms
- how much Markdown rendering is needed for v0

## App state sketch

```rust
enum View {
    Board,
    Review,
    Logs,
    Rules,
    Decisions,
    Search,
}

enum Mode {
    Browse,
    Search,
    Command,
    Move,
    EditForm,
    Confirm,
    Help,
}

struct AppState {
    workspace: WorkspaceState,
    view: View,
    mode: Mode,
    selection: SelectionState,
    filters: FilterState,
    theme: Theme,
    status: Option<StatusMessage>,
    hit_map: HitMap,
}
```

## UI actions

All input should resolve to actions:

```rust
enum UiAction {
    SwitchView(View),
    SelectNext,
    SelectPrev,
    OpenSelected,
    MoveSelected { state: String },
    CompleteSelected,
    AccordClaim,
    AccordDeliver,
    ReviewAccept,
    ReviewRequestChanges,
    OpenCommandPalette,
    Search(String),
    Reload,
    Quit,
}
```

This keeps keyboard, mouse, and command palette behavior consistent.

## Review workflow UX

Happy path:

1. Agent/human delivers item.
2. Item appears in Review queue with `[A:delivered]`.
3. User opens detail.
4. TUI shows deliverables, files changed, validation commands/results, summary.
5. User selects:
   - accept accord
   - request changes
   - run validation
   - complete/archive
6. Completion moves item to logs and appends event.

Important: accepted accord and completed work are not the same thing. The TUI should make this distinction visible.

## Completion UX

When pressing complete/archive:

- show summary form
- show policy checks
- show validation status
- ask for reviewer confirmation if required
- allow files changed list
- then archive to logs

Example:

```text
Complete work_01j...? 

Summary: [ Theme system implemented and tested                    ]
Validation: cargo test passed, cargo clippy passed
Accord: accepted by ivan
Files: 3 changed

[Complete] [Cancel]
```

## Logs UX

Logs are not a trash can. They are a memory/audit surface.

Expanded log item should show:

```text
work_01j... Implement Ratatui theme system
Completed: 2026-06-26 15:00 by ivan
Summary: Theme loading, built-in palettes, runtime style mapping.
Validation: passed
Files changed:
  src/tui/theme.rs
  src/tui/app.rs
Accord:
  assignee: pi
  status: accepted
Timeline:
  12:00 created
  12:05 claimed
  13:30 delivered
  15:00 completed
```

## Error handling

The TUI should never silently corrupt project files.

When a file has invalid YAML/schema:

- keep the app open
- show error panel
- identify file and line/field if possible
- allow opening in editor
- allow reload after fix

For failed writes:

- show precise error
- do not update UI optimistically unless recoverable
- log operation failure to status area

## Testing strategy

Core:

- fixture-based parse/list/mutate tests
- golden-file minimal diff tests
- event ledger append tests
- archive/reopen tests

TUI:

- widget snapshot tests with `ratatui::backend::TestBackend`
- input-to-action tests
- mouse hit-map tests
- layout breakpoint tests
- theme parsing tests

Manual smoke:

- small terminal
- wide terminal
- no-color terminal
- mouse scroll/click
- external editor open/return
- file changed externally while TUI is open

## MVP phases

### Phase 0: Protocol fixtures

- Create example workspace fixtures.
- Use Tandem as the working name/directory while final naming is confirmed.
- Define core structs and parse config/documents.

### Phase 1: Read-only board

- Workspace discovery.
- Read config and board docs.
- Render board/list layouts.
- Selection/navigation.
- Built-in themes.
- Basic detail expansion.

### Phase 2: Mutations

- Add work item.
- Move state.
- Edit in `$EDITOR`.
- Change priority/tags/assignee.
- Toggle subtasks.
- Hot reload.

### Phase 3: Accord and review

- Accord ready/claim/deliver/accept/rework/block.
- Review queue.
- Validation command display/results.
- Review accept/request changes.

### Phase 4: Completion and logs

- Completion form.
- Archive to logs.
- Event ledger append.
- Logs view with rich details.
- Restore/reopen.

### Phase 5: Mouse and polish

- Hit-map mouse selection.
- Clickable tabs/buttons.
- Scroll wheel.
- Drag/reorder if desired.
- Configurable keymap.
- Saved filters/views.

## Open questions

- Confirm Tandem as the final project/product name.
- Should the TUI run as a CLI subcommand (`tdm tui`), a standalone binary (`tdm-tui`), or both?
- Should theme config live in workspace, global config, or both?
- Should mouse mode be enabled by default?
- Should drag-and-drop be MVP or later?
- Which keybindings should be default vs user-configurable in v0?
- Should Markdown rendering be simple text initially or styled headings/lists/code blocks?
- How opinionated should the Review queue be?
