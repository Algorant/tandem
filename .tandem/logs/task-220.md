---
id: task-220
type: task
title: "Validate and document the Tandem web MVP"
priority: "medium"
parentId: "task-216"
blockers: ["task-217", "task-218", "task-219"]
references: ["task-121"]
relatedFiles: ["tandem/README.md", "tandem/RELEASE.md", "docs", "RELEASES.md"]
tags: ["docs", "web", "validation", "packaging"]
createdAt: "2026-08-05T18:46:31Z"
updatedAt: "2026-08-05T19:41:39Z"
accord:
  status: "accepted"
  assignee: "worker-task-220-00a81d60"
  claimedAt: "2026-08-05T19:24:01Z"
  deliveredAt: "2026-08-05T19:41:28Z"
  deliverables: ["Public Web documentation and navigation covering startup, scope, security, appearance, accessibility, and deferred capabilities.", "CLI web help and release validation checks for dist builds, strict Clippy, and bundled JavaScript syntax.", "Keyboard focus and skip-link corrections plus comprehensive packaged-binary/browser evidence."]
  validation:
    commands: ["Post-integration 241 unit and 11 integration tests passed.", "Formatting, strict all-feature Clippy, release build, and dist profile build passed.", "Docs built 19 pages including /web/; 896 internal links and high-severity Bun audit passed.", "JavaScript syntax and diff checks passed.", "Reviewed packaged-binary evidence for every view, CLI/API parity, security, polling, narrow layout, 200% zoom, reduced motion, semantic landmarks, dark mode, and Verdigris light palette."]
  summary: "Completed final documentation, packaging, cross-interface, browser, accessibility, security, and release validation for the read-only Tandem web MVP."
  evidence: ["Integrated commit d330fed into main via Worktrunk.", "Dist binary evidence proved embedded assets run outside the source tree without Node or network runtime dependencies.", "Known Astro 404-entry warning remains non-blocking; mutations, remote/auth, SSE/WebSockets, database/sync, multi-workspace, and agent feedback remain explicitly deferred."]
  filesChanged: ["README.md", "docs/cli/index.md", "docs/web/index.md", "justfile", "site/astro.config.mjs", "tandem/README.md", "tandem/RELEASE.md", "tandem/src/cli/mod.rs", "tandem/src/web.rs", "tandem/src/web/app.js"]
  reviewer: "orchestrator"
  note: "Accepted after integrated documentation/code review, full Rust/docs/dist validation, and review of packaged Chromium evidence for all required views, responsive layouts, accessibility, security, refresh, and theme variants."
  updatedAt: "2026-08-05T19:41:34Z"
assignee: "worker-task-220-00a81d60"
completedAt: "2026-08-05T19:41:39Z"
completion:
  summary: "Validated and documented the complete read-only Tandem web MVP across all views, packaged assets, CLI/API parity, security, polling, accessibility, responsive layouts, theme variants, and release checks."
  filesChanged: ["README.md", "docs/cli/index.md", "docs/web/index.md", "justfile", "site/astro.config.mjs", "tandem/README.md", "tandem/RELEASE.md", "tandem/src/cli/mod.rs", "tandem/src/web.rs", "tandem/src/web/app.js"]
  validation: "Integrated commit d330fed reviewed; 241 unit and 11 integration tests, formatting, strict all-feature Clippy, release/dist builds, 19-page docs build, 896 links, audit, JS syntax, packaged-runtime, API/security/parity, and full Chromium evidence passed."
  reviewer: "orchestrator"
---

## Description

Complete cross-interface validation, packaging checks, and concise user documentation for the read-only web mode.

Acceptance criteria:
- Document startup, default browser behavior, `--port`, `--no-open`, read-only scope, loopback boundary, and deferred capabilities.
- Verify all web views against representative workspace data and canonical CLI/TUI meaning.
- Run Rust formatting, full tests, strict Clippy, docs build/link checks, and release-build/version checks.
- Verify bundled assets work from the packaged binary without the source tree or Node runtime.
- Perform desktop, narrow-screen, keyboard-only, Default Dark, and Verdigris browser smoke checks.
- Record remaining mutation, remote-access, SSE, database, and agent-feedback work as deferred rather than silently expanding this Epic.
