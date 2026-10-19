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
  status: "delivered"
  acceptance: ["Board commits (sync, safety copy, migrate) use git config user.name/user.email for author and committer, with per-field fallback to Tandem/tandem@localhost", "An end-to-end sync test against a remote whose pre-receive hook enforces the author email passes, numbers the new Task, and shows the configured identity; the fallback identity is covered", "protocol/README.md, docs/concepts/index.md and RELEASES.md describe the behavior; just dev-check passes", "0.17.2 is released and worktop's mise-installed tandem reports 0.17.2"]
  claimedAt: "2026-10-19T18:24:16Z"
  deliveredAt: "2026-10-19T18:37:43Z"
  summary: "Board commits use the repository's git identity; released as 0.17.2 and installed on worktop."
  evidence: ["sync.rs commit() reads git config user.name/user.email (per-field fallback Tandem/tandem@localhost) for author and committer", "New e2e test board_commits_use_the_repository_identity_so_author_rules_accept_them: bare remote pre-receive hook rejects non-member authors; configured repo syncs task-1 with configured identity; unconfigured repo stays task-new-* with 'tandem@localhost' is not a member; configuring then syncing numbers it task-2. Fails on 0.17.1 code, passes now", "cargo test --release: all suites pass; just dev-check passes (strict clippy after moving config.rs test module to end of file)", "Scratch repo with user.email ivan.s.santos@medtronic.com: tandem 0.17.2 board commits on remote tandem branch authored/committed as Ivan Santos <ivan.s.santos@medtronic.com>", "Commit 4c3f386; just release 0.17.2 exit 0: https://github.com/Algorant/tandem/releases/tag/tandem-v0.17.2 verified (non-draft, assets, notes), AUR workflow verified", "Worktop: mise up github:Algorant/tandem (with MISE_USE_VERSIONS_HOST=0, versions host still served 0.17.1) -> tandem --version = 0.17.2"]
  filesChanged: ["tandem/src/project/sync.rs", "tandem/tests/sync_behavior.rs", "tandem/src/protocol/config.rs", "protocol/README.md", "docs/concepts/index.md", "RELEASES.md", "tandem/Cargo.toml", "tandem/Cargo.lock"]
  updatedAt: "2026-10-19T18:37:43Z"
createdAt: "2026-10-19T18:24:11Z"
updatedAt: "2026-10-19T18:37:43Z"
assignee: "tandem-ceph-worktop"
---

## Description

Board sync commits hardcode `Tandem <tandem@localhost>` as author and committer (sync.rs `commit()`), so remotes with author push rules (GitLab "author is not a member") reject every `tandem` branch push. First sync never succeeds and records keep provisional `task-new-*` IDs. Use the effective `git config user.name`/`user.email` for author and committer, falling back per field to `Tandem`/`tandem@localhost`. No new config key, no protocol version change. Approved by Algorant via Herdra; ship as 0.17.2.
