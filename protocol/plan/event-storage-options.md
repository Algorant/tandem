# Event log storage options

Status: research note; no protocol decision
Date: 2026-07-12
Related: `protocol/plan/spec.md`, Tandem task `task-119`

## Problem and terminology

Tandem currently writes minimal, append-only lifecycle records to per-actor `.tandem/events/<actor_id>.jsonl` files. Per-actor files avoid a shared append hotspot, but routine commands still dirty the user's project checkout and encourage commits containing low-level operational noise.

The product should present three distinct kinds of history:

- **Workflow history**: the user-facing timeline assembled from task, decision, accord, and review changes. It is a view, not necessarily one file or one storage class.
- **Completed-work logs**: durable archived Markdown documents under `.tandem/logs/`. They are portable project records and the source of truth for completed work.
- **Audit events**: low-level append-only records used to enrich timelines and diagnose who changed what. They must not be required to reconstruct the current board or completed logs.

CLI/TUI labels should use **History** for the readable workflow timeline, **Completed work** (or **Logs**) for archived task documents, and **Audit events** for the raw ledger. Avoid calling all three “logs.” A diagnostic command could report `event storage: workspace | user-state | git-ref | disabled/checkpointed`, its resolved location, whether it is tracked, and backup status.

## Evaluation criteria

A useful default should preserve local-first operation, avoid routine Git dirtiness, tolerate concurrent writers, survive ordinary workspace use, and remain explainable. Alternatives also differ in portability, collaboration, audit durability, repository identity/relocation, backup/recovery, privacy, discoverability, performance, compatibility, and implementation cost.

## Options

### 1. Workspace-local, tracked per-actor files (current model)

Keep `.tandem/events/<actor_id>.jsonl` in the normal project history.

**Strengths**

- Fully local-first, visible, portable with clone/fork/archive, and naturally covered by existing Git backup.
- Per-actor ownership gives simple append behavior and usually clean merges across actors.
- Best audit durability and collaboration when every participant commits and exchanges event files.
- Lowest compatibility and implementation cost.

**Weaknesses**

- Every mutation dirties the checkout and produces noisy commits, reviews, rebases, and pull requests.
- Concurrent use of the same actor ID still conflicts; long-lived JSONL files grow without bound.
- Events may expose names, activity times, task summaries, or operational metadata to every repository reader and fork.
- Selectively omitting generated changes undermines audit completeness.

**Fit:** an explicit `tracked` mode for projects that value a Git-audited ledger more than repository quietness, not the best general default.

### 2. Workspace-local but Git-ignored event storage

Continue writing `.tandem/events/`, but add it to `.gitignore` (project-wide) or an exclude mechanism (`.git/info/exclude`) and keep tracked board/log documents unchanged.

**Strengths**

- Simple, local-first, discoverable beside the workspace, fast, and nearly compatible with current paths/readers.
- Stops routine Git dirtiness and preserves per-actor append/concurrency behavior.
- Repository relocation and copies carry events when the whole directory, rather than only tracked files, is moved.

**Weaknesses**

- A fresh clone has no history; Git backup, collaboration, and audit durability disappear unless a separate sync/backup is added.
- Project `.gitignore` is itself a tracked policy change; `.git/info/exclude` is invisible and machine-specific. Global ignore rules are even less discoverable.
- `git clean`, disposable worktrees, CI checkouts, archive/export commands, and some editor cleanup flows can delete ignored data.
- Events remain inside the project tree, where broad backup/sharing tools may disclose them accidentally.
- Existing tracked event files remain tracked after adding an ignore rule and need explicit untracking/migration.

**Fit:** a low-cost compatibility mode or migration bridge. It solves Git noise but is too fragile as the sole durable default.

### 3. External user-state directory keyed to workspace identity

Store events outside the repository, for example under `$XDG_STATE_HOME/tandem/workspaces/<workspace-key>/events/<actor_id>.jsonl` (falling back to `~/.local/state/tandem/...`). Keep board documents and completed logs in `.tandem/`.

A stable key should not be only the absolute path: paths change, symlinks alias locations, and clones may coexist. Prefer a generated opaque `workspaceId` in `.tandem/tandem.md`, combined internally with a per-checkout instance ID when clones must not silently share a ledger. Repository remote URL can assist discovery but is unsuitable as canonical identity because it changes, can be absent, and can reveal private locations.

