---
id: task-16
type: task
title: "CLI: init emits no JSON envelope on success"
state: todo
priority: "low"
tags: ["cli", "papercut"]
accord:
  status: "ready"
  acceptance: ["tandem init --title X --json prints one success envelope on stdout when it creates a workspace."]
  updatedAt: "2026-09-05T22:07:28Z"
createdAt: "2026-09-05T22:07:28Z"
updatedAt: "2026-09-05T22:07:28Z"
---

## Description

## Description

Preserved from published tandem-v0.12.3 task-13 during reconciliation with local delegation proposals task-13–15. Original creation: 2026-09-04T01:36:29Z. Source commit: 735c4cf738a5f4b78c2c28f043ae64cd5461f90a.

Observed while writing the Pi adapter parser against 0.12.2. `tandem init --title probe --json` creates a workspace, prints nothing, and exits 0. Repeating it produces a JSON error envelope for the existing workspace.

D53 says JSON is global with one shared envelope. An adapter should not need to special-case empty stdout plus exit 0 as success for this command.
