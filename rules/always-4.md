---
id: always-4
uid: c10a7707-0886-4a59-af97-c44d8045fc65
category: always
createdAt: "2026-08-31T21:16:59Z"
updatedAt: "2026-10-30T20:11:51Z"
---
Request human review only when you cannot verify the acceptance criteria yourself. Exhaust verification first: run the code, read the output, and run the TUI in an hc terminal and read its screen with `hc read` (see AGENTS.md, Validating TUI changes). Layout, counts, and wrapping are verifiable this way and are not automatically human work; colors and attributes are not visible in `hc read` text, and temporal behavior such as flicker, resize tearing, and redraw latency is not verifiable from a snapshot, so those need a human look. When you do escalate, request review and state what you ran and what specifically stayed unresolved, because "visual work" is not a reason. When you are unsure whether something needs review, ask the orchestrator or user directly instead of parking the task.