**Strengths**

- Local-first and quiet: normal event activity never dirties the project checkout.
- Better privacy boundary and clean separation between portable authored state and machine/runtime state.
- Per-actor files retain cheap appends and independent-writer concurrency. Performance does not depend on Git status/index operations.
- Supports retention, compaction, permissions, and backup policy independently of project source.

**Weaknesses**

- Events do not travel with clone/archive by default and are not automatically shared among collaborators.
- Requires identity, relocation, orphan cleanup, discovery, export/import, and backup/restore UX.
- Reusing one `workspaceId` across multiple local clones can merge unrelated local timelines unless checkout identity is modeled; generating a new ID for every clone loses continuity.
- State-directory loss removes audit history, though current board and completed logs remain intact by design.
- Multi-machine collaboration needs an explicit export/sync backend rather than assuming Git provides it.

**Fit:** strongest default if Tandem treats events as optional audit enrichment rather than canonical project data.

### 4. Separate Git ref, branch, worktree, or repository

Write events to a dedicated Git history, such as an orphan `refs/tandem/events` branch, an attached hidden worktree, or a sidecar repository under user state.

**Strengths**

- Keeps the main branch clean while retaining versioning, replication, integrity checks, and familiar backup tooling.
- A sidecar repository can be private or pushed to a different remote. Dedicated commits can batch many event appends.
- Git refs can remain associated with repository identity better than absolute-path state.

**Weaknesses**

- Git branches do not provide a writable filesystem without index/worktree or plumbing operations. Hidden worktrees add lifecycle and locking complexity; direct blob/tree/ref updates require careful atomic code.
- A branch carried by the same remote may still expose private metadata and inflate clone/fetch size. Unpushed local refs are not durable off-machine.
- Multiple writers updating one ref contend even when event files differ; reconciliation requires fetch/merge/retry policy. Shared network filesystems are especially risky.
- Worktree pruning, branch deletion, garbage collection, shallow clones, bare repos, remote changes, and non-Git workspaces create edge cases.
- A nested/sidecar repository is easier to reason about but needs independent remote configuration, credentials, backup, and discovery.

**Fit:** viable advanced backend for teams requiring replicated audit history. Prefer a sidecar repository over a magic branch/worktree implementation; it has a clearer ownership and failure boundary.

### 5. Compact checkpoints or selected audit records in the project

Keep raw events externally or ignored, and periodically commit a compact artifact: a completion summary, signed/hashed checkpoint, sequence watermark, or selected high-value events (for example completion, decision, and acceptance records).

**Strengths**

- Balances a quiet repository with portable milestones and bounded history growth.
- Completed Markdown logs already capture much of the useful durable record; checkpoint hashes can make later tampering/loss detectable when raw events are backed up elsewhere.
- Privacy and retention can differ between raw operational events and durable project milestones.

**Weaknesses**

- Selection and compaction policy can confuse users and create false expectations of a complete audit trail.
- Hashes prove consistency only if the raw ledger is retained and canonicalization, ordering, and actor-set rules are specified.
- Concurrent checkpoint production requires deterministic aggregation and duplicate handling.
- Migration and verification add complexity; checkpointing cannot substitute for raw-event backup.

**Fit:** useful complement to external storage, especially by enriching completed logs. Not a standalone raw-event backend.

### 6. Configurable storage modes/backends

Expose a small mode choice rather than one universal topology, initially `user-state`, `workspace-ignored`, and `workspace-tracked`; later add `sidecar-git` or a sync/export target.

**Strengths**

- Supports private solo work, portable open-source projects, regulated audit needs, and ephemeral CI without forcing one trade-off.
- Enables gradual migration and preserves a strict compatibility mode.

**Weaknesses**

- Configuration multiplies test cases and makes collaboration ambiguous if participants use different modes.
- Readers must aggregate/migrate several sources without duplicating event identities.
- Per-user config cannot alone define a team audit promise; workspace policy and local destination must be distinct.

**Fit:** good product direction if the initial surface remains narrow. Separate **workspace policy** (required portability/retention) from **local backend** (resolved path and credentials).

## Comparison summary

