---
id: task-225
type: task
title: "Implement a coherent TUI input model and universal keybinding reference"
state: "in-progress"
priority: "high"
references: ["task-224"]
relatedFiles: ["tandem/src/tui/input.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/papercuts.rs", "tandem/plan/spec.md", "tandem/plan/todo.md", "docs/tui/index.md", "tandem/src/tui/state.rs", "tandem/src/tui/rules.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/validation.rs", "tandem/README.md"]
tags: ["tui", "keyboard", "papercuts", "mouse"]
createdAt: "2026-08-10T17:44:24Z"
updatedAt: "2026-08-10T20:25:18Z"
accord:
  status: "claimed"
  assignee: "worker-task-225-684911b8"
  claimedAt: "2026-08-10T20:25:18Z"
  deliverables: ["Integrated squash commit c47d336 from Worker commits 8f17897 and a31179c.", "Static semantic binding inventory and consistent input precedence.", "Global i utility inbox and removal of P, t/p/F, H/L, A/R/C, plain u/d paging, and Rules n alias.", "Responsive f filter, m move, and v Validation pickers with bounded Apply/Cancel/Help mouse controls and first-enabled selection.", "Universal sectioned ? keybinding reference with context prioritization, responsive scrolling, theme styling, and mouse controls.", "Decision row hits, Rules preview focus/scroll, pane-aware wheel behavior, modal controls, documentation, specifications, and regression coverage.", "Safe `just dev` preview route now targets integrated main code and the retained task-225 visual fixture."]
  validation:
    commands: ["Orchestrator rerun: cargo fmt --check passed.", "Orchestrator rerun: cargo test passed with 265 unit tests and 11 CLI integration tests.", "Orchestrator rerun: cargo clippy --all-targets --all-features -- -D warnings passed.", "Orchestrator rerun: cd site && bun run check:docs passed; 19 pages and 915 internal links.", "git diff --check passed.", "Worker real-TUI review covered wide, narrow, short, populated, empty, warning, Validation, default-dark, and custom light contexts.", "Worker physically exercised picker Apply/Cancel/Help clicks, long Rules preview wheel scrolling, keyboard paging, Decision rows, Rules preview, and confirmation controls after rework."]
  constraints: ["Keep parent and all six child Subtasks in Validation until final human visual and interaction approval.", "No configurable keymap, command palette, notification center, warning/error gutter feature, or Papercut mutations were added."]
  updatedAt: "2026-08-10T20:25:18Z"
assignee: "worker-task-225-684911b8"
---
## Outcome

Replace Tandem's accumulated fixed TUI hotkeys with one coherent, documented, and visually polished input model. Remove unrelated case-only bindings, standardize navigation and overlay precedence, add explicit Board action pickers, close existing keyboard/mouse parity gaps, and provide a universal `?` keybinding reference from every non-text TUI surface.

This is a fixed-keymap overhaul. It is not a configurable keybinding system.

## Problem

The current keymap grew feature by feature. It now contains unrelated case-only pairs and inconsistent behavior:

- `p` cycles Board priority while `P` opens Papercuts.
- `r` reloads while `R` requests rework.
- `a` adds while `A` accepts Validation work.
- `h/l` navigate while `H/L` mutate task state.
- `u/d`, `Ctrl-U/D`, PageUp/PageDown, Enter, and Tab do not have one consistent meaning across views.
- Input handling, footer hints, built-in help, specifications, and user documentation have drifted.
- Some documented mouse behavior is missing or does not target the pane under the pointer.

## Fixed-keymap standard

### Global keys

| Key | Action |
| --- | --- |
| `q` | Quit from every non-text context, including help and the utility inbox. |
| `Ctrl-C` | Emergency safe quit, including from prompts. |
| `?` | Open the universal keybinding reference from every non-text page, panel, picker, and dialog where `?` is not text input. |
| `r` | Reload project, theme, and utility data. |
| `1` through `4` | Switch Board, Logs, Rules, and Decisions. |
| `i` | Open or close the global utility inbox, currently the Papercuts panel. |
| `Esc` | Close or back out of the topmost temporary layer. |

Text prompts own printable characters. Typing `q`, `i`, `?`, or another bound character into a text field must enter that character rather than invoke a global action. `Ctrl-C` remains the explicit emergency exit.

### Navigation keys

| Key | Action |
| --- | --- |
| `j/k` or Up/Down | Move selection or scroll the focused pane vertically. |
| `h/l` or Left/Right | Move between local tabs, categories, or panes. |
| `g/G` or Home/End | Move to the beginning or end of the active list/detail. |
| `Ctrl-U/Ctrl-D` or PageUp/PageDown | Page movement everywhere that supports paging. |
| `Tab`/`Shift-Tab` | Change pane focus only. |
| `Enter` | Open, expand, confirm, or activate the selected item. |
| `Esc` | Close the top layer, cancel, or return to the prior local context. |

