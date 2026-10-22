---
id: task-92
uid: 903edb6b-5b88-4c2d-9b32-277488a49c51
type: task
kind: "research"
title: "Plan live editable SSH workspaces in the Tandem TUI"
state: todo
priority: "medium"
effort: "medium"
relatedFiles: ["tandem/src/app/project.rs", "tandem/src/cli/model.rs", "tandem/src/cli/commands.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/editor.rs"]
tags: ["tui", "ssh"]
accord:
  status: "ready"
  acceptance: ["A source-grounded plan is sent through Herdra to Algorant BEFORE implementation, naming read/write commands, live refresh, touched modules, approximate size and material risks, with no more than four questions.", "The plan honors on-demand owner-machine Tandem execution over SSH, fully editable remote TUI boards, explicit compatible-version errors, and no copies/mounts/always-on service or special offline machinery."]
  validation: ["$ tandem tui --help"]
  constraints: ["Planning/read-only investigation only; no source/protocol changes, remote-board writes, new infrastructure or implementation dispatch until Algorant approves the plan.", "Herdra research task39's mirror/readonly recommendation is superseded by Algorant's approved SSH-owner execution shape; do not implement htt here."]
  updatedAt: "2026-10-22T18:48:19Z"
createdAt: "2026-10-22T18:48:19Z"
updatedAt: "2026-10-22T18:48:19Z"
---

## Description

Algorant via Herdra m6b55556c requests native remote addresses such as --add-workspace cartlab:/home/ivan/cart-lab using ssh aliases. Remote machine's own compatible Tandem must carry reads and writes; ownership/locks/sync remain there. Live full editable TUI like local; nothing left running between on-demand operations, no copies/mounts/network web service. Connectivity given; fail clearly instead of offline frameworks. Read ~/Projects/herdra/research/task-39.md as prior-source investigation, not current direction. Plan few lines plus <=3–4questions, then approval before code. Future delivery is a minor Tandem release and coordinated desktop/archbox/x1nano upgrades, followed by Herdra task41 htt integration/realcartlab verification with Algorant.
