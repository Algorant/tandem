---
id: task-224
type: task
title: "Add a read-only Papercuts utility panel to the TUI"
priority: "medium"
references: ["task-222"]
relatedFiles: ["tandem/src/tui/mod.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/input.rs", "tandem/src/project/mod.rs", "tandem/src/app/papercuts.rs", "tandem/plan/spec.md", "tandem/plan/todo.md", "docs/tui/index.md"]
tags: ["tui", "papercuts", "keyboard", "mouse"]
createdAt: "2026-08-10T17:07:31Z"
updatedAt: "2026-08-10T17:43:45Z"
accord:
  status: "accepted"
  assignee: "worker-task-224-55226d73"
  claimedAt: "2026-08-10T17:09:41Z"
  deliveredAt: "2026-08-10T17:39:01Z"
  deliverables: ["Integrated feature commit a881962 (rebased from Worker commit a382e4201833b8636297cc7bff60d7d716c04ee1).", "Global Papercuts count indicator and P/Esc temporary list/detail panel across Board, Logs, Rules, and Decisions.", "Tolerant open-inbox loading, reload integration, keyboard/mouse interactions, responsive rendering, tests, specs, todos, and user documentation.", "Delegated `just dev` preview route configured for final visual review."]
  validation:
    commands: ["Orchestrator rerun: cargo fmt --check passed.", "Orchestrator rerun: cargo test passed with 257 unit tests and 11 CLI integration tests.", "Orchestrator rerun: cargo clippy --all-targets --all-features -- -D warnings passed.", "Orchestrator rerun: cd site && bun run check:docs passed; 19 pages and 912 internal links.", "git diff --check passed.", "Worker visually inspected the real TUI on Board, Logs, Rules, and Decisions; wide 139x48 and narrow 63-column layouts; default-dark and custom light palettes; populated, empty, long-body, resolved-exclusion, and malformed states; keyboard, reload, focus, scrolling, and hit geometry."]
  constraints: ["Keep task in Validation until human visual acceptance.", "Papercut mutations and resolution behavior remain deferred and excluded from this TUI MVP.", "Physical mouse interaction has automated coverage but was not manually clicked during Worker self-review."]
  summary: "User visually reviewed the integrated `just dev` preview and confirmed that the Papercuts utility panel looks great."
  evidence: ["Worker handoff handoff-895b34cb-ef89-474b-814d-492937f82b4b reported no blockers and a clean source branch.", "Worker corrected header spacing, border collisions, narrow-list visual weight, short-layout row visibility, and narrow footer text after visual inspection.", "Run `just dev` from /home/ivan/Projects/tandem, then press P and inspect each underlying view with 1-4."]
  filesChanged: ["tandem/src/tui/papercuts.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/input.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/state.rs", "tandem/src/app/papercuts.rs", "tandem/src/project/mod.rs", "docs/tui/index.md", "tandem/README.md", "tandem/plan/spec.md", "tandem/plan/todo.md"]
  reviewer: "user"
  updatedAt: "2026-08-10T17:43:35Z"
assignee: "worker-task-224-55226d73"
completedAt: "2026-08-10T17:43:45Z"
completion:
  summary: "Added and visually validated the read-only global Papercuts utility panel. The TUI now shows an open-count indicator in the global gutter and opens a responsive list/detail inbox with `P`, while preserving the four main views and excluding mutation actions."
  filesChanged: ["tandem/src/tui/papercuts.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/chrome.rs", "tandem/src/tui/input.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/state.rs", "tandem/src/app/papercuts.rs", "tandem/src/project/mod.rs", "docs/tui/index.md", "tandem/README.md", "tandem/plan/spec.md", "tandem/plan/todo.md"]
  validation: "257 unit tests and 11 CLI integration tests passed; cargo fmt passed; strict Clippy passed; documentation built with 912 internal links; Worker completed real-TUI visual self-review; user visually reviewed `just dev` and approved the result."
  reviewer: "user"
---

## Description

## Outcome

Expose project Papercuts in the Tandem TUI through a small global utility inbox. Keep Papercuts outside Board workflow and avoid adding a permanent fifth main view.

This Task implements the selected Option A mockup: a global header indicator opens a temporary Papercuts list/detail panel over the current main view.

## Product and information-architecture boundary

Papercuts are lightweight project friction records. They are not Tasks, workflow states, Logs, Rules, or Decisions.

Keep the existing top-level navigation unchanged:

- Board
- Logs
- Rules
- Decisions

Do not add Papercuts after `todo`, `in-progress`, or `validation`. Do not add a `Papercuts` main navigation section in this increment.

## MVP interaction

