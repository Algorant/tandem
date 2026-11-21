---
id: task-118
uid: 2b564f43-0369-4c6b-939d-8630a38fa854
type: task
title: "Remove Tandem web and release"
state: "in-progress"
priority: "high"
effort: "large"
references: ["decision-13", "task-117", "task-116"]
relatedFiles: ["tandem/", "docs/", "protocol/", "site/", "scripts/", "justfile", "README.md", "RELEASES.md", "AGENTS.md", ".github/workflows/"]
tags: ["ui", "release"]
accord:
  status: "claimed"
  acceptance: ["Web command, server, assets, dependencies and web-only checks removed; CLI/TUI and Git-backed workflow still work.", "Current docs and links resolve without advertising the removed viewer; normal site build/link checks pass.", "Normal release gates pass, annotated tag and non-draft GitHub Release with required assets and removal notes verified; AUR outcome reported separately."]
  claimedAt: "2026-11-21T03:17:55Z"
  validation: ["$ just dev-check", "$ just site-build", "$ cd site && bun run check:links", "$ git diff --check"]
  updatedAt: "2026-11-21T03:17:55Z"
createdAt: "2026-11-21T03:17:34Z"
updatedAt: "2026-11-21T03:17:55Z"
assignee: "pi:remove-tandem-web"
---

## Description

Algorant, verbatim (2026-11-21, via Herdra m4f3c0814): "ok for tandem yes deprecate and mark as an official decision. can revisit at a later date. don't worry about 'announcing' whatever that means. I am the only user. just rip it out, fix all the documentation and code, then cut a release."

Implement decision-13: remove the web command, help, server, routes, embedded assets, dev shortcut, web-only tests/checks, release gates and unused dependencies. Remove stale viewer references across code/docs/README/site, keep the public documentation site and handle old guide URLs cleanly. Preserve board data, protocol, CLI/TUI, sync, installer and normal single-binary packaging. Historical release notes may remain truthful; new release notes name the removal. No announcement stage, compatibility server, unrelated cleanup, live app restart or package install into the user environment. Leave parked draft untouched pending separate disposition.

Manus prepares and checks the removal and release candidate in isolation; starter verifies and integrates, then runs normal release from main, verifying annotated tag, GitHub Release/assets and normal AUR outcome. Do not push from child or bypass any publication gate. Choose next suitable SemVer for command removal from current tags/release plan; earlier 0.18.0 reservation for held SSH work must not be silently overridden. Report proposed version before preparing version files if reservation remains.
