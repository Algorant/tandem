---
id: task-18
type: task
title: "Release Tandem 0.12.4"
priority: "high"
tags: ["release"]
accord:
  status: "accepted"
  acceptance: ["Version and curated release notes describe the assignment, TUI workflow, native checkpointing, and safe development recipe changes; release validations pass or any blocker is explicitly resolved before publication.", "Push main and annotated tandem-v0.12.4 tag, publish the matching GitHub Release with required platform archives, installer and checksums, and verify the published assets.", "Report exact release commit/URL and availability; do not install the binary or mutate Pi adapters implicitly. Record AUR as downstream nonblocking packaging per project rules."]
  claimedAt: "2026-09-06T00:20:19Z"
  deliveredAt: "2026-09-06T00:36:15Z"
  constraints: ["Use existing release automation and checks; no force push or hook bypass.", "Native checkpointing replaces adapter-owned checkpointing only through a separately coordinated Pi cutover; include the behavior change in curated notes.", "Do not claim a pushed tag alone is a completed release."]
  summary: "Published Tandem0.12.4 from1af1db80b4e10c302794dd75c1f45afc95e960a2; main and annotated tag pushed, GitHub Release and all assets verified."
  evidence: ["Release URL:https://github.com/Algorant/tandem/releases/tag/tandem-v0.12.4; annotated tag resolves to1af1db80b4e10c302794dd75c1f45afc95e960a2.", "Authoritative just release0.12.4 passed notes/dist manifest checks, cargo fmt, optimized full tests/build/dist build, strict Clippy, docs build/audit (no vulnerabilities), web syntax and three adapter smoke scripts. Three lint errors and obsolete adapter import were fixed before any publication.", "GitHub Release workflow34001441498 succeeded for Linux x86_64/aarch64 and macOS x86_64/aarch64, global installer assets and non-draft/non-prerelease publication.", "Downloaded all four platform archives and source archive; every entry in published sha256.sum verified. Installer is nonempty and syntax-valid; https://trytandem.dev/install.sh resolves to script with APP_VERSION=0.12.4.", "Executed downloaded Linux x86_64 binary: reports0.12.4 and passes disposable real-Git assignment-token, claim/delivery/complete checkpoint JSON and archived-evidence smoke. No global install or Pi adapter cutover performed.", "AUR downstream workflow34001610847 completed successfully, despite outdated recipe comment about read-only AUR. AUR was not treated as a release blocker."]
  filesChanged: ["RELEASES.md", "tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/src/app/accord.rs", "tandem/src/project/checkpoint.rs", "tandem/src/tui/workflow_prompt.rs", "extensions/pi-tandem/index.ts", "extensions/pi-tandem/tests/smoke.ts", "extensions/pi-tandem/README.md"]
  updatedAt: "2026-09-06T00:36:23Z"
createdAt: "2026-09-06T00:20:14Z"
updatedAt: "2026-09-06T00:36:23Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-06T00:36:23Z"
resolution:
  outcome: "completed"
---

