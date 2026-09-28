---
id: task-47
type: task
title: "Release Tandem 0.14.1 patch (ignored .tandem checkpoint fix)"
priority: "high"
effort: "small"
blockers: ["task-46"]
references: ["task-46"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "justfile"]
tags: ["release", "checkpoint"]
accord:
  status: "accepted"
  acceptance: ["Version is 0.14.1 in tandem/Cargo.toml and Cargo.lock (patch line, never 0.15), with a curated ## 0.14.1 section in RELEASES.md describing the local-only checkpoint fix.", "Project release gates pass (just release 0.14.1 notes/cargo/dist-manifest checks and native tests), main is pushed only after a successful tandem checkpoint --consolidate with clean .tandem/.", "Annotated tag tandem-v0.14.1 and its GitHub Release with dist assets and checksums are published and verified; an installed CLI reports tandem 0.14.1 and returns localOnly:true consolidated on an ignored-.tandem fixture. AUR outcome is non-blocking."]
  claimedAt: "2026-09-28T21:29:16Z"
  deliveredAt: "2026-09-28T21:36:33Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml", "gh release view tandem-v0.14.1 shows assets; installed tandem --version reports 0.14.1"]
  summary: "Published Tandem 0.14.1 patch: version bump + RELEASES.md notes, gates passed via just release 0.14.1 (371 tests, clippy -D warnings, fmt, dist manifest, pi-tandem read-only smokes), main pushed after consolidated checkpoint, annotated tag tandem-v0.14.1 and GitHub Release with assets/checksums verified."
  evidence: ["https://github.com/Algorant/tandem/releases/tag/tandem-v0.14.1 non-draft with installer, 4 platform tarballs, sha256.sum", "just release 0.14.1 exit 0: 'Release tandem-v0.14.1 and GitHub assets/notes verified.' AUR step skipped (read-only, non-blocking per never-5)", "mise upgrade github:Algorant/tandem: 0.13.7 -> 0.14.1; tandem --version = tandem 0.14.1; installed binary sha256 50a1223c... equals release x86_64-linux tarball binary (tarball .sha256 OK)", "Installed CLI on ignored-.tandem fixture with upstream: checkpoint --consolidate --json -> status consolidated, localOnly true, collapsed 0, HEAD unchanged, git status clean"]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
  updatedAt: "2026-09-28T21:36:33Z"
createdAt: "2026-09-28T21:16:26Z"
updatedAt: "2026-09-28T21:36:33Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-28T21:36:33Z"
resolution:
  outcome: "completed"
---

## Description

Separate independently deliverable patch release requested by Algorant after task-46 merges. Owned directly by the desktop Tandem orchestrator on main (release automation). No extensions/pi-tandem edits. After publication, report the release URL/tag and a tested x1nano update command.
