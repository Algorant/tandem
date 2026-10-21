---
title: Agents
description: Point an AI agent at Tandem.
---

Add a short section to your project's `AGENTS.md` (or equivalent instructions file):

```markdown
## Tandem

This project tracks work
with Tandem.
Read project rules:
`tandem rules list`
Find work:
`tandem list`
Read a Task:
`tandem show <id>`
Claim work before starting.
Deliver with summary and
evidence.
Do not edit `.tandem/` directly.
```

Fresh `tandem init` seeds six enabled coordination rules and appends this line to existing `AGENTS.md` and `CLAUDE.md` files only: Read the current project rules with `tandem rules list` before starting work. It does not create missing instruction files, duplicate an exact existing line, or make init rerunnable. Symlinks to regular files are appended to in place; aliases of one target receive one insertion.

At the start of **each session**, read `tandem rules list --json`, check `ok` and `warnings`, and apply the returned enabled rules without reclassifying their categories. Reads use Tandem's normal local/remote refresh policy: a stale-sync warning is not a promise of current remote guidance. Report and resolve failed reads rather than silently reusing old or empty guidance. `tandem rules list --all` includes disabled records for management, not active instructions.

Turning a rule off preserves its identity, text and metadata. It does not push context to running agents. An explicit re-read returns data; the consuming agent/framework must separately apply that guidance, and old transcript content is not erased. Existing rules without an `enabled` field remain enabled without a rewrite. Before relying on disabled rules, upgrade all sharing clients to **0.17.3** and restart old clients; older versions ignore the state.

Then ask your agent to find or create a Task and work to its acceptance criteria. The Tandem CLI is the shared interface; integrations should call it rather than parse or edit board files. Pi's optional task-routing metadata belongs to its integration, not the Tandem CLI.
