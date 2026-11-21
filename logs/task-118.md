---
id: task-118
uid: 2b564f43-0369-4c6b-939d-8630a38fa854
type: task
title: "Remove Tandem web and release"
priority: "high"
effort: "large"
references: ["decision-13", "task-117", "task-116"]
relatedFiles: ["tandem/", "docs/", "protocol/", "site/", "scripts/", "justfile", "README.md", "RELEASES.md", "AGENTS.md", ".github/workflows/"]
tags: ["ui", "release"]
accord:
  status: "accepted"
  acceptance: ["Web command, server, assets, dependencies and web-only checks removed; CLI/TUI and Git-backed workflow still work.", "Current docs and links resolve without advertising the removed viewer; normal site build/link checks pass.", "Normal release gates pass, annotated tag and non-draft GitHub Release with required assets and removal notes verified; AUR outcome reported separately."]
  claimedAt: "2026-11-21T03:17:55Z"
  deliveredAt: "2026-11-21T03:32:08Z"
  validation: ["$ just dev-check", "$ just site-build", "$ cd site && bun run check:links", "$ git diff --check"]
  summary: "Tandem web was removed under decision-13 and Tandem 0.19.0 was published. The normal release job verified the GitHub Release and required assets/notes; AUR verification timed out."
  evidence: ["3f7419f: 27 files, +33/-2757; signed-off scope is CLI/server/assets/deps/checks/docs removal, no board migration. Annotated tandem-v0.19.0 and main push confirmed by release log.", "just dev-check, Git-backed CLI/TUI smoke, dist build, clippy, docs build, 171 internal links, high-severity Bun audit and diff checks passed. Candidate rejected tandem web with exit 2; built /web/ redirects to /tui/.", "https://github.com/Algorant/tandem/releases/tag/tandem-v0.19.0 — release job verified published assets and removal notes before the network failure. AUR run 44861347249 exists, but GitHub API timed out reading it; outcome unknown, not a verified packaging failure. Independent installer smoke could not start: trytandem.dev DNS lookup failed. No user install or restart."]
  updatedAt: "2026-11-21T04:15:06Z"
createdAt: "2026-11-21T03:17:34Z"
updatedAt: "2026-11-21T04:15:06Z"
assignee: "pi:remove-tandem-web"
archivedAt: "2026-11-21T04:15:06Z"
resolution:
  outcome: "completed"
  note: "Merged by worker_finish: remove-tandem-web → main at c2449ff669d6b77ef8bd69158a635258716dfd9a."
---

## Description

Algorant, verbatim (2026-11-21, via Herdra m4f3c0814): "ok for tandem yes deprecate and mark as an official decision. can revisit at a later date. don't worry about 'announcing' whatever that means. I am the only user. just rip it out, fix all the documentation and code, then cut a release."

Implement decision-13: remove the web command, help, server, routes, embedded assets, dev shortcut, web-only tests/checks, release gates and unused dependencies. Remove stale viewer references across code/docs/README/site, keep the public documentation site and handle old guide URLs cleanly. Preserve board data, protocol, CLI/TUI, sync, installer and normal single-binary packaging. Historical release notes may remain truthful; new release notes name the removal. No announcement stage, compatibility server, unrelated cleanup, live app restart or package install into the user environment. Leave parked draft untouched pending separate disposition.

Manus prepares and checks the removal and release candidate in isolation; starter verifies and integrates, then runs normal release from main, verifying annotated tag, GitHub Release/assets and normal AUR outcome. Do not push from child or bypass any publication gate. Release 0.19.0 (confirmed by Herdra m81c63059); 0.18.0 stays reserved for the parked SSH work. No change to that work or its reservation.
