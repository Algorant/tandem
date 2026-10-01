---
id: task-49
type: task
title: "Implement independent native metadata sync for seamless multi-machine Tandem use"
priority: "high"
effort: "large"
references: ["task-48", "decision-7", "decision-8"]
relatedFiles: ["protocol/plan/independent-metadata-sync.md", "protocol/README.md", "protocol/plan/spec.md", "tandem/src/protocol/", "tandem/src/project/", "tandem/src/app/", "tandem/src/cli/", "tandem/src/tui/", "tandem/tests/checkpoint_behavior.rs", "docs/workspace/index.md", "docs/guides/agents-and-adapters.md"]
tags: ["protocol", "sync", "multi-machine", "git"]
accord:
  status: "accepted"
  acceptance: ["An Algorant-approved task-48 design defines the metadata ref/store, authoritative Markdown representation, stable workspace discovery, linked-worktree behavior, ID grammar/short handles, native synchronization triggers, migration, and narrow supersession of conflicting protocol/checkpoint decisions before implementation begins.", "Routine metadata mutations are durably recorded and synchronized through a Tandem-owned channel on the existing remote, without staging source files, changing the source branch HEAD/history, or modifying unrelated staged, unstaged, and untracked state. Pending metadata does not obstruct an otherwise-valid source-only git pull --ff-only.", "Independent offline creation of Tasks, Subtasks under the same parent, Decisions, and Rules produces permanent non-colliding canonical identities and reference keys; common synchronization does not repeatedly renumber records or require fetching/reservation before local creation. Short-handle ambiguity is detected rather than silently resolving to the wrong record.", "Native CLI/TUI use automatically refreshes and publishes metadata according to the approved bounded scheduling policy; users need not switch metadata branches, manually checkpoint each mutation, or run a sync command after each edit. Explicit native sync/status diagnostics distinguish saved locally, pending/offline, synchronized, and conflicted outcomes.", "Native three-way semantic reconciliation preserves independent creations and compatible independent edits, recognizes archive moves by logical identity, validates relationships and lifecycle invariants after merging, preserves unknown fields/bodies, and combines event ledgers with identity/sequence integrity checks rather than a blind text union.", "Genuinely contradictory edits preserve both versions and expose a precise native resolution flow naming the affected record and both sides. No conflict markers become canonical Markdown, no wall-clock last-write-wins silently loses work, and neither agents nor adapters hand-edit, delete, or rename .tandem to repair sync.", "Offline writes, interrupted sync, repeated requests, and a remote advance between fetch and push remain recoverable and idempotent. Native sync retries ordinary publication races without force-push or rewriting published source history and never reports synchronized before remote acknowledgement.", "New clones and linked worktrees find the same workspace through the approved stable locator and native setup/hydration path without copying actor identity or relying on code-branch freshness. Concurrent local mutations and synchronization are serialized or revision-checked safely.", "A reviewed native migration/dry-run path preserves existing valid IDs, references, Decisions, Rules, Logs, and event history; inventories every participating machine's pending state; safely handles eligible unpublished/unstarted legacy collisions; and stops with both sides identified for ambiguous legacy collisions. Old clients cannot silently overwrite the new format.", "A reproducible native two-clone regression matrix covers all task-48 failures plus Rule/Subtask collisions, compatible same-record edits, true conflicts, offline/reconnect, concurrent push, interruption/retry, fresh-clone hydration, and linked worktrees. It asserts exact content preservation and unchanged source HEAD/index/dirty files, not merely command exit success.", "Protocol, CLI/TUI, and agent guidance document the independent-sync contract, explain that historical code checkouts no longer select historical boards, and provide explicit adapter/sysup handoffs. Superseded source-branch metadata checkpoint/consolidation behavior is removed according to the approved cutover rather than retained as a fallback."]
  claimedAt: "2026-09-30T03:55:22Z"
  deliveredAt: "2026-10-01T06:38:02Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml", "$ git diff --check", "Run the native disposable two-clone synchronization matrix; compare canonical records, references, event identities/sequences, and source HEAD/index/working-tree bytes before and after every scenario.", "Exercise interrupted publication and remote-race retries; verify no data loss, duplicate application, false synced status, or source-history rewrite.", "Dry-run migration using disposable copies of representative existing workspaces, including pending legacy collisions; verify existing valid IDs/logs remain valid and ambiguous cases produce two-sided diagnostics."]
  constraints: ["Implement the approved design in protocol/plan/independent-metadata-sync.md; material deviations need Algorant's approval first.", "Work the Subtasks in order. Check in with Algorant after the spike (task-49-1) and before changing how Tandem stores files.", "Spike uncertain Git-ref, storage, crash-recovery, and merge behavior in disposable state before implementation; review the implementation plan and contract readiness before delegation.", "Do not create an Epic, implicit compatibility modes, Pi-side reconciliation, a source-Git wrapper, or a mandatory hosted service.", "Tandem owns all metadata Git writes, identities, parsing, migration, conflict detection, and resolution. Changes to external Pi/sysup adapters require separate authorized Tasks.", "Preserve one authoritative path and fail clearly on structural ambiguity or unsupported versions; network unavailability must retain local work and be reported as pending, not masquerade as successful sync.", "Do not migrate live repositories, commit actor identity/runtime state, force-push, or rewrite published source history as part of implementation verification."]
  summary: "Independent metadata sync shipped in Tandem 0.15.0: tandem branch sync, uid identity with numbering at first sync, semantic merge and conflict resolution, safety copy, background sync, migrate/--adopt, docs, rollout guide, and Pi/sysup handoff. Live repository migration follows ~/.pi task-305 and the upgrade guide."
  evidence: ["`just release 0.15.0` exited 0: fmt, release tests, release/dist builds, clippy -D warnings, docs site build and audit, pi-tandem smokes against the 0.15.0 binary; pushed main and annotated tag tandem-v0.15.0; Release workflow verified GitHub Release body and assets (log /tmp/tandem-release-0150.log).", "Independent check: origin main = 1d5a092; tag tandem-v0.15.0 dereferences to 1d5a092; GitHub Release https://github.com/Algorant/tandem/releases/tag/tandem-v0.15.0 is non-draft, non-prerelease, with Linux/macOS x86_64/aarch64 archives, checksums, installer; `mise latest github:Algorant/tandem` = 0.15.0.", "Installer smoke: trytandem.dev/install.sh into scratch TANDEM_INSTALL_DIR printed `tandem 0.15.0`; this machine's mise-installed tandem intentionally left at 0.14.1 until the rollout.", "Two-clone regression matrix: 12 scenarios in tandem/tests/sync_behavior.rs, full suite 373 passed. AUR verification skipped per the read-only AUR rule (never-5).", "All subtasks task-49-1..10 completed with evidence; decision-9 accepted; migration rehearsed on copies of this repo, ~/.dotfiles, and ~/.pi."]
  filesChanged: ["tandem/src/project/sync.rs", "tandem/src/project/migrate.rs", "tandem/src/protocol/merge.rs", "tandem/src/protocol/ids.rs", "protocol/README.md", "protocol/plan/spec.md"]
  reviewer: "Algorant"
  updatedAt: "2026-10-01T06:38:02Z"
