---
id: task-79
type: task
title: "Specify default and project-configurable TUI badges"
priority: "high"
relatedFiles: ["tandem/src/tui.rs", "tandem/src/tui/theme.rs", "tandem/plan/spec.md", "protocol/plan/spec.md"]
tags: ["tui", "badges", "theme", "config", "spec"]
createdAt: "2026-07-01T16:51:34Z"
updatedAt: "2026-07-01T19:03:05Z"
accord:
  status: "accepted"
  assignee: "shep-badges"
  claimedAt: "2026-07-01T18:07:32Z"
  deliveredAt: "2026-07-01T18:28:47Z"
  deliverables: ["Branch: hp/task79-badges-config in ../tandem-worktrees/hp-task79-badges", "Implemented badge config/rendering in tandem/src/tui.rs and tandem/src/tui/theme.rs", "Updated TUI docs and CLI/TUI planning docs"]
  validation:
    commands: ["cargo fmt --manifest-path tandem/Cargo.toml --check: pass", "cd tandem && cargo test --quiet: 103 passed", "git diff --check: pass", "git diff stat: 6 files, 479 insertions, 50 deletions"]
  summary: "Accepted after human visual validation: user confirmed task-79 badge rendering looks nice and is approved."
  evidence: ["shep_check task-79 showed worker done and no blockers", "Local verification run in /home/ivan/dev/projects/tandem-worktrees/hp-task79-badges"]
  filesChanged: ["docs/tui/index.md", "tandem/README.md", "tandem/plan/spec.md", "tandem/plan/todo.md", "tandem/src/tui.rs", "tandem/src/tui/theme.rs"]
  reviewer: "user/orchestrator"
  updatedAt: "2026-07-01T19:03:00Z"
completedAt: "2026-07-01T19:03:05Z"
completion:
  summary: "Completed configurable TUI badge support after automated verification and user visual approval."
  validation: "Automated verification passed: cargo fmt --check, cargo test --quiet (103 passed), git diff --check. User visually approved task-79 badge rendering: \"everything renders nicely\"."
  reviewer: "user/orchestrator"
---

## Description

Review and specify the TUI badge/chip system with a minimal explicit default set plus project-configurable tag badges.

Goal:

- Canonicalize the built-in/default badge sources in the spec/docs so users know what becomes a badge.
- Add a minimum project/global config mechanism for enabling additional tag badges with optional label/tone.
- Add a minimum project/global config mechanism for disabling built-in or configured badges that a workspace/user does not want surfaced.
- Avoid a broad badge DSL or theming redesign.

Proposed default badge set to verify against current implementation:

- Priority: `CRIT`, `HIGH`, `MED`, `LOW` from priority values.
- Work type tags: `RESEARCH`, `SPIKE`, `DELIVERABLE`.
- Validation attention: `VISUAL` for validation items tagged `visual`, `ui`, or `ux`.
- Accord attention: `DELIVERED`, `ACCEPTED`, `REWORK`, `BLOCKED`, `FAILED` when they affect scan/action priority.
- Review attention: `PENDING`, `CHANGES-REQUESTED`, `REJECTED`, `FAILED`.
- Subtask progress: `2/5`, `5/5`, etc.

Configurable tag badges and suppression:

- Workspace config may define additional tags to render as badges.
- Global/user config may also define personal/default badge preferences, with workspace config taking precedence where needed.
- Tags like `tui`, `cli`, `docs`, `spec`, `protocol` should not be global defaults; they are project/domain-specific.
- Minimum custom badge config should allow optional `label` and optional `tone`.
- Label defaults to uppercase tag.
- Tone defaults to a generic/accent tag badge tone.
- Tones should use existing semantic theme/preset names where possible; raw arbitrary colors are out of scope unless already trivial.
- Config should also allow disabling/removing badges, including built-ins, at project or global scope. This should be simple allow/disable behavior, not a rule engine.

Example shape to evaluate, not necessarily final:

```yaml
badges:
  disabled:
    - deliverable
    - visual
  tags:
    tui:
      label: TUI
      tone: accent
    docs:
      label: DOCS
      tone: success
    spec:
      label: SPEC
      tone: warning
```

Acceptance criteria:

- Current built-in badge behavior is inspected and documented/spec'd explicitly.
- Minimal workspace/global config shape for custom tag badges is proposed or implemented.
- Optional `label` and `tone` behavior is defined.
- Project/global badge disabling behavior is defined for built-in and configured badges.
- Project-specific tags remain opt-in, not global defaults.
- Scope avoids regex rules, arbitrary field badges, custom icons, ordering DSLs, or broad theme rewrites.
