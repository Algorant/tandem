---
id: task-90
uid: 8ee67079-7198-46b0-84b1-f61447d6c088
type: task
title: "Upgrade x1nano to the coordinated Tandem release when online"
state: todo
priority: "medium"
effort: "small"
references: ["task-89"]
tags: ["config", "upgrade"]
accord:
  status: "ready"
  acceptance: ["After x1nano is online, its actual mise-resolved Tandem binary reports the coordinated release (at least0.17.3) before resuming shared-board work.", "Any old open Tandem TUIs are restarted with cwd/arguments/session-board entries preserved; no agents or unrelated services are interrupted."]
  constraints: ["External availability condition: x1nano was offline at0.17.3 publication; do not claim it upgraded or repeatedly probe/wait.", "Use existing mise/SSH workflow; no source implementation, new infrastructure or profile changes."]
  updatedAt: "2026-10-21T02:35:32Z"
createdAt: "2026-10-21T02:35:32Z"
updatedAt: "2026-10-21T02:35:32Z"
---

## Description

Algorant via Herdra authorized x1nano catch-up when back online as part of0.17.3 rollout. Desktop and archbox were upgraded together to0.17.3; x1nano stilloffline in Tailscale pre/postpublication checks. Older clients ignore enabled:false, so upgrade before shared-board work. This is explicit deferred tracking, not a claimed/started upgrade.
