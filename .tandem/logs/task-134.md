---
id: task-134
type: task
kind: "epic"
title: "Correct Epic, Task, and Subtask roles across Tandem"
priority: "critical"
references: ["decision-7", "decision-4", "task-101", "task-132", "task-133"]
relatedFiles: ["AGENTS.md", "protocol", "tandem/src", "tandem/plan", "extensions/pi-tandem", "docs", "README.md", "plan/delegated-task-tree-worker-spec.md"]
tags: ["epic", "protocol", "hierarchy", "cli", "tui", "pi-tandem", "docs"]
createdAt: "2026-07-15T17:42:36Z"
updatedAt: "2026-07-22T04:32:39Z"
accord:
  status: "accepted"
  assignee: "parent-orchestrator"
  claimedAt: "2026-07-22T04:32:15Z"
  deliveredAt: "2026-07-22T04:32:28Z"
  deliverables: ["Canonical role and strict ID semantics implemented across all project surfaces.", "Invalid hierarchy shapes fail closed with aggregated diagnostics and no legacy compatibility.", "Task-only delegation campaign boundary documented and smoke-tested.", "Tandem v0.6.0 published with curated notes, verified cargo-dist artifacts/checksums, branded installer, and tandem-bin AUR package."]
  validation:
    commands: ["154 Rust tests plus release build/version checks passed.", "Bun syntax, three pi-tandem smokes, frozen docs build, high-severity audit, and 602 link checks passed.", "Human TUI hierarchy validation approved.", "GitHub Release workflow 29891077796 and AUR workflow 29891174478 succeeded.", "Published x86_64 archive and isolated branded installer report tandem 0.6.0; /usr/bin remains unchanged at 0.5.0."]
  constraints: ["No compatibility shim for invalid decision-4 hierarchy shapes.", "Do not modify /usr/bin/tandem during release verification."]
  summary: "Decision-7 hierarchy hardening is complete across specification, CLI, TUI, repository-local Pi integration, campaign guidance, historical metadata, integration tests, and public documentation. Direct Tasks task-139 through task-143 are archived; standalone release task-144 published and verified Tandem v0.6.0 from e39fd86."
  evidence: ["Main and annotated tandem-v0.6.0 tag resolve to e39fd86534b5e69c15506a6baa2b49eb2dd1b532.", "GitHub Release: https://github.com/Algorant/tandem/releases/tag/tandem-v0.6.0", "AUR tandem-bin commit 976918deb0d2730d40382fc3b5ea49bc03ab0ca3 with matching release checksum."]
  filesChanged: ["protocol/", "tandem/src/", "tandem/plan/", "extensions/pi-tandem/", "docs/", "README.md", "plan/delegated-task-tree-worker-spec.md", "site/", "tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md"]
  reviewer: "user-and-parent-orchestrator"
  note: "All canonical hierarchy acceptance criteria, human visual validation, release publication, artifact verification, and AUR verification are satisfied."
  updatedAt: "2026-07-22T04:32:31Z"
completedAt: "2026-07-22T04:32:39Z"
completion:
  summary: "Completed decision-7 hierarchy hardening across Tandem and released the canonical Epic → global Task → parent-derived leaf Subtask model as verified v0.6.0."
  filesChanged: ["protocol/", "tandem/src/", "tandem/plan/", "extensions/pi-tandem/", "docs/", "README.md", "plan/delegated-task-tree-worker-spec.md", "site/", "tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md"]
  validation: "Tasks 139-143 and release task 144 completed. Full Rust/Bun/docs/visual matrix passed; annotated tag and main point to e39fd86; cargo-dist Release, curated assets/checksums/isolated installer, and tandem-bin AUR 0.6.0 were externally verified."
  reviewer: "user-and-parent-orchestrator"
---

## Description

Implement accepted decision-7, which fully supersedes decision-4, across every project surface so Tandem consistently enforces the canonical Epic → Task → Subtask relationship and naming boundary.

Required outcome:
- Epics and Tasks use globally allocated `task-N` IDs.
- Direct children of `kind: epic` documents are globally numbered Tasks with `epic-task` relationship semantics and canonical `parentId` links to the Epic.
- Only Subtasks beneath Tasks use parent-derived `task-N-M` IDs and `subtask` relationship semantics.
- Epics cannot have parents, Subtasks cannot have children, and invalid role-changing reparenting is rejected.
- Generic decision/custom parents retain generic Parent semantics and their children remain globally numbered Tasks.
- There is no compatibility or migration shim for direct Epic children with Subtask-shaped IDs; incorrect active records were replaced before implementation.
- Only Tasks are delegation roots initially; a Task worker executes its Subtasks through the Pi todo projection.
- Repository instructions, protocol, allocation, CLI, TUI, repository pi-tandem adapter/guidance/tests, and public documentation agree.

The direct children of this Epic are globally numbered Tasks task-139 through task-143. This repository Epic does not modify personal dotfiles; `plan/delegated-task-tree-worker-spec.md` is the explicit cross-repository handoff.
