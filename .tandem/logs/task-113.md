---
id: task-113
type: task
title: "Replace branded install shim with real installer redirect"
priority: "high"
relatedFiles: ["site/public/install.sh", "docs/quick-start/index.md", "docs/index.md", "tandem/RELEASE.md", ".github/workflows/docs.yml"]
tags: ["docs", "release", "install", "redirect"]
createdAt: "2026-07-06T02:02:54Z"
updatedAt: "2026-07-07T23:57:19Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-06T02:03:05Z"
  deliveredAt: "2026-07-06T02:06:39Z"
  deliverables: ["Deleted site/public/install.sh so trytandem.dev no longer serves a Tandem-maintained shell wrapper.", "Updated docs/index.md and docs/quick-start/index.md to use the direct cargo-dist GitHub Release installer as the currently available install command and mark trytandem.dev/install.sh as pending provider-level redirect configuration.", "Updated docs/guides/docs-site.md and site/README.md with exact branded redirect requirements and Cloudflare Redirect Rules guidance.", "Updated tandem/RELEASE.md to remove shim validation and clarify direct installer vs pending branded redirect.", "Worker branch/worktree: shep/task-113-replace-branded-install-shim-with-real-i at /home/ivan/.pi/agent/worktrees/tandem/task-113-replace-branded-install-shim-with-real-i.", "Commit: cee85f18b0d12ceeb3a8b1ebb2099df5dc31cd61 Replace install shim with redirect docs."]
  validation:
    commands: ["cd site && bun install --frozen-lockfile: passed.", "cd site && bun run build: passed; existing non-blocking Starlight warning: Entry docs → 404 was not found.", "cd site && bun run check:links: passed, checked 598 internal docs links across 13 HTML files.", "git diff --check: passed.", "git status --short in worker worktree: clean."]
  summary: "Accepted: branded install endpoint is a real Cloudflare HTTP redirect to the cargo-dist generated GitHub Release installer, no checked-in shell shim remains. Validated HTTP 302 location and v0.4.2 release installer asset exists."
  evidence: ["Reviewed commit/diff from worktree branch shep/task-113-replace-branded-install-shim-with-real-i.", "Commit cee85f18b0d12ceeb3a8b1ebb2099df5dc31cd61."]
  filesChanged: ["site/public/install.sh", "docs/quick-start/index.md", "docs/index.md", "docs/guides/docs-site.md", "site/README.md", "tandem/RELEASE.md"]
  reviewer: "pi"
  updatedAt: "2026-07-07T23:57:13Z"
completedAt: "2026-07-07T23:57:19Z"
completion:
  summary: "Replaced the branded install shim approach with a real Cloudflare HTTP redirect to the cargo-dist generated installer and validated it live."
  validation: "curl -fsSI https://trytandem.dev/install.sh returns HTTP/2 302 with Location: https://github.com/Algorant/tandem/releases/latest/download/tandem-installer.sh. GitHub Release tandem-v0.4.2 contains tandem-installer.sh and expected platform archives/checksums. Debian container install was tested successfully after installing required xz-utils, with cargo-dist installing to ~/.cargo/bin."
  reviewer: "pi"
---

## Description

Rework the branded install URL implementation from task-110. The user does not want a Tandem-maintained shell shim at https://trytandem.dev/install.sh. Desired outcome: `curl -fsSL https://trytandem.dev/install.sh | sh` should fetch the cargo-dist generated installer itself, either through a real HTTP redirect or equivalent hosting configuration, without duplicating installer logic in a checked-in shell wrapper.

Context:
- task-109 configured cargo-dist release artifacts. cargo-dist publishes `tandem-installer.sh` at GitHub Releases, e.g. https://github.com/Algorant/tandem/releases/latest/download/tandem-installer.sh once a release exists.
- task-110 added `site/public/install.sh` as a shell-level redirect shim. This was merged as commit cdd904d but is not the desired shape.
- GitHub Pages static hosting may not support arbitrary HTTP 30x redirects directly; investigate Pages-compatible options and/or required external infra such as Cloudflare/Netlify/Vercel rules.

Acceptance criteria:
- Remove or supersede the checked-in shell shim approach (`site/public/install.sh`) unless investigation proves it is the only viable choice and user explicitly re-approves it.
- Determine the best practical way for `https://trytandem.dev/install.sh` to be a real redirect or direct generated-installer endpoint while keeping cargo-dist as source of truth.
- If implementation is possible in this repo, implement it and update docs accordingly.
- If implementation requires external DNS/hosting/provider configuration not represented in this repo, document exact required settings and update docs to avoid claiming the branded URL is live until configured.
- Keep install docs precise: distinguish direct GitHub cargo-dist URL, branded redirect URL, and any pending infra.
- Validate site build/link checks if docs/site change.
