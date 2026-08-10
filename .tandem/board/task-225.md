---
id: task-225
type: task
title: "Overhaul and rationalize the fixed TUI keybindings"
state: todo
priority: "medium"
references: ["task-224"]
relatedFiles: ["tandem/src/tui/input.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/papercuts.rs", "tandem/plan/spec.md", "tandem/plan/todo.md", "docs/tui/index.md"]
tags: ["tui", "keyboard", "papercuts"]
createdAt: "2026-08-10T17:44:24Z"
updatedAt: "2026-08-10T17:44:24Z"
---

## Description

## Outcome

Audit and rationalize Tandem's fixed TUI keybindings so global and local actions are predictable, discoverable, and free from unnecessary case-only conflicts.

The immediate usability problem is that lowercase `p` and uppercase `P` currently invoke unrelated features. Lowercase `p` cycles the Board priority filter, while uppercase `P` opens the global Papercuts utility panel. This is easy to trigger incorrectly and difficult to remember.

Do not assume the correct fix is simply moving one of those actions. Review the complete keymap and establish a coherent system before changing bindings.

## Scope

### Inventory and design

- Inventory every implemented keyboard action across global navigation, Board, Logs, Rules, Decisions, Papercuts, prompts, dialogs, list/detail panes, and help.
- Record each key's scope, action, mnemonic, case sensitivity, and context-dependent reuse.
- Identify collisions, case-only distinctions, weak mnemonics, unreachable actions, inconsistent navigation, and differences between actual input handling, footer hints, help text, specifications, and user documentation.
- Define concise principles for the fixed keymap, including when context-dependent reuse is acceptable and when uppercase/lowercase variants are too easy to confuse.
- Prefer familiar terminal and vim-style movement where it does not conflict with product actions.
- Keep numeric `1` through `4` navigation for Board, Logs, Rules, and Decisions unless the audit finds a concrete blocker.
- Keep Papercuts globally accessible, but choose a binding that fits the complete keymap rather than preserving `P` automatically.

### Implementation

- Apply the approved fixed-keymap changes consistently across input dispatch, mouse-equivalent actions, contextual footer hints, help surfaces, tests, specifications, and user documentation.
- Keep local actions local. A local binding must not unexpectedly invoke a global action when focus or an overlay changes.
- Ensure prompts and dialogs continue to own ordinary text input without global shortcuts stealing entered characters.
- Add regression coverage for global/local precedence, modal behavior, case handling, and the final Papercuts shortcut.
- Centralize keybinding labels or metadata where a small shared representation can prevent input/help/footer drift. Do not build a generalized keymap framework without evidence that it is necessary.

## Global utility/status gutter

Preserve the new global header gutter introduced by task-224. Papercuts fit well there because the area is visible but visually secondary.

Treat this as the beginning of a possible global utility/status area. Future work may also surface warning or error counts there. This Task may clarify naming, spacing, hit geometry, and keybinding interaction for the gutter, but it must not add a general notification system or new warning/error UI.

## Documentation

- Update the canonical fixed-keybinding table in `tandem/plan/spec.md`.
- Update `tandem/plan/todo.md` so keymap planning state is accurate.
- Update `docs/tui/index.md` and any other user-facing shortcut references.
- Ensure built-in TUI help and contextual footer hints match the implemented bindings exactly.
- Document intentional context-dependent reuse where it remains.

## Acceptance criteria

1. A complete implemented keybinding inventory is produced and used to drive the changes.
2. Unrelated actions no longer depend on the `p` versus `P` distinction.
3. The final Papercuts shortcut is global, documented, tested, and shown consistently in the header/footer/help surfaces.
4. Global, view-local, pane-local, and modal precedence is explicit and covered by tests.
5. Prompt text entry is not intercepted by unrelated global shortcuts.
6. Footer hints, help, specifications, documentation, and input behavior agree.
7. Mouse hit targets continue to invoke the same actions as their keyboard equivalents.
8. The four main views and Papercuts panel retain their current functionality and interaction state.
9. No configurable keymap system or general notification center is introduced.
10. Focused keymap tests, the full test suite, formatting, strict Clippy, and documentation checks pass.
11. The updated help, footer, header gutter, and common workflows receive real-TUI visual and interaction review before acceptance.

## Deferred considerations

Keep these as future possibilities unless a separate Task authorizes them:

- user-configurable keybindings;
- command palette implementation;
- multi-key chords or leader-key systems;
- a general global notification, warning, or error center;
- warning/error counters in the utility gutter;
- redesigning Papercuts beyond the existing read-only panel;
- changing the four main-view information architecture.

