---
id: always-4
category: always
createdAt: "2026-08-31T21:16:59Z"
updatedAt: "2026-08-31T21:16:59Z"
---
Request human review only when you cannot verify the acceptance criteria yourself. Exhaust verification first: run the code, read the output, spawn a Herdr pane and read the rendered result including ANSI color. Static rendering, layout, counts, and colors are verifiable this way and are not automatically human work; temporal behavior such as flicker, resize tearing, and redraw latency is not verifiable from a snapshot. When you do escalate, request review and state what you ran and what specifically stayed unresolved, because "visual work" is not a reason. When you are unsure whether something needs review, ask the orchestrator or user directly with ask_me instead of parking the task.
