---
id: task-49-8
type: task
title: "Step 8: end-to-end verification, documentation, and release"
priority: "high"
effort: "medium"
parentId: "task-49"
tags: ["protocol", "sync", "docs"]
accord:
  status: "accepted"
  acceptance: ["The native two-clone matrix from the design's verification plan passes, asserting exact record content, references, event integrity, and unchanged source HEAD, index, and working-tree bytes.", "CLI, workspace, and agent docs describe independent sync and state that old code checkouts no longer show historical boards.", "The release is published per the release rules (tag plus GitHub Release) and installable through mise."]
  claimedAt: "2026-10-01T06:30:00Z"
  deliveredAt: "2026-10-01T06:38:02Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml", "$ git diff --check"]
  summary: "Published Tandem 0.15.0 (protocol 0.4.0 independent sync) with docs, release notes, and verified assets."
  evidence: ["`just release 0.15.0` exited 0: fmt, release tests, release/dist builds, clippy -D warnings, docs site build and audit, pi-tandem smokes against the 0.15.0 binary; pushed main and annotated tag tandem-v0.15.0; Release workflow verified GitHub Release body and assets (log /tmp/tandem-release-0150.log).", "Independent check: origin main = 1d5a092; tag tandem-v0.15.0 dereferences to 1d5a092; GitHub Release https://github.com/Algorant/tandem/releases/tag/tandem-v0.15.0 is non-draft, non-prerelease, with Linux/macOS x86_64/aarch64 archives, checksums, installer; `mise latest github:Algorant/tandem` = 0.15.0.", "Installer smoke: trytandem.dev/install.sh into scratch TANDEM_INSTALL_DIR printed `tandem 0.15.0`; this machine's mise-installed tandem intentionally left at 0.14.1 until the rollout.", "Two-clone regression matrix: 12 scenarios in tandem/tests/sync_behavior.rs, full suite 373 passed. AUR verification skipped per the read-only AUR rule (never-5)."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "docs/workspace/index.md", "docs/cli/index.md", "docs/guides/agents-and-adapters.md", "docs/guides/upgrading-to-independent-sync.md", "plan/task-49-pi-handoff.md"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-10-01T06:38:02Z"
createdAt: "2026-09-30T03:54:18Z"
updatedAt: "2026-10-01T06:38:02Z"
assignee: "pi-orchestrator"
archivedAt: "2026-10-01T06:38:02Z"
resolution:
  outcome: "completed"
  reviewer: "pi-orchestrator"
---

