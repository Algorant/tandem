---
id: task-99
uid: 61d45ec0-13ba-4a7f-b382-1cbdc99426fd
type: task
title: "Terminate owned Git SSH descendants when a sync timeout expires"
state: "in-progress"
priority: "high"
effort: "medium"
references: ["task-98"]
relatedFiles: ["tandem/src/project/git.rs", "tandem/Cargo.toml", "tandem/Cargo.lock"]
tags: ["sync"]
accord:
  status: "delivered"
  acceptance: ["A real Git-over-SSH fetch timeout terminates its owned Git/SSH subprocesses and returns rather than hanging on inherited stdout/stderr pipes.", "After the timeout, the board lock releases and a separate native `tandem show --json` can read the temporary board promptly.", "Normal Git-backed native board operations still work, without changing protocol, sync cadence, or release/version."]
  claimedAt: "2026-11-08T05:01:36Z"
  deliveredAt: "2026-11-08T05:04:45Z"
  validation: ["$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings", "$ cargo build --release --manifest-path tandem/Cargo.toml"]
  constraints: ["Scope only shared Git child lifecycle plus a minimal dependency/lock update if necessary.", "Do not touch real cart-lab processes, locks, records, seedbox, or recovery work.", "No maintained tests or mocked transports; remove temporary repro artifacts before delivery."]
  summary: "Git invocations now own a Unix process group and timeout sends SIGKILL to that group before waiting or joining readers; committed as 5e33995."
  evidence: ["A real Git-over-SSH fetch timeout terminates its owned Git/SSH subprocesses and returns rather than hanging on inherited stdout/stderr pipes.: Read-only SSH access to this repo's configured GitHub remote was confirmed. In the temporary-board repro, SIGSTOP was sent to the fresh direct SSH child after confirming its parent was the repro Git PID and its PGID matched that Git process group. Sync returned the timeout result; both captured Git and SSH PIDs were absent afterward.", "After the timeout, the board lock releases and a separate native `tandem show --json` can read the temporary board promptly.: Immediately after the timed-out sync exited, a separate release-binary `tandem show task-1 --json` succeeded and returned `id: task-1`.", "Normal Git-backed native board operations still work, without changing protocol, sync cadence, or release/version.: A separate temporary board with a local bare Git remote completed `tandem sync --json` promptly (expected `pending`, because its pushurl deliberately targeted a nonexistent path), and native `show --json` read task-1 successfully. Changes touch only shared Git runner and libc dependency/lock entry; no protocol or version edits."]
  filesChanged: ["tandem/src/project/git.rs", "tandem/Cargo.toml", "tandem/Cargo.lock"]
  updatedAt: "2026-11-08T05:04:45Z"
createdAt: "2026-11-08T05:01:33Z"
updatedAt: "2026-11-08T05:06:21Z"
assignee: "bound-git-timeout"
---

## Description

Approved by Herdra m122e8c27 in incident task-98. Concrete live evidence: old TUI's sync worker retained the board flock while an orphan SSH child held its stderr pipe writer open; Git had already disappeared. `git::run` kills/waits only for the direct Git process at timeout then joins read_to_end pipe readers indefinitely.

Plan first. Smallest causal fix is owned subprocess-group cleanup at the shared Git runner (supported releases Linux/macOS) so descendants cannot keep its pipes open beyond timeout. Reuse current timeout; no lock-file deletion/reader bypass/notes fallback/new orchestration framework or generic retry infrastructure. Any dependency choice must stay minimal; supported-platform boundary explicit. No live-machine recovery in this Worker; parent coordinates the incident.

Verification must drive real Git and real SSH. No mocked SSH wrapper/transport, no fabricated helper binary, no persistent fake fixtures or maintained tests. Suggested temporary E2E: a disposable local Git board uses a read-only GitHub SSH remote; identify and SIGSTOP only its own freshly spawned SSH descendant to induce the same real hang, allow TANDEM_SYNC_TIMEOUT to expire, assert own Git/SSH gone and `show` returns. No remote-board writes, no new sshd/account/profile. Explain the chosen check before implementation; if unavailable ask rather than fake it. Preserve out-of-scope legacy tests (project-wide cutover belongs to owner). No push/release.


## Parent independent E2E verification
At ready source commit 5e33995, drove the release binary on a temporary Git-backed board with a read-only GitHub SSH origin and a nonexistent temporary pushurl (so publishing externally was impossible). Identified its own Git/SSH parentage and PGID, SIGSTOPed only that fresh SSH child. With TANDEM_SYNC_TIMEOUT=2, sync returned the timeout in 1.997s; both Git and SSH PIDs were gone; a separate native show read task-1 in 0.699s. Switched the disposable origin to a local bare Git remote with pushes enabled: sync reported synced and a new rule published as numbered prefer-2. Temporary repositories and harness removed after verification. No mocked transport or live-board mutations.