createdAt: "2026-09-30T02:50:14Z"
updatedAt: "2026-10-01T06:38:02Z"
assignee: "pi-orchestrator"
archivedAt: "2026-10-01T06:38:02Z"
resolution:
  outcome: "completed"
  reviewer: "Algorant"
---

## Description

## Approved direction

Algorant approved separating Tandem metadata synchronization from source-code commits after investigating task-48. A dedicated Tandem-managed metadata branch/ref on the existing Git remote provides an independent synchronization channel, analogous to a separate-purpose gh-pages branch. Tandem—not the user, Pi, or sysup—owns fetching, reconciliation, persistence, and publication of that metadata.

The desired experience is simple: open Tandem and see incoming work; save a task and have it synchronize automatically; work offline with durable, visibly pending changes; reconnect without repair scripts. Creating or completing tasks must not dirty or advance the source branch or prevent a source-only fast-forward.

This approves the architecture direction, not an unreviewed ID grammar, physical storage layout, background-process design, or migration implementation. Complete the detailed design and migration review in task-48 before starting this Task. It is a standalone implementation outcome, not an Epic.

## Core design requirements

- Use the existing Git remote and authentication; no mandatory additional hosted service or database.
- Keep portable, readable Markdown records with one clear authoritative mutation path. A separate branch alone is insufficient: unique record identities and protocol-aware reconciliation are essential.
- New Tasks, Subtasks, Decisions, and Rules need permanent identities that independent checkouts can allocate without a shared local max+1 counter. Readable short handles must never become collision-prone filenames or reference keys.
- Native sync must use causal/base revisions and validate the complete merged graph. Audit events alone cannot reconstruct current records.
- Keep actor identity worktree-local, ignored, and owned by Tandem. Source branches, linked Workers, and different machines must not share one configurable event-writer identity.
- Define new-clone discovery, workspace identity, branch/fork behavior, linked-worktree ownership, and the replacement for source-branch checkpoint/consolidation in task-48.
- Adapter/sysup changes belong to explicit later handoff Tasks in their owning repositories; do not implement them here.

## Investigation evidence

Installed Tandem 0.14.1 reproduced independent Task and Rule ID collisions; untracked-file pull blockage; successful pull followed by active/log duplicate-ID failure; same-record text conflicts; and fast-forward/consolidation refusal even for independent records without ID overlap. Immediate checkpointing alone did not resolve these problems.

A separate-channel disposable Git probe showed that source fast-forward can succeed while metadata has unpushed commits, using two branches on the same local remote. This proves transport isolation only, not a completed Tandem sync engine. The detailed protocol merge, automatic scheduling, migration, and failure recovery still require design and implementation verification.

## Scope boundary

Implement the native subsystem and its CLI/TUI diagnostics after the design gate. Do not migrate live ~/.pi, ~/.dotfiles, or this workspace during implementation tests. Do not claim that source-code conflicts or genuine contradictory task edits disappear.
