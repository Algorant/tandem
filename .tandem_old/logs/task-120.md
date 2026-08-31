---
id: task-120
type: task
title: "Research database-backed Tandem state and pluggable sync providers"
priority: "medium"
references: ["task-119"]
relatedFiles: ["protocol/plan/spec.md", "tandem/src"]
tags: ["protocol", "database", "sync", "research"]
createdAt: "2026-07-10T12:43:17Z"
updatedAt: "2026-07-26T14:47:20Z"
accord:
  status: "accepted"
  assignee: "worker-task-120-283168c1"
  claimedAt: "2026-07-26T14:27:04Z"
  deliveredAt: "2026-07-26T14:47:04Z"
  deliverables: ["protocol/plan/database-state-and-sync-options.md"]
  validation:
    commands: ["git diff HEAD~2..HEAD --check"]
  summary: "Delivered database-backed state and pluggable sync provider research in protocol/plan/database-state-and-sync-options.md."
  evidence: ["Integrated commit 355e76c"]
  filesChanged: ["protocol/plan/database-state-and-sync-options.md"]
  reviewer: "orchestrator"
  note: "Research deliverable covers requested options, authority model, SQLite/PostgreSQL design, provider boundaries, sync/conflict/security/migration concerns, POC stages, risks, and open questions. Acceptance is of the research deliverable, not the proposed architecture as a settled decision."
  updatedAt: "2026-07-26T14:47:15Z"
assignee: "worker-task-120-283168c1"
completedAt: "2026-07-26T14:47:20Z"
completion:
  summary: "Completed database-backed Tandem state and pluggable sync provider research; architecture remains a research proposal pending a separate decision."
  filesChanged: ["protocol/plan/database-state-and-sync-options.md"]
  validation: "Reviewed integrated research document; git diff check passed."
  reviewer: "orchestrator"
---

## Description

Research how the relevant parts of Tandem state could be stored in a database and synchronized through an open, extensible set of providers. Begin with SQLite for local storage and PostgreSQL for shared/remote synchronization, while defining a provider model that can support additional backends later.

Scope the research to determine:
- which Tandem data belongs in database-backed state, including active documents, relationships, accords, review metadata, decisions, rules, completed logs, and event/audit history;
- whether Markdown files, a database, or a hybrid model should be canonical versus derived/cache state;
- a minimal provider interface and capability model for SQLite, PostgreSQL, and future providers;
- local-first and offline behavior, bidirectional sync, change tracking, conflict detection/resolution, ordering, idempotency, and concurrent human/agent updates;
- database schema/versioning, migrations, workspace identity, portability, backup/export, and recovery;
- authentication, authorization, privacy, encryption, and multi-user/workspace boundaries;
- CLI/TUI configuration and operational UX without coupling protocol behavior to one vendor or hosted service;
- compatibility and migration paths for existing file-based `.tandem/` workspaces.

Deliverable: an options analysis and proposed architecture, including a recommended source-of-truth model, an initial SQLite/PostgreSQL design, provider extension boundaries, staged proof-of-concept plan, risks, and open questions. Keep this as research until the architecture is reviewed and recorded as a decision.