Remove plain `u/d` page movement. Uppercase action keys are not used except established related navigation forms such as `g/G` and modifier-driven keys such as `Shift-Tab`.

### Board action keys

| Key | Action |
| --- | --- |
| `a` | Add a task. |
| `e` | Edit the selected active task. |
| `b` | Toggle State Board and Epic Board. |
| `f` | Open Board filter controls. |
| `m` | Open the task state-movement picker. |
| `v` | Open context-aware Validation actions. |
| `Space` | Toggle the selected row's inline preview. |

Replace the old direct groups:

- replace `t/p/F` with `f`;
- replace `H/L` with `m`;
- replace `A/R/C` with `v`;
- replace global `P` with `i`.

### View-local actions

- Logs keeps `/` for search.
- Rules uses `a` add, `e` edit, `d` delete, and Enter preview.
- Decisions uses `a` add and Enter open/expand.
- The utility inbox uses `i` or Esc to close, Tab/Shift-Tab for pane focus, Enter to open/activate, and the standard navigation keys.
- Remove the duplicate Rules `n` add alias.
- Do not retain compatibility aliases for removed keys unless a concrete migration need is found during implementation and approved before delivery.

## Board action pickers

The `f`, `m`, and `v` bindings must open explicit, consistently designed picker surfaces. They must not become undocumented chains of second-stage hotkeys.

### Filter picker (`f`)

- Show current tag and priority filters.
- Allow tag selection, priority selection, clearing one filter, and clearing all filters.
- Make unavailable choices clear.
- Preserve selection and hierarchy context after applying a filter.

### Move picker (`m`)

- Show the selected task and its current state.
- List valid configured target states.
- Prevent invalid or no-op movement.
- Use the existing shared app mutation path and preserve graph validation.
- Require an explicit selection/confirmation rather than mutating because Shift was held accidentally.

### Validation picker (`v`)

- Show only actions valid for the selected task and current Accord/review state.
- Route accept, rework, and apply/archive through existing shared Validation operations and existing confirmation or feedback steps.
- Explain why an action is unavailable rather than silently doing nothing.

All three pickers use the same visual grammar, focus model, footer structure, keyboard behavior, and mouse hit-map conventions.

## Global utility/status gutter

Keep the `Papercuts N` indicator in the global header gutter introduced by task-224. Rename internal concepts where useful so the layout is a utility/status gutter rather than a Papercuts-only architectural slot.

- `i` opens the current utility inbox.
- The visible label remains specific, such as `Papercuts 3`.
- Preserve the compact, visually secondary treatment.
- Keep hit geometry extensible so future warning or error indicators can occupy the gutter.
- Do not add warning/error indicators, a general notification center, or additional utility panels in this Task.

## Universal keybinding reference

Add or overhaul the built-in `?` help into a complete, attractive keybinding reference. It must be callable from Board, Logs, Rules, Decisions, the utility inbox, and every non-text picker/dialog.

Requirements:

- Organize commands into clear sections rather than one undifferentiated list.
- At minimum include: Global, Navigation, Current view, Board actions, Validation, Logs, Rules, Decisions, Utility inbox, Dialogs and text input, and Mouse.
- Prioritize the current view and currently open picker/panel while still allowing discovery of the complete keymap.
- Clearly distinguish global, view-local, pane-local, and modal controls.
- Show primary keys and equivalent arrow/Page/Home/End forms without noisy repetition.
- Use theme-owned styles and a deliberate hierarchy for section titles, key labels, descriptions, selected section, and scroll position.
- Render well in wide and narrow/short terminals. The reference must scroll without clipping.
- Support standard keyboard scrolling and pane/section navigation.
- Support mouse wheel scrolling, section selection where shown, and a safe close hit target.
- `Esc` closes help and restores the exact underlying view/panel/picker state.
- `q` quits the TUI from help under the global quit standard.
- In active text-entry prompts, `?` remains text. Provide a visible help affordance outside the text field rather than stealing the character.
- Remove planned or unimplemented commands from the canonical reference.

Use a small static semantic action/binding inventory as the source for runtime help, footer/header labels, mouse-equivalent actions, and consistency tests where practical. Do not build runtime rebinding, user configuration, chord parsing, or a general keymap framework.

## Input precedence

Implement and test this order:

