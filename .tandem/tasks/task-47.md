---
id: task-47
type: task
title: "Release Tandem 0.14.1 patch (ignored .tandem checkpoint fix)"
state: "in-progress"
priority: "high"
effort: "small"
blockers: ["task-46"]
references: ["task-46"]
relatedFiles: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", "justfile"]
tags: ["release", "checkpoint"]
accord:
  status: "claimed"
  acceptance: ["Version is 0.14.1 in tandem/Cargo.toml and Cargo.lock (patch line, never 0.15), with a curated ## 0.14.1 section in RELEASES.md describing the local-only checkpoint fix.", "Project release gates pass (just release 0.14.1 notes/cargo/dist-manifest checks and native tests), main is pushed only after a successful tandem checkpoint --consolidate with clean .tandem/.", "Annotated tag tandem-v0.14.1 and its GitHub Release with dist assets and checksums are published and verified; an installed CLI reports tandem 0.14.1 and returns localOnly:true consolidated on an ignored-.tandem fixture. AUR outcome is non-blocking."]
  claimedAt: "2026-09-28T21:29:16Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml", "gh release view tandem-v0.14.1 shows assets; installed tandem --version reports 0.14.1"]
  updatedAt: "2026-09-28T21:29:16Z"
createdAt: "2026-09-28T21:16:26Z"
updatedAt: "2026-09-28T21:29:16Z"
assignee: "pi-orchestrator"
---

## Description

Separate independently deliverable patch release requested by Algorant after task-46 merges. Owned directly by the desktop Tandem orchestrator on main (release automation). No extensions/pi-tandem edits. After publication, report the release URL/tag and a tested x1nano update command.
