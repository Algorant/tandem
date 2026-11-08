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
  status: "claimed"
  acceptance: ["A real Git-over-SSH fetch timeout terminates its owned Git/SSH subprocesses and returns rather than hanging on inherited stdout/stderr pipes.", "After the timeout, the board lock releases and a separate native `tandem show --json` can read the temporary board promptly.", "Normal Git-backed native board operations still work, without changing protocol, sync cadence, or release/version."]
  claimedAt: "2026-11-08T05:01:36Z"
  validation: ["$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings", "$ cargo build --release --manifest-path tandem/Cargo.toml"]
  constraints: ["Scope only shared Git child lifecycle plus a minimal dependency/lock update if necessary.", "Do not touch real cart-lab processes, locks, records, seedbox, or recovery work.", "No maintained tests or mocked transports; remove temporary repro artifacts before delivery."]
  updatedAt: "2026-11-08T05:01:36Z"
createdAt: "2026-11-08T05:01:33Z"
updatedAt: "2026-11-08T05:01:36Z"
assignee: "bound-git-timeout"
---

## Description

Approved by Herdra m122e8c27 in incident task-98. Concrete live evidence: old TUI's sync worker retained the board flock while an orphan SSH child held its stderr pipe writer open; Git had already disappeared. `git::run` kills/waits only for the direct Git process at timeout then joins read_to_end pipe readers indefinitely.

Plan first. Smallest causal fix is owned subprocess-group cleanup at the shared Git runner (supported releases Linux/macOS) so descendants cannot keep its pipes open beyond timeout. Reuse current timeout; no lock-file deletion/reader bypass/notes fallback/new orchestration framework or generic retry infrastructure. Any dependency choice must stay minimal; supported-platform boundary explicit. No live-machine recovery in this Worker; parent coordinates the incident.

Verification must drive real Git and real SSH. No mocked SSH wrapper/transport, no fabricated helper binary, no persistent fake fixtures or maintained tests. Suggested temporary E2E: a disposable local Git board uses a read-only GitHub SSH remote; identify and SIGSTOP only its own freshly spawned SSH descendant to induce the same real hang, allow TANDEM_SYNC_TIMEOUT to expire, assert own Git/SSH gone and `show` returns. No remote-board writes, no new sshd/account/profile. Explain the chosen check before implementation; if unavailable ask rather than fake it. Preserve out-of-scope legacy tests (project-wide cutover belongs to owner). No push/release.