- Show a compact global header indicator such as `Papercuts 3`, where the number is the open Papercut count.
- Show the indicator from every main TUI view when Papercuts are available. Define a quiet empty state that does not dominate the header when the count is zero.
- Open and close the panel with `P`.
- Make the header indicator mouse-clickable through the existing hit-map model.
- Render the panel as a temporary utility surface over the current view. Closing it restores the prior view, selection, focus, and scroll state.
- Present a selectable list of open Papercuts and a detail pane for the selected record.
- Show useful existing metadata when present: ID, title, status, tags, references, body, and timestamps.
- Support normal list/detail keyboard navigation and mouse selection/scroll consistent with the existing TUI.
- Handle a missing `.tandem/papercuts/` directory as an empty inbox.
- Show malformed-record errors without crashing or preventing unrelated Board, Logs, Rules, or Decisions use.
- Reload Papercut data through the existing TUI reload path.

## Read-only boundary

The first panel is read-only.

- Do not add, edit, resolve, reopen, delete, or promote Papercuts in the TUI.
- Do not define new resolution semantics.
- Existing CLI and `tandem_papercut` actions remain the mutation surfaces.
- The mockup's `resolve` action is only a placeholder for possible future work and is not part of this Task.

## Architecture

- Reuse protocol-owned Papercut parsing and `TandemProject` filesystem access.
- Do not reparse Papercut Markdown or duplicate validation rules in TUI code.
- Keep the established ownership direction: protocol owns meaning, project owns concrete files, app owns shared operations, and TUI consumes those layers.
- Preserve current Board state, arrangement, filters, detail focus, and main-view navigation behavior.

## Visual direction

Use the approved utility-inbox concept from the Sideshow exploration:

- a compact open-count indicator in global chrome;
- a temporary list/detail panel;
- visual weight below the four primary views;
- theme-compatible styling in light and dark themes;
- keyboard-first behavior with equivalent mouse interactions.

The result should feel like a secondary project inbox, not a new workflow column or major product area.

## Documentation

- Update `tandem/plan/spec.md` and `tandem/plan/todo.md` with the implemented TUI behavior and deferred actions.
- Update `docs/tui/index.md` so users can discover the indicator, `P` shortcut, list/detail navigation, and read-only boundary.
- Update other TUI help or reference text only where it enumerates global shortcuts or views.
- Do not imply that Papercuts are Board items or that the TUI can mutate them.
- Run the canonical documentation checks after updates.

## Acceptance criteria

1. The four existing main views remain unchanged and accessible through their current navigation.
2. The global header reports the correct number of open Papercuts.
3. `P` and the header hit target open and close the panel from each main view.
4. Closing the panel restores the prior view and interaction state.
5. The list defaults to open Papercuts and selection drives the detail pane.
6. Details render ID, title, status, optional metadata, and Markdown body safely.
7. Keyboard navigation, mouse selection, scrolling, and theme behavior match existing TUI conventions.
8. No Papercut mutation action is available in the TUI.
9. Missing, empty, and malformed Papercut storage states are handled safely.
10. Existing Board, Logs, Rules, and Decisions tests remain passing.
11. Focused automated tests cover loading, count, opening/closing, navigation, rendering, reload, mouse hit targets, and error isolation.
12. `cargo fmt --check`, full tests, strict Clippy, and documentation checks pass.
13. The delegated preview is made available through the normal `just dev` route and receives human visual validation before acceptance.

## Deferred follow-up ideas

Keep these out of this MVP and record them as later Tasks only if usage supports them:

- add/edit/resolve/reopen/delete actions in the TUI;
- precise TUI resolution workflows or prompts;
- task promotion;
- status, tag, reference, or text filters;
- full-text search inside the panel;
- resolved-history browsing;
- configurable shortcuts or indicator placement;
- a Board `Work | Papercuts` subview;
- a dedicated Papercuts main view;
- dashboards, metrics, grouping, or trends.
## Worker visual self-evaluation before delivery

The implementing Worker must evaluate the rendered TUI itself before delivery, not rely only on unit tests or the orchestrator. It must:

- create or reuse a safe fixture with several open Papercuts, optional metadata, a long body, and empty/error states;
- configure the delegated `just dev` preview route to its worktree and fixture;
- run the real TUI in its retained terminal and inspect the actual rendered frame;
- review the panel from Board, Logs, Rules, and Decisions, at practical narrow and wide terminal sizes, with keyboard and mouse-relevant hit targets, and in available light/dark theme variants;
- correct visible clipping, overlap, weak hierarchy, inconsistent focus, or excessive visual weight before delivery;
- include a concise visual self-review in the handoff with what it inspected, what it changed after inspection, any remaining uncertainty, and exact `just dev` reproduction steps.

Automated render tests are required evidence but do not replace this visual self-evaluation. The orchestrator and user still retain final visual acceptance.
