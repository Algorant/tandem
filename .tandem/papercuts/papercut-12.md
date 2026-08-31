---
id: papercut-12
title: "TUI: removed a (Add) and m (Move) actions are still bound after the 0.3.0 cutover"
status: open
createdAt: "2026-08-31T02:48:28Z"
updatedAt: "2026-08-31T02:48:28Z"
---
The protocol 0.3.0 cutover removed TUI Add and direct Move from the footer/help text, but the live bindings and mouse hit actions remain in code (input.rs:124 'a' -> start_quick_add on Board; input.rs:138 'm' -> start_move_picker; StartQuickAdd/OpenMovePicker hit actions). Both contradict D67 and the task-247 acceptance ('Add/direct Move are removed'), and quick-add likely fails the mandatory-Accord rule on creation (add requires --acceptance). Fix: remove the 'a' Board quick-add binding + quick-add path, remove the 'm' move binding + move picker + PickerAction::Move dispatch, keep Rules/Decisions 'a' add prompts (those are in-scope surfaces), and update tests.
