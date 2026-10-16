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

Then ask your agent to find or create a Task and work to its acceptance criteria. The Tandem CLI is the shared interface; integrations should call it rather than parse or edit board files. Pi's optional task-routing metadata belongs to its integration, not the Tandem CLI.