| Option | Main Git quiet | Portable/shared | Audit durability | Concurrency | Privacy | Complexity |
| --- | --- | --- | --- | --- | --- | --- |
| Tracked workspace files | No | Excellent | Excellent when pushed | Good per actor; Git merges | Low | Low |
| Ignored workspace files | Yes | Poor | Poor without backup | Good per actor | Medium | Low |
| External user state | Yes | Poor by default; exportable | Medium; backup-dependent | Good per actor | High | Medium |
| Separate Git storage | Yes | Good when configured | High | Medium; ref/merge contention | Configurable | High |
| Checkpoints + raw sidecar | Mostly | Good for selected history | Medium–high with raw backup | Needs deterministic aggregation | High | Medium–high |

## Recommended default

Adopt **external user-state storage as the default for new event writes**, while keeping `.tandem/board/`, `.tandem/logs/`, decisions, rules, and workspace configuration portable in the project. This matches the existing invariant that events enrich history but are not needed to reconstruct current or completed work.

Recommended shape:

1. Add a stable opaque `workspaceId` to workspace configuration (subject to protocol review), generated at init or lazily during an explicit migration.
2. Resolve the default event root through XDG state conventions and partition it by workspace plus local checkout identity. Store a small metadata record with known paths/remotes to support relocation and diagnosis; never use a path or remote URL as the sole identity.
3. Preserve per-actor JSONL and `<actor>:<seq>` identities. Backend changes should alter location, not event semantics.
4. Provide `tandem events status`, `export`, `import`, and eventually `gc`/`repair` capabilities before relying on external storage. Export should be explicit and redactable.
5. Offer `workspace-tracked` as the portable/audited alternative and `workspace-ignored` as a simple compatibility mode. Defer Git-native sidecar sync until demand justifies its complexity.
6. Record high-value durable outcomes in completed-work Markdown and decisions, not only in raw events. Optionally add checkpoint manifests later after canonical hashing and retention rules are designed.

The default should be described honestly as **local audit history**, not a durable collaborative audit trail, unless backup/sync is configured.

## Migration implications

- Readers should continue aggregating legacy `.tandem/events.jsonl` and current `.tandem/events/*.jsonl`; do not move or delete them automatically on first run.
- New writes can switch independently from reads. A migration command should copy records to the selected backend, validate JSONL and duplicate `<actor>:<seq>` identities, write atomically, then optionally offer a separate cleanup/untracking step.
- Ignore rules do not untrack existing files. Cleanup guidance must distinguish `git rm --cached` from deleting local history and must not rewrite Git history automatically.
- If both old and new sources contain the same event, readers should deduplicate canonical events by `<actor>:<seq>` and surface conflicting payloads as corruption, not silently pick one. Legacy records without canonical identity need source-based identities such as `legacy:<source>:<line>`.
- Downgrades remain safe if board/log state is self-contained. Older tools may continue writing workspace events, so mixed-version readers need multi-source aggregation until a future protocol boundary.
- Clone behavior needs an explicit choice: preserve project `workspaceId` for conceptual continuity, but create a local checkout instance identity to prevent unrelated clones from sharing append files accidentally.

## Open questions for review

1. Is `workspaceId` portable project identity, or should identity be a separate sidecar pointer to avoid modifying protocol config?
2. Should clones on one machine share a logical event history, remain isolated by checkout, or prompt on first use?
3. What audit guarantee, if any, may a workspace require from collaborators: none, local retention, export-on-demand, or replicated tracked storage?
4. Which events are sufficiently valuable to duplicate into completed logs or future checkpoints, and what privacy/redaction policy applies?
5. What are default retention, size rotation, compaction, encryption, and backup expectations for user-state events?
6. Should raw event commands be under `tandem events` or an explicitly diagnostic namespace such as `tandem audit`?
7. How should worktrees map to identity: one checkout ledger per worktree, or one repository ledger with distinct actor writers?
8. Is a pluggable backend interface needed immediately, or can v1 support only three filesystem modes with a stable resolver abstraction?

## Viable rollout

A conservative sequence is: (1) add storage resolution and status diagnostics without changing writes; (2) support external writes while reading all legacy sources; (3) add export/import and conflict diagnostics; (4) make external user state the new-workspace default after migration and backup messaging are adequate; and (5) evaluate sidecar Git sync and checkpoints from real collaboration requirements.
