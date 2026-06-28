---
id: task-15
type: task
title: "Promote pi-tandem to canonical Pi config"
state: todo
priority: "low"
tags: ["pi-tandem", "config"]
createdAt: "2026-06-27T23:30:05Z"
updatedAt: "2026-06-28T02:22:39Z"
accord:
  status: "ready"
  deliverables: ["config:/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem:global extension copy", "config:/home/ivan/.dotfiles/pi/.pi/agent/config-manifest.json:manifest entries"]
  validation:
    commands: ["cd /home/ivan/.dotfiles && stow -n -v pi", "pi config check or /config-check after reload"]
  constraints: ["Do this only after project-local smoke passes.", "Never commit auth tokens, sessions, caches, logs, or private transcripts."]
  updatedAt: "2026-06-27T23:30:05Z"
---

## Description

After project-local `pi-tandem` testing passes, promote the extension into the canonical global Pi config managed by dotfiles/Stow.

Acceptance direction:
- Copy or adapt the tested extension into `~/.dotfiles/pi/.pi/agent/extensions/pi-tandem/` following the existing Pi config maintenance rules.
- Update `config-manifest.json` with the extension path, registered tools/commands, and optional dependency on `tdm`.
- Add or update a Pi skill only if workflow guidance needs more than tool prompt snippets.
- Run the Pi config check workflow and document reload/restart steps.

This task should remain deferred until the in-repo project-local extension is validated.
