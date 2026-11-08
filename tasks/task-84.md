---
id: task-84
uid: b9fad76c-5f47-446a-b17e-dce664872d83
type: task
kind: "papercut"
title: "Refresh stale capability descriptions in the release checklist"
state: "in-progress"
priority: "low"
relatedFiles: ["tandem/RELEASE.md"]
accord:
  status: "delivered"
  acceptance: ["tandem/RELEASE.md capability and limitation descriptions match current native command tree, protocol layout and mutation JSON support without changing the maintained release procedure."]
  claimedAt: "2026-11-08T02:59:37Z"
  deliveredAt: "2026-11-08T03:00:30Z"
  validation: ["$ tandem --help", "$ git diff --check"]
  summary: "Replaced stale duplicated capability and limitation inventories with links to current authoritative CLI, protocol, TUI, and configuration documentation."
  evidence: ["tandem/RELEASE.md capability and limitation descriptions match current native command tree, protocol layout and mutation JSON support without changing the maintained release procedure.: Checked installed tandem 0.17.4 --help and protocol/README.md: the release prose now defers command/JSON behavior and workspace layout to those sources, and no longer claims removed command families or limited mutation JSON; diff is confined to Current capabilities and Known limitations.", "tandem/RELEASE.md capability and limitation descriptions match current native command tree, protocol layout and mutation JSON support without changing the maintained release procedure.: Resolved every relative Markdown link in tandem/RELEASE.md successfully; git diff --check passed. Committed as b31ab27; only tandem/RELEASE.md changed."]
  filesChanged: ["tandem/RELEASE.md"]
  updatedAt: "2026-11-08T03:00:30Z"
createdAt: "2026-10-16T12:26:31Z"
updatedAt: "2026-11-08T03:00:30Z"
effort: "small"
assignee: "current-release-capabilities"
---

## Description

Observed during authorized 0.17.1 release: maintained release procedure/gates are current, but later Current capabilities/Known limitations prose still lists removed version/move/log/decision/papercut command families and accord accept; .tandem/board and papercuts directories; old TUI keys; and broad mutation JSON as deferred. Capture only, not a release blocker or in-place release-scope expansion. Native --help and protocol README remain authoritative. Do not rewrite actual release gates or archived records.
