# Protocol Architecture Refactor Campaign Baseline

- Campaign Epic: `task-146`
- Governance Task: `task-148`
- Architecture decision: `decision-7` (accepted; campaign records reference it)
- Governing specification: [`refactor_spec.md`](./refactor_spec.md)
- Established: 2026-07-26

## Controlled integration base

`refactor/protocol-architecture` was created locally from the approved, clean
`main` commit:

```text
355e76c9a51a90b5b41b383bbc4c4efe5ffa74e5
# docs: research database state and sync architecture
```

At branch creation, both `main` and this branch resolved to that commit and
both checked worktrees were clean. `origin/HEAD` resolves to `origin/main`;
`main` remains the default/release branch. This campaign branch is an
integration branch, not a release branch. Do not push or release from it
without separate authorization.

Before implementation begins, the Epic, its direct Task records, this
specification, and `decision-7` must continue to agree. Tandem records remain
the authoritative source for Task relationships and workflow; this document
does not replace them or authorize a Task to start.

## Synchronization and integration rules

1. Keep Rust CLI/TUI implementation work frozen on `main` for the campaign.
   Documentation, planning, extensions, and unrelated non-Rust work may
   continue there only when they do not change the frozen Rust architecture.
2. A critical Rust fix that cannot wait lands on `main` first, then is
   integrated into `refactor/protocol-architecture` immediately and passes the
   full applicable checkpoint suite.
3. Integrate other eligible `main` changes into the campaign branch at explicit
   module checkpoints. Record the source commit(s), checkpoint, and validation
   evidence in the integrating Task handoff or review record.
4. Delegated work stays on isolated Task worktrees/branches. Review it before
   integrating it into the campaign branch; never use the campaign branch as a
   substitute for Task review.
5. Preserve behavior-changing protocol work separately from move-only module
   extraction. Do not merge unfinished module boundaries to `main` merely to
   reduce branch age.
6. Merge the completed, synchronized campaign branch to `main` only after
   architecture review, behavior and lint checkpoints, documentation review,
   and required human TUI validation. `main` remains the release/default branch
   throughout.

## Initial boundary checkpoint

The intended boundaries are `protocol`, `project`, `app`, `cli`, and `tui`.
They are target ownership boundaries, not a claim that the corresponding Rust
modules already exist. Follow [`refactor_spec.md`](./refactor_spec.md) for
ownership and dependency direction; do not pre-create empty module trees or
change production Rust, dependencies, protocol behavior, releases, or package
shape under this governance task.
