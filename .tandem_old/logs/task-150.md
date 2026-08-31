---
id: task-150
type: task
title: "Implement the approved protocol 0.2 compatibility policy"
priority: "high"
parentId: "task-146"
blockers: ["task-149"]
references: ["decision-7"]
relatedFiles: ["plan/refactor_spec.md", "protocol/README.md", "protocol/plan/spec.md", "tandem/src/main.rs", "tandem/src/tui.rs", "extensions/pi-tandem/README.md", "extensions/pi-tandem/tests/"]
tags: ["protocol", "compatibility", "cli", "rust"]
createdAt: "2026-07-22T20:40:37Z"
updatedAt: "2026-07-26T21:47:00Z"
accord:
  status: "accepted"
  assignee: "worker-task-150-7eff9ec7"
  claimedAt: "2026-07-26T21:31:38Z"
  deliveredAt: "2026-07-26T21:46:47Z"
  deliverables: ["Protocol 0.2 normative documentation and CLI implementation", "Explicit tandem upgrade with 0.1 operation gate", "Legacy custom types/documents preserved read-only with warnings", "Deprecated completion policy preserved, warned, and ignored", "Priority/effort fixed-value validation and pi-tandem forwarding", "Executable compatibility and focused TUI coverage"]
  validation:
    commands: ["cargo fmt --check", "cargo test: 168 unit + 4 executable tests passed", "cargo clippy --all-targets -- -D warnings", "three pi-tandem Bun smoke suites passed", "git diff --check", "parent review/rework closed custom-document, TUI, and init-gating gaps"]
  summary: "Implemented and integrated the approved protocol 0.2 compatibility policy, including explicit upgrade, legacy data preservation/read-only access, fixed metadata vocabularies, TUI diagnostics, and pi-tandem exposure."
  evidence: ["commits 414edc1 and c324460 reviewed and fast-forward integrated", "168 unit and 4 real-command tests passed", "strict Clippy and formatting passed", "pi-tandem smoke, relationship, and runtime suites passed", "legacy custom effort metadata remains readable after upgrade", "0.1 init/list operations explicitly gate to tandem upgrade", "TUI runtime warnings and Effort detail covered"]
  filesChanged: ["tandem/src/main.rs", "tandem/src/tui.rs", "tandem/tests/cli_behavior.rs", "protocol/README.md", "protocol/plan/spec.md", "tandem/README.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/tests/smoke.ts"]
  reviewer: "parent-orchestrator"
  note: "Reviewed the full behavior-changing diff, returned the retained Worker for three concrete compatibility gaps, reviewed the corrective commit, and independently reran Rust and pi-tandem validation. Upgrade intentionally patches only protocolVersion and preserves all other project content. Focused TUI tests establish the added warnings/detail line without a broader visual redesign."
  updatedAt: "2026-07-26T21:46:52Z"
assignee: "worker-task-150-7eff9ec7"
completedAt: "2026-07-26T21:47:00Z"
completion:
  summary: "Implemented protocol 0.2.0 compatibility: explicit upgrade-only conversion from 0.1.0, project-operation gating, legacy custom data preserved as deprecated read-only content, legacy completion policies preserved/deprecated/ignored, fixed priority/effort validation, coherent CLI/TUI/pi-tandem exposure, and regression coverage."
  filesChanged: ["tandem/src/main.rs", "tandem/src/tui.rs", "tandem/tests/cli_behavior.rs", "protocol/README.md", "protocol/plan/spec.md", "tandem/README.md", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/README.md", "extensions/pi-tandem/tests/smoke.ts"]
  validation: "Parent reviewed both commits after one rework cycle and independently passed cargo fmt, 168 Rust unit tests, 4 executable tests, strict Clippy, and all three pi-tandem smoke suites. Upgrade/raw preservation, human/JSON output, TUI compatibility warnings/details, legacy custom access, fixed vocabularies, events, and logs are covered."
  reviewer: "parent-orchestrator"
---

## Description

## Objective

Implement the explicitly approved protocol behavior changes separately from module movement.

## Required behavior

- Update the normative protocol to `protocolVersion: 0.2.0`; new projects initialize at 0.2.0.
- When a 0.1.0 project is discovered, refuse every project operation except explicit `tandem upgrade`; process-level help/version remain available and upgrade is never implicit.
- Keep `task` and `decision` as the only first-class/creatable document types.
- Preserve existing custom-type declarations/documents as deprecated read-only content that can be listed, shown, and searched after upgrade; reject creation and mutation of custom-type documents.
- Preserve legacy project-level completion-policy fields, warn that they are deprecated, ignore their values, and always apply canonical warn-but-complete behavior unless structural errors block.
- Enforce optional priority values `low|medium|high|critical` and effort values `trivial|small|medium|large`.
- Update diagnostics, CLI/TUI exposure, pi-tandem-facing behavior documentation, and executable tests coherently.

## Acceptance criteria

- Normative protocol documents and implementation agree on 0.2.0 and all rules above.
- `tandem upgrade` is explicit, preserves project content, and provides clear success/error output without silently inventing document conversions.
- 0.1.0 rejection, upgrade, legacy custom-type access, deprecated completion settings, fixed-value validation, human output, JSON output, and project-file preservation have executable regression coverage.
- Existing hierarchy, events, completion archives, references, and unknown fields/bodies remain intact.
- Formatting, full tests, real-command tests, and strict Clippy pass.
- Behavior-changing commits remain separate from later move-only extraction commits; no architectural extraction, release, or push occurs.

Creating this Task does not authorize starting it.
