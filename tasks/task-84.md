---
id: task-84
uid: b9fad76c-5f47-446a-b17e-dce664872d83
type: task
kind: "papercut"
title: "Refresh stale capability descriptions in the release checklist"
state: todo
priority: "low"
relatedFiles: ["tandem/RELEASE.md"]
accord:
  status: "ready"
  acceptance: ["tandem/RELEASE.md capability and limitation descriptions match current native command tree, protocol layout and mutation JSON support without changing the maintained release procedure."]
  updatedAt: "2026-10-16T12:26:31Z"
createdAt: "2026-10-16T12:26:31Z"
updatedAt: "2026-10-16T12:26:31Z"
---

## Description

Observed during authorized 0.17.1 release: maintained release procedure/gates are current, but later Current capabilities/Known limitations prose still lists removed version/move/log/decision/papercut command families and accord accept; .tandem/board and papercuts directories; old TUI keys; and broad mutation JSON as deferred. Capture only, not a release blocker or in-place release-scope expansion. Native --help and protocol README remain authoritative. Do not rewrite actual release gates or archived records.
