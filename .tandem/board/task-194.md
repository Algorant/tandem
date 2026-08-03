---
id: task-194
type: task
title: "Consolidate pi-tandem agent guidance and remove redundant document"
state: todo
priority: "medium"
relatedFiles: ["extensions/pi-tandem/pi-tandem.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/plan/spec.md", "extensions/pi-tandem/tests/relationship-smoke.ts"]
tags: ["pi-tandem", "guidance", "docs"]
createdAt: "2026-08-03T15:13:05Z"
updatedAt: "2026-08-03T15:13:05Z"
---

## Description

Consolidate pi-tandem guidance so runtime instructions and maintainer documentation have clear, non-duplicated authority. Treat `extensions/pi-tandem/pi-tandem.md` as material to evaluate, not as presumed-valid requirements.

Scope:
- Inventory every statement in `extensions/pi-tandem/pi-tandem.md` against the extension README, `index.ts` tool prompt metadata and workspace-aware prompt addendum, tests, protocol documentation, active Tandem rules, and current Pi guidance.
- Classify each statement as: necessary runtime behavior, useful maintainer documentation, repository-specific policy, duplicate/obvious, obsolete, or unsupported.
- Migrate only unique, current, operationally important agent behavior into the appropriate `index.ts` guidance section.
- Organize `extensions/pi-tandem/README.md` around architecture, authority, lifecycle boundaries, runtime-guidance ownership, and maintainer usage. Keep examples only when they clarify non-obvious behavior.
- Remove duplicated, obvious, stale, unsupported, or repository-only material instead of copying it.
- Delete `extensions/pi-tandem/pi-tandem.md` after all relevant unique content has an intentional destination.
- Remove or update references to that file in README/spec/tests and other tracked documentation.
- Add or refine focused tests for critical runtime guidance and authority boundaries. Do not snapshot or assert incidental prose.
- Produce an explicit cross-repository promotion note listing only changes that must later be applied to the canonical Pi dotfiles implementation. Do not modify personal dotfiles from this repository.

Required design outcome:
- `index.ts` is the authority for guidance actually delivered to agents.
- `README.md` is the authority for maintainer-facing explanation.
- Active `.tandem` rules remain the authority for repository-specific policy.
- No separate unconsumed guidance document remains.

Acceptance evidence:
- A concise disposition table or review note accounts for all sections of the removed file without assuming they must survive.
- Runtime guidance contains all necessary non-obvious agent behavior and no material duplication.
- README clearly explains guidance authority and does not duplicate the runtime prompt verbatim.
- Repository-specific commit-frequency policy remains in active Tandem rules; include it in generic runtime guidance only if independent analysis shows it belongs in every consuming repository.
- Focused pi-tandem smoke, relationship, type/static, and relevant documentation checks pass.
- `git diff --check` passes.
- Cross-repository promotion note identifies exact dotfiles files/sections to update, with no direct cross-repository edits.
