---
id: task-78
type: task
title: "Investigate and improve validation summary rendering"
priority: "medium"
relatedFiles: ["tandem/src/tui.rs", "tandem/src/tui/logs.rs", "tandem/plan/spec.md", "extensions/pi-tandem", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-shep/index.ts", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem/index.ts"]
tags: ["tui", "validation", "markdown", "ux", "agents"]
createdAt: "2026-07-01T16:21:32Z"
updatedAt: "2026-07-04T23:35:48Z"
accord:
  status: "accepted"
  assignee: "pi-tui-validation"
  claimedAt: "2026-07-04T23:06:43Z"
  deliveredAt: "2026-07-04T23:31:12Z"
  deliverables: ["Updated tandem/src/tui.rs inline board preview rendering to preserve Markdown-ish line structure instead of flattening body text.", "Validation rows now prefer accord.summary as Delivery summary and include validation/evidence/files-changed accord fields.", "Expanded preview height is capped against the visible list viewport so bottom selected rows remain visible.", "Added regression tests for Markdown body preservation, validation accord summaries, and bottom-row viewport behavior."]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml --check", "cargo test --manifest-path tandem/Cargo.toml", "git diff --check -- tandem/src/tui.rs"]
  summary: "Accepted after human TUI validation. User ran the local TUI, expanded task-78 in Validation, and confirmed the delivery summary rendering looks great."
  evidence: ["Shep worker reported 120 Rust tests passing and no blockers.", "Parent re-ran cargo fmt --manifest-path tandem/Cargo.toml --check successfully.", "Parent re-ran cargo test --manifest-path tandem/Cargo.toml successfully: 120 passed.", "Parent re-ran git diff --check -- tandem/src/tui.rs successfully.", "Parent inspected git diff for tandem/src/tui.rs."]
  filesChanged: ["tandem/src/tui.rs"]
  reviewer: "Algorant"
  updatedAt: "2026-07-04T23:34:41Z"
completedAt: "2026-07-04T23:35:48Z"
completion:
  summary: "Improved TUI expanded board preview rendering for validation summaries and committed/pushed the change in 2b960b9. Expanded board rows now preserve Markdown-ish line structure, Validation rows prefer delivery summary/accord metadata, and bottom-row previews are capped to remain visible."
  validation: "Automated validation: cargo fmt --manifest-path tandem/Cargo.toml --check; cargo test --manifest-path tandem/Cargo.toml (120 passed); git diff --check -- tandem/src/tui.rs. Human validation: user verified in the local TUI and said it looks great now."
  reviewer: "Algorant"
---

## Description

Investigate why Validation-board summaries render as hard-to-read walls of text when agent-delivered summaries include bullets, tables, flowcharts, or other Markdown-like structure, then propose the smallest fix.

Context / observed issue:

- Screenshots from another work directory show expanded Validation rows displaying long summaries as a flattened paragraph. Bullets, tables, and paragraph breaks are not preserved, making validation review difficult.
- Initial code inspection suggests this may not be Validation-specific: expanded board rows call `inline_preview_lines_for_doc`, which renders `Summary` from `body_summary(&doc.body)`. `body_summary` trims non-heading lines and joins them with spaces; `inline_preview_paragraph` then wraps with `split_whitespace()`. This intentionally discards newlines and most Markdown structure.
- User examples show Todo items appearing nicely formatted because structured fields such as Tags and Files render as their own sections. The Summary text in those examples is still paragraph-wrapped, so the investigation should distinguish actual Markdown preservation from structured-field rendering.
- Follow-up investigation found task-77/78/79 do have Markdown bodies and show correctly in the detail pane. Their inline expansion appears blank because they sit at the bottom of the list/viewport; the expanded preview is rendered below the selected row and is clipped rather than scrolled into view. This is likely a board inline-expansion viewport/scroll behavior bug, not missing task bodies.
- The detail pane appears to use `markdownish_lines(&doc.body, theme)`, which already preserves/render-styles some Markdown constructs. Need verify whether Validation uses the same renderer as todo/in-progress in the detail pane and whether the problem is specifically expanded board preview.
- Agent-generated deliver/validation summaries may also be stored in accord fields (`accord.summary`, `accord.validation`, evidence, etc.) and/or task body depending on workflow. Need trace where Pi/tandem delivery text lands and how the TUI chooses what to show in Validation.
- Initial Shep/pi-tandem inspection shows weak formatting guidance: `shep_deliver.summary` is a freeform string passed as `tandem accord deliver --summary`, while `deliverables`, `validations`, `evidence`, and `filesChanged` are arrays. Tandem currently renders `accord.summary` as a quoted scalar and validation commands/evidence as arrays, so agents may be encouraged to put too much structured content into `summary` instead of separate fields.

Investigation questions:

1. Is the ugly rendering specific to the Validation state, or does every expanded board row flatten body Markdown the same way when given equivalent Markdown body content?
2. If Todo/In Progress appear formatted, which fields/sections are producing that formatting, and why are Validation items not using the same structure?
3. In Validation rows, should the board preview display task body, `accord.summary`, validation evidence, or a structured combination?
4. Should expanded board previews preserve lightweight Markdown/newlines instead of flattening everything into one paragraph?
5. Is there separate agent/tool guidance that should instruct delivery summaries to use concise, line-oriented Markdown and split structured content across `summary`, `deliverables`, `validations`, `evidence`, and `filesChanged`?
6. Is the existing `markdownish_lines` renderer reusable for inline previews with truncation/height limits, or does the board need a separate preview renderer?
7. How should inline expansion behave near the bottom of the board viewport so expanded content is scrolled into view instead of clipped below the selected row?

Possible fixes to evaluate:

- Replace `body_summary` flattening with a newline-aware preview parser that preserves paragraph breaks, bullets, numbered lists, and simple tables up to the preview line cap.
- Reuse or adapt `markdownish_lines` for expanded board previews while enforcing `INLINE_PREVIEW_MAX_LINES`.
- For Validation specifically, preview `accord.summary` / validation fields before or alongside body text.
- Add/adjust Shep/pi-tandem agent guidance for Tandem validation/delivery summaries so agents avoid huge single-paragraph blobs and use structured tool fields where possible.
- Add tests covering equivalent Todo/In Progress/Validation Markdown bodies and Validation accord summaries with bullets, blank lines, Markdown tables, and long summaries in expanded board previews.
- Add a regression test or manual verification for expanding the last/bottom-visible board item with a non-empty body.

Acceptance criteria:

- Investigation identifies the source(s) of formatting loss and explains why Todo/In Progress can appear formatted while Validation does not.
- Recommendation separates TUI rendering fixes from any agent-guidance changes.
- If implementation is in scope, expanded board previews preserve basic line structure for Markdown-ish summaries without overflowing the board.
- Tests or fixtures demonstrate improved rendering for Validation-style summaries with bullets/tables/newlines.
