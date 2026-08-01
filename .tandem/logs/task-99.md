---
id: task-99
type: task
title: "Stub secondary docs pages with placeholders instead of reference bulk"
priority: "medium"
parentId: "task-59"
references: ["decision-1"]
relatedFiles: ["site/src/content/docs", "docs"]
tags: ["docs", "site", "content"]
createdAt: "2026-07-04T23:25:54Z"
updatedAt: "2026-07-10T10:45:31Z"
accord:
  status: "accepted"
  assignee: "pi"
  claimedAt: "2026-07-10T10:44:39Z"
  deliveredAt: "2026-07-10T10:45:24Z"
  deliverables: ["Confirmed the current Spec, CLI, TUI, Concepts, Workflows, Extensions, and Skills pages represent the already-reduced useful site direction.", "Recorded the user's product judgment that the prior reference-bulk concern is no longer valid."]
  validation:
    commands: ["Reviewed current secondary-page content and Git/Tandem history against task-99 scope.", "cd site && bun run check:docs (15 pages built; 577 internal links passed after the Quickstart reduction)."]
  summary: "Accepted as reconciled: the current secondary docs are already the intentionally pared-down useful version, so additional stubbing would be counterproductive."
  evidence: ["User confirmed the old site contained substantially more useless content and considers the current secondary-page concern resolved."]
  reviewer: "user"
  updatedAt: "2026-07-10T10:45:28Z"
completedAt: "2026-07-10T10:45:31Z"
completion:
  summary: "Reconciled the secondary-page reset scope with the current docs: prior work already removed the old reference bulk, and the user confirmed further stubbing is no longer desired."
  validation: "User product judgment accepted the current pared-down secondary pages; current docs build passed and 577 internal links were checked."
  reviewer: "user"
---

## Description

Replace the old reference/user-guide expansion direction with sparse stubs.

Scope:
- Keep or create minimal pages for Spec, CLI, TUI, Concepts, Workflows, Extensions, and Skills as needed by the sidebar.
- Use explicit placeholders only where future images, tables, or diagrams are expected.
- Defer exhaustive CLI/protocol reference content, full TUI guide text, and visual assets until later tasks are intentionally cut.
- Avoid adding filler paragraphs just to populate pages.