1. `Ctrl-C` emergency quit.
2. Active text input owns printable characters and text-editing controls.
3. Active confirmations, menus, and pickers own their local controls.
4. `Esc` closes the topmost layer.
5. `q` quits from non-text contexts.
6. Universal global actions such as `?`, `r`, `1` through `4`, and `i` run where the active layer permits them.
7. View-local actions run before pane-local navigation only when the action is valid in that view.
8. Unbound keys are safe no-ops with no mutation.

Switching main views while a picker or utility panel is open must be explicitly defined and consistent. Prefer closing the temporary layer before switching rather than mutating hidden underlying state.

## Mouse parity included in this Task

Bring documented mouse behavior into parity with the semantic action model:

- Add Decision row hit targets.
- Make Rules preview activation available by mouse.
- Make Rules and Decisions wheel behavior target the pane under the pointer.
- Add mouse controls for confirmation dialogs and the new `f`, `m`, and `v` pickers.
- Keep Board, Logs, Papercuts, header gutter, tabs, list/detail focus, and footer actions aligned with their keyboard equivalents.
- Ensure overlays consume pointer events so clicks never mutate the obscured underlying view.
- Dispatch keyboard and mouse paths to the same semantic operations where possible.

## Documentation and specification

- Replace the stale/planned key table in `tandem/plan/spec.md` with the implemented fixed-keymap standard.
- Update `tandem/plan/todo.md` so final fixed-keymap work is accurately marked.
- Update `docs/tui/index.md` with the canonical user-facing keymap and help behavior.
- Update `tandem/README.md` only if its TUI summary or shortcuts require correction.
- Ensure built-in help, header indicators, contextual footer hints, specs, docs, tests, and actual dispatch agree.
- Correct existing overstatements about Decision row clicks, pane-aware wheel behavior, and modal mouse controls as they become implemented.

## Validation and visual review

Automated coverage must include:

- the final global keymap;
- removal of old aliases;
- prompt ownership of printable text;
- `Ctrl-C`, `q`, Esc, help, utility inbox, and picker precedence;
- view switching with temporary layers;
- consistent Ctrl-U/D and PageUp/PageDown behavior;
- Enter action versus Tab focus behavior;
- `f`, `m`, and `v` picker actions and disabled states;
- keyboard/mouse semantic parity;
- Decision row clicks, Rules preview clicks, pane-aware wheel behavior, and modal hit targets;
- help content completeness and agreement with the semantic binding inventory;
- wide, narrow, short, dark, light, empty, populated, warning, and validation contexts.

Run formatting, the full Rust test suite, strict Clippy for all targets/features, documentation checks, and `git diff --check`.

Before delivery, the implementing Worker must configure the delegated `just dev` preview route and perform its own real-TUI visual and interaction review. It must inspect every main view, utility inbox, `f/m/v` picker, confirmation dialog, universal help reference, keyboard flow, mouse targets, narrow/wide terminal sizes, and available light/dark themes. The handoff must state what changed after visual inspection and provide exact reproduction steps. Final human visual acceptance remains required.

## Acceptance criteria

1. No unrelated action depends on lowercase versus uppercase of the same letter.
2. `i` opens the utility inbox globally and the old `P` binding is removed.
3. `f`, `m`, and `v` replace `t/p/F`, `H/L`, and `A/R/C` respectively.
4. Navigation follows the fixed standard across every view and panel.
5. Plain `u/d` paging is removed; Ctrl-U/D and PageUp/PageDown work consistently.
6. Enter activates and Tab changes focus consistently.
7. Text prompts retain every printable character, including `?`, `q`, and `i`.
8. `q` quits from every non-text context and Esc closes the topmost layer.
9. The universal `?` keybinding reference is complete, sectioned, polished, responsive, scrollable, and available from every non-text page/panel/picker.
10. Help, footer/header hints, mouse hits, specs, docs, tests, and input behavior agree.
11. Decision selection, Rules preview, pane-aware wheel behavior, confirmation dialogs, and new pickers have working mouse equivalents.
12. Overlays cannot leak keyboard or mouse actions into underlying views.
13. The four main views and read-only Papercuts behavior remain intact.
14. No configurable keymap system, notification center, or warning/error gutter feature is introduced.
15. Automated validation and Worker visual self-review pass.
16. The integrated result remains in Validation until user visual approval.

## Execution model

The child Subtasks below are ordered checkpoints within this parent Task. They share one retained Worker, worktree, and review boundary. Do not delegate the Subtasks to separate Workers.

## Deferred work

- User-configurable keybindings.
- Multi-key chords or leader-key systems.
- Command palette implementation.
- Warning/error counters or a general notification center in the utility gutter.
- Additional utility panels.
- Papercut mutation actions.
- Changes to the four-view information architecture.
