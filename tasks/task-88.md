---
id: task-88
uid: 46aebb1d-3297-4a8f-a81f-8ca95f3a6dd2
type: task
kind: "papercut"
title: "mise picks up a new Tandem release late through its versions host"
state: "in-progress"
priority: "low"
tags: ["docs", "release"]
accord:
  status: "claimed"
  acceptance: ["tandem/RELEASE.md's post-release steps (or the release notes' upgrade guidance) tell mise users how to pick up a just-published release immediately, e.g. MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem, and to confirm with tandem --version"]
  claimedAt: "2026-11-08T03:00:39Z"
  validation: ["$ mise settings get use_versions_host", "$ tandem --version", "$ git diff --check"]
  updatedAt: "2026-11-08T03:00:39Z"
createdAt: "2026-10-19T18:37:51Z"
updatedAt: "2026-11-08T03:00:39Z"
effort: "small"
relatedFiles: ["tandem/RELEASE.md"]
assignee: "fresh-mise-upgrades"
---
Right after the 0.17.2 release, normal `mise up github:Algorant/tandem` still resolved the previous version via mise's versions host. `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem` installed the new release immediately.

## Staleness check
The workaround was used successfully for the later 0.17.3 and 0.17.4 desktop/homelab upgrades, but the maintained release checklist still has no mise upgrade guidance. Current `mise settings get use_versions_host` remains true. Narrow fix: document the existing one-command bypass and `tandem --version` verification in tandem/RELEASE.md's post-release procedure. No global mise setting change, no package-manager infrastructure or upstream report.