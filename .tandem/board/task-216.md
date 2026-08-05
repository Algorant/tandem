---
id: task-216
type: task
kind: "epic"
title: "Ship the read-only Tandem web MVP"
state: todo
priority: "high"
references: ["task-121", "task-120"]
relatedFiles: ["plan/web-ui-research.md", "tandem/src", "tandem/Cargo.toml", "tandem/README.md", "docs"]
tags: ["ui", "web"]
createdAt: "2026-08-05T18:46:00Z"
updatedAt: "2026-08-05T18:46:00Z"
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
