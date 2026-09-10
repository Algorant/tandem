---
id: task-31
type: task
title: "Publish Tandem 0.13.1 reference-links patch release"
state: "in-progress"
effort: "medium"
references: ["task-29", "task-30", "decision-4", "task-1"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "site/bun.lock"]
tags: ["docs", "release"]
accord:
  status: "blocked"
  acceptance: ["Package and lockfile identify 0.13.1, with concise curated notes for the approved reference warning/web/TUI changes.", "Required release checks pass; main and annotated tandem-v0.13.1 tag are pushed, and a non-draft GitHub Release contains the notes, all platform assets, installer, and checksums.", "Primary installer resolves to 0.13.1 and an isolated installation reports tandem 0.13.1; record downstream AUR status."]
  claimedAt: "2026-09-10T18:57:30Z"
  validation: ["$ just release 0.13.1", "Verify primary installer and installed version in disposable state."]
  constraints: ["Use the established just release path; no release-automation or adapter changes.", "Resolve the observed GHSA-7w5x-hrqm-74c2 docs dependency audit blocker with a minimal in-range lockfile update; do not bypass the security audit."]
  note: "First just release 0.13.1 attempt stopped at bun audit (GHSA-7w5x-hrqm-74c2, smol-toml 1.7.0); no tag or push occurred. Minimal lockfile fix to 1.7.1 is committed as 55d8557 and bun audit now reports no vulnerabilities. Recording the gate failure before resuming the full release retry."
  updatedAt: "2026-09-10T19:00:04Z"
createdAt: "2026-09-10T18:57:26Z"
updatedAt: "2026-09-10T19:00:04Z"
assignee: "pi-orchestrator"
---

## Description

Algorant approved the task-29/task-30 visual preview and explicitly requested committing, pushing, and the next patch release. Current package and latest GitHub Release are 0.13.0. Prepare and publish 0.13.1 using the established just release path; do not change release automation or adapters. Record AUR separately as non-blocking.
