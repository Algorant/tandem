---
id: task-38
type: task
title: "Stop writing chore(tandem) commits next to real work"
priority: "high"
effort: "medium"
references: ["task-37", "task-34"]
relatedFiles: ["tandem/src/project/checkpoint.rs", "tandem/tests/checkpoint_behavior.rs", "protocol/README.md", "RELEASES.md"]
tags: ["protocol", "git"]
accord:
  status: "accepted"
  acceptance: ["Going forward, git log of unpushed work shows Algorant's real commit messages only. Board files live inside those commits. Consecutive chore(tandem): checkpoint metadata commits next to real work are gone.", "Pushed commits and merges are not rewritten. Unrelated staged, unstaged, and untracked files stay untouched. No Pi tidy path, no user-run hook or just recipe.", "Independently verify main commit 5b9cd2e / archived task-37 against this contract. Keep it if it matches; rework it if it does not. Do not blindly re-implement.", "Algorant can run the behavior from the installed tandem binary on this machine (publish a release, or otherwise make this the live tandem)."]
  claimedAt: "2026-09-17T14:18:03Z"
  deliveredAt: "2026-09-17T14:18:17Z"
  validation: ["$ cargo test --manifest-path tandem/Cargo.toml --test checkpoint_behavior", "$ cargo test --manifest-path tandem/Cargo.toml", "$ git diff --check", "Disposable real-Git probe: after a real unpushed commit, claim/deliver/complete keep that commit message and do not add chore commits beside it."]
  constraints: ["Do not rewrite origin or other remote-tracking commits.", "Do not fold into merge commits.", "Do not modify Pi adapters or ~/.pi.", "Tests may mutate disposable repositories only."]
  summary: "Kept 5b9cd2e. It already folds unpushed .tandem/ into the last real commit and collapses leftover chores. Installed that release binary as the live tandem on this machine. Disposable Git probes and this repo’s claim both kept real commit messages with no new chore commits."
  evidence: ["Independent read of 5b9cd2e checkpoint.rs/protocol README/tests: unpushed non-merge HEAD is amended with .tandem/ and keeps a real message; leftover tandem-only commits fold into the neighboring real commit; pushed/merges are not rewritten.", "cargo test --manifest-path tandem/Cargo.toml --test checkpoint_behavior: 12 passed, including unpushed_ordinary_commits_absorb_tandem_and_pushed_commits_are_never_amended and leftover_adjacent_chore_runs_collapse_on_a_safe_boundary (6 commits -> ordinary source + baseline).", "cargo test --manifest-path tandem/Cargo.toml: 308 unit + all integration suites passed. git diff --check clean.", "Installed tandem/target/release/tandem over mise github-algorant-tandem 0.13.3/latest. Disposable probe: feat: real work stayed the only subject across claim/deliver/complete (amended true, 0 chores). Fold probe: chore/fix/chore/chore/docs became fix/docs/fixture (consolidated 3).", "This repo claim of task-38 with the live binary: checkpoint amended=true consolidated=3. Unpushed range is now feat(tandem): fold board files... then fix(tui): order Board and Logs... with no chore(tandem) commits."]
  filesChanged: ["tandem/src/project/checkpoint.rs", "tandem/tests/checkpoint_behavior.rs", "protocol/README.md"]
  reviewer: "pi-orchestrator"
  updatedAt: "2026-09-17T14:18:20Z"
createdAt: "2026-09-17T14:13:58Z"
updatedAt: "2026-09-17T14:18:20Z"
assignee: "pi-orchestrator"
archivedAt: "2026-09-17T14:18:20Z"
resolution:
  outcome: "completed"
  reviewer: "pi-orchestrator"
---

## Description

Algorant (from the ~/.pi session) wants clean, concise git history that is not covered in Tandem metadata commits. This is for current, ongoing, and future work. He does not want a lecture about leftover chores after push or about rewriting already-pushed history.

Desired shape, using a recent ~/.pi unpushed range as the exhibit:

Before (today with 0.13.3):
chore(tandem): checkpoint metadata
docs(tandem): retarget Pi guidance
chore(tandem): checkpoint metadata
fix(pi-herdr): track Scriptify panes
feat(pi-herdr): add tracked Scriptify pane
chore(tandem): checkpoint metadata

After:
docs(tandem): retarget Pi guidance
fix(pi-herdr): track Scriptify panes
feat(pi-herdr): add tracked Scriptify pane

Same trees, including .tandem/. Real messages stay. No extra chore commits next to real work.

Simplest rule he accepted: if HEAD is unpushed, put .tandem/ into that commit and keep its message. Fold leftover unpushed tandem-only commits into the neighboring real commit. Pi stays a client. No extra command.

Context he rejected: option-letter jargon (A/B/C), pre-push tidy scripts, restoring Pi housekeeping, jj for this now.

Archived task-37 / commit 5b9cd2e was an attempt from the ~/.pi conversation. You own verification against THIS contract. Spawn a Worker if the work is implementation/rework; ask Algorant (or the ~/.pi orchestrator) only if a product choice is blocking.

Installed tandem is still 0.13.3 until you make a new binary live.

