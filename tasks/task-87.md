---
id: task-87
uid: b2b8e445-215b-4431-8c8c-e8eb3af9d6ce
type: task
title: "Board commits use the repository's git identity"
state: "in-progress"
priority: "high"
relatedFiles: ["tandem/src/project/sync.rs", "tandem/tests/sync_behavior.rs", "protocol/README.md", "docs/concepts/index.md", "RELEASES.md"]
tags: ["protocol", "sync"]
accord:
  status: "claimed"
  acceptance: ["Board commits (sync, safety copy, migrate) use git config user.name/user.email for author and committer, with per-field fallback to Tandem/tandem@localhost", "An end-to-end sync test against a remote whose pre-receive hook enforces the author email passes, numbers the new Task, and shows the configured identity; the fallback identity is covered", "protocol/README.md, docs/concepts/index.md and RELEASES.md describe the behavior; just dev-check passes", "0.17.2 is released and worktop's mise-installed tandem reports 0.17.2"]
  claimedAt: "2026-10-19T18:24:16Z"
  updatedAt: "2026-10-19T18:24:16Z"
createdAt: "2026-10-19T18:24:11Z"
updatedAt: "2026-10-19T18:24:16Z"
assignee: "tandem-ceph-worktop"
---

## Description

Board sync commits hardcode `Tandem <tandem@localhost>` as author and committer (sync.rs `commit()`), so remotes with author push rules (GitLab "author is not a member") reject every `tandem` branch push. First sync never succeeds and records keep provisional `task-new-*` IDs. Use the effective `git config user.name`/`user.email` for author and committer, falling back per field to `Tandem`/`tandem@localhost`. No new config key, no protocol version change. Approved by Algorant via Herdra; ship as 0.17.2.
