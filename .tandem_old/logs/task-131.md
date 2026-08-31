---
id: task-131
type: task
title: "Create a release notes guidance decision record"
priority: "medium"
references: ["task-72", "task-27"]
relatedFiles: ["tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md"]
tags: ["docs", "release", "decision", "guidance"]
createdAt: "2026-07-14T11:37:20Z"
updatedAt: "2026-07-15T04:25:24Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-14T20:04:15Z"
  deliveredAt: "2026-07-14T20:16:26Z"
  deliverables: ["decision-5 — Curate concise public release notes without installation guidance", "ADR-compatible proposed decision with Context, Decision, Product review, Consequences, and Alternatives considered", "Conventional conditional release-note template informed by Keep a Changelog, GitHub guidance, and current Rust project examples"]
  validation:
    commands: ["tandem decision show decision-5 --json — parsed and returned proposed decision", "tandem decision list --json — decision-5 present with status proposed", "Parent inspected full raw `.tandem/board/decision-5.md` and structured Tandem output", "git diff --check — passed", "git status --short — clean; main remains ahead only by the two previously merged commits", "No commit expected: `.tandem/` is intentionally ignored and no tracked files changed"]
  summary: "Accepted after explicit product-owner approval. Decision-5 is now recorded as accepted with Algorant as decider; the conventional conditional template, no-install-content rule, dedicated Bug fixes rule, and curated-over-generated policy were reviewed and validated."
  evidence: ["Decision references task-131, task-72, and task-27", "Decision body cites tandem/RELEASE.md and tandem/GITHUB_RELEASE_NOTES.md as contextual implementation inputs", "External references: Keep a Changelog 1.1.0, GitHub generated-release-notes documentation, uv, bat, and ripgrep release examples"]
  reviewer: "user-and-parent-orchestrator"
  updatedAt: "2026-07-15T04:25:17Z"
completedAt: "2026-07-15T04:25:24Z"
completion:
  summary: "Created and accepted decision-5 establishing conventional, concise, curated public release notes without installation guidance and requiring a dedicated Bug fixes section whenever fixes ship."
  validation: "Explicit product-owner approval; accepted accord; decision parses through tandem decision show/list; parent reviewed complete record and research citations; tracked git status clean."
  reviewer: "user-and-parent-orchestrator"
---

## Description

Create a first-class Tandem decision that establishes durable public release-notes guidance.

Context:
- Current release guidance recommends curated, version-specific notes grouped by product surface.
- The project owner clarified that public release notes must never include installation commands or installation guidance.
- Releases containing bug fixes should have a clear Bug fixes section rather than burying fixes among features.
- Existing release documentation may conflict with this direction and should be cited as context, not silently treated as authoritative.

Acceptance criteria:
- Use `tandem decision add` / `tandem_decision` to create an ADR-compatible decision record rather than encoding the decision in task state.
- Record the no-install-content rule explicitly.
- Define when and how a dedicated Bug fixes section is used.
- Define the preferred concise release-note structure, including highlights, product-surface sections, upgrade/compatibility notes when relevant, and omission of rejected/not-shipped work.
- Clarify the role of curated notes versus commit-generated or GitHub-generated drafts.
- Reference prior release-notes research and the current release checklist/notes files.
- Present the proposed decision for product review before treating any ambiguous structural preferences as final.
