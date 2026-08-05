---
id: task-216
type: task
kind: "epic"
title: "Ship the read-only Tandem web MVP"
priority: "high"
references: ["task-121", "task-120"]
relatedFiles: ["plan/web-ui-research.md", "tandem/src", "tandem/Cargo.toml", "tandem/README.md", "docs"]
tags: ["ui", "web"]
createdAt: "2026-08-05T18:46:00Z"
updatedAt: "2026-08-05T19:41:49Z"
completedAt: "2026-08-05T19:41:49Z"
completion:
  summary: "Shipped the complete read-only Tandem web MVP: embedded loopback server, canonical read APIs, bundled responsive browser interface for all TUI views, revision polling, local security safeguards, accessibility, documentation, packaging, and comprehensive validation."
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "tandem/src/web.rs", "tandem/src/web/index.html", "tandem/src/web/app.css", "tandem/src/web/app.js", "tandem/src/web/api.js", "tandem/src/web/ui.js", "tandem/src/app/queries.rs", "tandem/src/project/mod.rs", "tandem/src/cli", "tandem/src/main.rs", "tandem/tests/cli_behavior.rs", "README.md", "tandem/README.md", "tandem/RELEASE.md", "docs/cli/index.md", "docs/web/index.md", "site/astro.config.mjs", "justfile"]
  validation: "All four direct Tasks completed. Final integrated validation passed 241 unit and 11 integration tests, formatting, strict all-feature Clippy, release/dist builds, docs build and 896 links, audit, JS syntax, packaged-runtime isolation, API/security/parity checks, and desktop/narrow/zoom/accessibility/theme Chromium evidence."
  reviewer: "orchestrator"
---

## Description

Add a simple local browser interface as a peer to the CLI and TUI. The MVP serves one discovered Tandem workspace from the existing Rust binary, reuses canonical protocol/project/app behavior, and provides read-only versions of all current TUI views.

Product decisions:
- Command: `tandem web [--port <port>] [--no-open]`.
- Bind loopback only, choose an available port by default, print the URL, and open the browser unless disabled.
- Use semantic HTML with small bundled vanilla JavaScript; no SPA framework or Node runtime.
- Include Board and Validation, task details and relationships, Logs, Rules, Decisions, and project health.
- Use simple revision polling; defer SSE.
- No mutations, remote access, authentication system, database, sync provider, or agent-feedback channel in this Epic.
- Apply same-origin security, Host validation, safe Markdown rendering, responsive Verdigris styling, keyboard accessibility, and no-CORS defaults.

The implementation must keep the web layer as a thin peer adapter over `protocol`, `project`, and `app`. It must not parse Tandem Markdown or duplicate hierarchy and lifecycle semantics.
