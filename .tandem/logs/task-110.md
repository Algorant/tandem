---
id: task-110
type: task
title: "Expose branded trytandem.dev install script redirect"
priority: "high"
parentId: "task-108"
relatedFiles: ["site/", "docs/", "tandem/RELEASE.md"]
tags: ["docs", "release", "install"]
createdAt: "2026-07-05T17:08:25Z"
updatedAt: "2026-07-07T23:57:28Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-05T22:12:40Z"
  deliveredAt: "2026-07-05T23:54:10Z"
  deliverables: ["Added site/public/install.sh as a minimal branded shell redirect shim for `curl -fsSL https://trytandem.dev/install.sh | sh`, forwarding to cargo-dist's latest GitHub Release installer.", "Updated docs/index.md and docs/quick-start/index.md to make the branded installer the primary install path and document user-local/PATH fallback behavior.", "Updated tandem/RELEASE.md with branded install target notes and release validation checks.", "Worker branch/worktree: shep/task-110-expose-branded-trytandem-dev-install-scr at /home/ivan/.pi/agent/worktrees/tandem/task-110-expose-branded-trytandem-dev-install-scr.", "Commit: 248b507 Expose branded install shim."]
  validation:
    commands: ["sh -n site/public/install.sh: passed.", "cd site && bun install --frozen-lockfile: passed.", "cd site && bun run build: passed; existing non-blocking Starlight warning: Entry docs → 404 was not found.", "cd site && bun run check:links: passed, checked 591 internal docs links across 13 HTML files.", "test -f site/dist/install.sh: passed.", "rg cargo-dist latest installer URL in site/dist/install.sh: passed.", "git diff --check: passed.", "git status --short in worker worktree: clean."]
  summary: "Accepted as superseded/fulfilled by the final redirect implementation: the branded install command now works through task-113's real HTTP redirect to cargo-dist's installer, rather than the earlier checked-in shim."
  evidence: ["Reviewed commit/diff from worktree branch `shep/task-110-expose-branded-trytandem-dev-install-scr`.", "Commit 248b507."]
  filesChanged: ["site/public/install.sh", "docs/quick-start/index.md", "docs/index.md", "tandem/RELEASE.md"]
  reviewer: "pi"
  updatedAt: "2026-07-07T23:57:22Z"
completedAt: "2026-07-07T23:57:28Z"
completion:
  summary: "Closed branded install URL work as fulfilled by the final real-redirect implementation from task-113, superseding the earlier checked-in shim."
  validation: "Confirmed https://trytandem.dev/install.sh returns a real HTTP 302 redirect to the cargo-dist GitHub Release installer. Release tandem-v0.4.2 includes tandem-installer.sh. Container install path validated via cargo-dist installer after adding Debian xz-utils prerequisite."
  reviewer: "pi"
---

## Description

Add the primary user-facing install command via trytandem.dev while keeping cargo-dist as the installer source of truth.

Requirements:
- Provide a branded install URL such as https://trytandem.dev/install.sh for `curl -fsSL https://trytandem.dev/install.sh | sh`.
- The branded endpoint should redirect to the cargo-dist generated installer rather than maintaining a separate custom installer initially.
- Installer behavior should detect OS/architecture and install the matching release binary.
- Default install should be user-local/no-sudo, likely ~/.local/bin or cargo-dist equivalent.
- Document the install command and path behavior on the docs site.
- Include fallback instructions if ~/.local/bin is not on PATH.
