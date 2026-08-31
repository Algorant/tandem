---
id: task-15
type: task
title: "Promote pi-tandem to canonical Pi config"
priority: "low"
tags: ["pi-tandem", "config"]
blockers: ["task-27"]
createdAt: "2026-06-27T23:30:05Z"
updatedAt: "2026-06-29T18:27:55Z"
accord:
  status: "delivered"
  assignee: "promote-pi-tandem-15"
  claimedAt: "2026-06-29T18:04:58Z"
  deliveredAt: "2026-06-29T18:07:32Z"
  deliverables: ["/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem/README.md", "/home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem/pi-tandem.md", "Verified existing /home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem/index.ts remains adapted for canonical relative imports.", "Verified existing /home/ivan/.dotfiles/pi/.pi/agent/config-manifest.json already contains pi-tandem extension, tools, command, and optional tandem dependency."]
  validation:
    commands: ["bun --check /home/ivan/.dotfiles/pi/.pi/agent/extensions/pi-tandem/index.ts", "TANDEM_BIN=/home/ivan/dev/projects/tandem/tandem/target/debug/tandem bun --check /home/ivan/dev/projects/tandem/extensions/pi-tandem/index.ts /home/ivan/dev/projects/tandem/extensions/pi-tandem/tests/smoke.ts /home/ivan/dev/projects/tandem/extensions/pi-tandem/tests/pi-runtime-smoke.ts /home/ivan/dev/projects/tandem/extensions/pi-tandem/tests/relationship-smoke.ts", "tandem_status cwd=/home/ivan/dev/projects/tandem", "timeout 20s env PI_CODING_AGENT_DIR=/home/ivan/.dotfiles/pi/.pi/agent pi --mode rpc --approve --offline --no-session; pi-config-check report: No drift detected.", "cd /home/ivan/.dotfiles && stow -n -v pi"]
  constraints: ["Do this only after project-local smoke passes.", "Do this only after the tandem CLI/TUI release/install target is clear.", "Never commit auth tokens, sessions, caches, logs, or private transcripts."]
  summary: "Promoted pi-tandem into canonical Pi dotfiles config. The extension source was already present under ~/.dotfiles/pi/.pi/agent/extensions/pi-tandem/index.ts with manifest entries for extension path, tools, /tandem command, and optional tandem CLI dependency; added canonical README and agent guidance docs for the promoted global copy."
  evidence: ["Dotfiles commit: 2e713898 Document promoted pi-tandem extension", "/home/ivan/.dotfiles/pi/.pi/agent/cache/pi-config-check/latest.md reports No drift detected.", "git -C /home/ivan/.dotfiles status --short was clean after commit."]
  filesChanged: ["pi/.pi/agent/extensions/pi-tandem/README.md", "pi/.pi/agent/extensions/pi-tandem/pi-tandem.md"]
  updatedAt: "2026-06-29T18:07:32Z"
completedAt: "2026-06-29T18:27:55Z"
completion:
  summary: "Closed per user direction without applying additional promotion changes. The accidental dotfiles documentation commit from the delegated herd was reset locally and not pushed."
  validation: "User directed task closure; dotfiles repo reset to prior HEAD and status verified clean."
---

## Description

After project-local `pi-tandem` testing passes, promote the extension into the canonical global Pi config managed by dotfiles/Stow.

Acceptance direction:
- Copy or adapt the tested extension into `~/.dotfiles/pi/.pi/agent/extensions/pi-tandem/` following the existing Pi config maintenance rules.
- Update `config-manifest.json` with the extension path, registered tools/commands, and optional dependency on `tandem`.
- Add or update a Pi skill only if workflow guidance needs more than tool prompt snippets.
- Run the Pi config check workflow and document reload/restart steps.

This task should remain deferred until the in-repo project-local extension is validated and the project has a clear `tandem` CLI/TUI release/install target.
