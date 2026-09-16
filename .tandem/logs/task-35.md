---
id: task-35
type: task
title: "Publish Tandem 0.13.3 native checkpoint-collapse release"
priority: "high"
effort: "medium"
references: ["task-34", "decision-4"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
tags: ["docs"]
accord:
  status: "accepted"
  acceptance: ["Package and lockfile identify 0.13.3 and concise curated release notes describe native unpushed tandem-only checkpoint amend/reconcile, amended/consolidated JSON, and removal of just tidy-history.", "Required release checks pass; main and annotated tandem-v0.13.3 tag are pushed and a non-draft/non-prerelease GitHub Release contains curated notes, four platform archives, installer and checksums.", "All published checksums and the primary branded installer are verified; isolated installation reports tandem 0.13.3 and passes a relevant CLI smoke. Record the actual non-blocking AUR result."]
  claimedAt: "2026-09-16T17:43:07Z"
  deliveredAt: "2026-09-16T17:50:30Z"
  validation: ["$ just release 0.13.3", "Verify branded installer, isolated installed binary, release checksums and actual AUR workflow result."]
  summary: "Published Tandem 0.13.3 with native unpushed tandem-only checkpoint amend/reconcile. Main and annotated tag are pushed; GitHub Release, artifacts/checksums, primary installer, and AUR git push are verified."
  evidence: ["just release 0.13.3 exited 0 after a rustfmt prepare commit: formatting, release tests, release/dist builds, strict Clippy, docs build/security audit, web syntax and project adapter smoke tests passed; Release workflow 35130118853 completed and the recipe verified curated notes and all required nonempty assets.", "Annotated tandem-v0.13.3 points to 313554a0d39eedf73f656f0fba753763316d4d65, the pushed release commit. https://github.com/Algorant/tandem/releases/tag/tandem-v0.13.3 is published, not draft or prerelease, with all four platform archives, installer, source tarball, and aggregate/per-archive checksums.", "/tmp/tandem-release-0.13.3-verify.sh exited 0: aggregate and per-archive SHA-256 checksums passed; https://trytandem.dev/install.sh was byte-identical to the published installer and declared APP_VERSION 0.13.3. Isolated HOME install reported tandem 0.13.3 and init created .tandem/tandem.md.", "AUR workflow 35130502702 completed successfully and pushed aur.archlinux.org/tandem-bin.git e80e909..dd3572a (Update tandem-bin to 0.13.3). Fresh AUR RPC still listed tandem-bin Version 0.13.2-1 (index lag); not treated as a release blocker."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "tandem/src/project/checkpoint.rs", "tandem/src/tui/validation.rs", "tandem/src/tui/workflow_prompt.rs", "tandem/tests/checkpoint_behavior.rs"]
  updatedAt: "2026-09-16T17:50:35Z"
createdAt: "2026-09-16T17:43:03Z"
updatedAt: "2026-09-16T17:50:35Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-16T17:50:35Z"
resolution:
  outcome: "completed"
---

## Description

Algorant asked for a clean checkout, commit/push of current main, then a small 0.13.3 release for native unpushed tandem-only checkpoint collapse. Use the established just release path without changing release automation or unrelated backlog. Verify artifacts/checksums and the branded installer in disposable state; inspect the actual AUR outcome.
