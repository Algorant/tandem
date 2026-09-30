# Independent metadata sync

Status: **proposed design for Algorant review** (task-48). Q1–Q3 resolved by Algorant on 2026-09-30. Implementation is task-49 and must not begin until this document is approved.
Date: 2026-09-30

## 1. Summary

Tandem metadata stops travelling inside source-code commits. Each repository gets a Tandem-managed `tandem` branch on its existing Git remote, similar in spirit to a `gh-pages` branch. Tandem keeps one local board per clone, synchronizes it automatically while Tandem is used, and merges changes by understanding records rather than lines of text.

Three mechanisms work together; none is sufficient alone:

1. **Separate channel.** Board changes never dirty, stage, or advance a source branch, so they cannot block a source `git pull --ff-only`.
2. **Hidden permanent identity, human sequential numbers.** Every record has a UUID internally. The human number (`task-50`) is assigned when the record first reaches the shared branch, which serializes assignment. Existing IDs never change.
3. **Semantic three-way merge.** Independent creations and edits combine automatically. Only genuinely contradictory edits stop, and they never block unrelated work.

Everyday behavior: open Tandem and incoming work appears; make a change and it publishes within seconds; work offline and changes are durable and visibly pending; reconnect and they publish.

## 2. Verified failures today (Tandem 0.14.1)

Reproduced with independent disposable clones; these scripts become native regression tests in task-49.

| Failure | Verified cause |
| --- | --- |
| Two machines create different `task-1` | Local `max + 1` allocation (`app/tasks.rs`, `app/support.rs`, `protocol/ids.rs`); no reservation. Rules (`always-N`) and Subtasks collide the same way. |
| `git pull --ff-only` refuses: untracked `.tandem/tasks/task-N.md` would be overwritten | Same path allocated locally and upstream. |
| Pull succeeds, then every command fails with a duplicate ID | Remote archived `logs/task-N.md`, local active `tasks/task-N.md`: different paths, same ID. |
| Checkpoint first, then pull still fails | Committing converts untracked files into divergent history. |
| Ordinary merge succeeds but `checkpoint --consolidate` refuses | Consolidation rejects merge commits in the unpushed range. |
| Same record edited on two machines | Git merges Markdown lines, not fields or lifecycle. Independent fields sometimes merge by luck; nearby fields conflict. |

Decision-7's per-actor UUID separates event ledgers only. It never namespaced record IDs, and events cannot reconstruct records (creation events carry only ID and title).

## 3. Goals and guarantee

The design removes **routine Tandem-induced Git failures and manual metadata repair**. It does not remove source-code conflicts, and it cannot automatically decide between two genuinely contradictory edits to the same field.

Non-goals: a hosted service, a database authority, real-time collaboration, always-on background daemons.

## 4. Architecture

### 4.1 Storage

| Item | Location | Tracked? |
| --- | --- | --- |
| Shared board history | branch `tandem` on the repository's remote | yes, on the `tandem` branch only |
| Local board (the Markdown files) | `.tandem/` in the main worktree, ignored by the source branch | no |
| Safety copy of every saved change | ref `refs/tandem/pending` (a commit object, never pushed) | local ref |
| Last-synced remote commit | ref `refs/tandem/base` | local ref |
| Change-detection index | `<git-common-dir>/tandem/index` (private `GIT_INDEX_FILE`) | local |
| Held conflicts and status | `<git-common-dir>/tandem/conflicts/`, `status.json` | local |
| Actor identity | `<per-worktree git-dir>/tandem-actor-id` | never |

The board uses the existing 0.3.0 layout (`tandem.md`, `tasks/`, `decisions/`, `rules/`, `logs/`, `events/`) with readable Markdown.

**Visible folder with a safety copy.** The board stays in the familiar visible `.tandem/` folder. A disposable spike confirmed two ways Git can damage an ignored folder: checking out a pre-migration commit (which tracked `.tandem/`) overwrites it with historical files and deletes them again on return, and `git clean -fdx` deletes it. Synced content is always recoverable from the `tandem` branch; the safety copy covers unsynced changes:

- Every Tandem mutation, and every sync snapshot, writes the local board tree into Git's object store and points `refs/tandem/pending` at it. This is local, needs no network, takes milliseconds, and is invisible to `git status`, `git log`, branches, and the source index. Superseded snapshots are ordinary unreachable objects collected by `git gc`.
- Records are deleted only through Tandem commands (Rules). A board file that disappears on its own is damage and is restored from `refs/tandem/pending`, never synced as a deletion.
- If `.tandem/tandem.md` shows a pre-0.4.0 protocol, the folder holds historical files from an old checkout. Tandem warns that it is showing historical data, does not sync it, and restores the current board when the checkout returns to a migrated commit.
- Remaining exposure: an editor change that no Tandem command has seen yet, followed immediately by an old checkout or `git clean`.

One clone has exactly one board, in its main worktree. Commands run from linked worktrees (including Pi Workers) resolve the main worktree through the Git common directory and use the same board, so Workers no longer carry `.tandem` state on their source branches.

One clone has exactly one local board. All linked worktrees (including Pi Workers) read and write that same board, so Workers no longer carry `.tandem` state on their source branches.

A directory that is not a Git repository keeps a plain local `.tandem/` board with no sync. A repository without a remote keeps a local-only board and reports `sync: no remote`.

### 4.2 Actor identity (decision-7)

Unchanged guarantee: one non-configurable random UUID per independent checkout or linked worktree, created and owned by Tandem, never committed. Only the file location moves, from `.tandem/actor-id` to the per-worktree Git directory. Each actor appends only to `events/<actor>.jsonl`, so every ledger has exactly one writer.

## 5. Record identity and numbering

Every Task, Subtask, Epic, Decision, and Rule gets `uid: <UUIDv4>` in frontmatter at creation. The uid is permanent, is used to match the same record across machines, and must not be edited.

The human ID is assigned by publication, like a GitHub issue number being assigned by the server:

- **Created while unsynced:** the ID is a provisional handle derived from the uid, e.g. `task-new-3f9a2c1d`, `decision-new-…`, `always-new-…`. Everything works with it: show, reference, claim, complete.
- **First successful publish:** Tandem assigns the next number against the freshly fetched shared branch: `task-50`, Subtask `task-50-2` (parents before children), `decision-9`, `always-5`. Files are renamed and every structured reference and exact prose occurrence of the handle is rewritten. After this the number never changes.
- **Race:** Git accepts only one push per starting point. The loser fetches, numbers its still-unpublished records against the new tip, and retries.
- **Invariant:** the shared branch never contains provisional handles, so every reference stored there is a permanent numbered ID.
- Provisional handles keep resolving forever through their uid prefix; an ambiguous prefix is reported, never guessed.
- Online, a mutation publishes before the command returns, so users and agents normally receive the final number immediately. Offline creation is the only time a provisional handle is visible.

Existing records keep their IDs and receive uids during migration.

## 6. Sync transaction

Holding one repository-wide lock (the same lock every mutation takes):

1. **Snapshot local:** restore any damaged files from `refs/tandem/pending` (§4.1), stage the local board into the private index, write tree `L`, and update `refs/tandem/pending`. Files that fail to parse or validate are held back at their last good version and reported (see §8).
2. **Fetch:** `git fetch <remote> refs/heads/tandem`, bounded by a short timeout. Offline → stop; changes remain pending.
3. **Merge:** three-way semantic merge of base `B` (`refs/tandem/base`), local `L`, and remote `R` (§7), then number unpublished records against `R` (§5). Result tree `M`, committed with parent `R`.
4. **Publish:** `git push <remote> M:refs/heads/tandem` without force. A rejection means the remote advanced: return to step 2 (bounded retries).
5. **Apply locally:** only after the push is acknowledged, write `M` into the local board (renames, numbering, remote changes) and set `refs/tandem/base = M`. Files edited during the sync are left for the next cycle.

Crash safety: the local files, backed by `refs/tandem/pending`, are the pending truth and `base` advances only after acknowledgement. If a crash happens after a successful push, the next sync sees identical changes on both sides and matches renumbered records by uid, so nothing is duplicated. No step rewrites published history or touches the source branch.

## 7. Semantic merge rules

Records are matched by uid (moves between `tasks/` and `logs/` are the same record).

| Situation | Result |
| --- | --- |
| Changed on one side only | Take that side. |
| Created independently | Keep both. |
| Same scalar field changed on both sides to the same value | Take it. |
| Different fields changed | Combine. |
| Same field changed differently | Conflict. |
| Set-like lists (`tags`, `references`, `blockers`, `relatedFiles`, `supersedes`) | Three-way set merge. |
| Ordered lists (`acceptance`, `constraints`, `validations`, evidence) | Take the changed side; conflict if both changed differently. |
| Markdown body | Three-way text merge; conflict if it does not merge cleanly. Conflict markers are never written into a record. |
| `updatedAt` | Latest value; never a conflict. |
| Lifecycle (`state`, Accord status, assignee, archive/resolution) changed on both sides differently | Conflict. |
| Archived on one side, edited on the other | Conflict: Tandem asks which to keep (Algorant, Q2). |
| Deleted on one side (Rules), edited on the other | Conflict. |
| Event ledgers | One writer per file: the longer ledger must extend the shorter one exactly; anything else is an integrity error (copied identity) and stops. |
| `tandem.md` settings | Per-field three-way. |

The merged board must pass full validation (parents, blockers, Epic/Task/Subtask roles, active descendants). A local change that would make it invalid is held as a conflict.

Timestamps never decide a winner.

## 8. Conflicts and invalid edits never block unrelated work

When a record conflicts, the published result keeps the remote version of **that record only**; everything else publishes normally. The local version, base, and remote are preserved in `<git-common-dir>/tandem/conflicts/<uid>/`. Tandem reports:

```text
task-50 conflicts: title changed on this machine ("A") and on another machine ("B").
Resolve: tandem sync resolve task-50 --keep local|remote   (or edit, then --keep edited)
```

Mutations to a conflicted record are refused until it is resolved. Resolution is an ordinary change that then syncs.

Direct Markdown edits are fully supported. Sync detects them by comparing the local board with the last synced version (the same way Git detects changes). An edit that does not parse or validate, or that changes `uid` or a published ID, is held back with a precise error (file, line, reason) and is not published. Other changes continue to sync.

## 9. When sync runs (no daemon)

- **Mutations** (CLI, TUI, web): publish immediately after the local write, bounded to a few seconds. Offline is reported as `saved locally; pending sync`, never as failure.
- **Reads:** fetch first when the last successful fetch is older than a short freshness window (default 60 s), bounded; on timeout, read the local board and warn that it may be stale.
- **TUI/web:** sync on start, after each mutation, and periodically while open.
- **Explicit:** `tandem sync` and `tandem sync status` (pending, conflicted, held edits, last sync, remote).

Nothing runs when Tandem is not in use; pending changes publish on the next use.

## 10. Discovery, new clones, and worktrees

From any directory inside a clone, Tandem resolves the Git common directory and uses its board, including from linked worktrees. If the clone has no local board and the remote has a `tandem` branch, the first command hydrates it automatically (network required once). The remote defaults to the current branch's upstream remote, else `origin`, and is recorded at hydration.

`tandem.md` gains a permanent `workspaceId` so hydration and migration can confirm they are joining the same board.

## 11. Migration and rollout

Protocol version becomes **0.4.0**. Older Tandem refuses 0.4.0 boards, so an old binary cannot silently overwrite the new format.

### First machine: `tandem migrate`

1. Preflight: fetch; require the source branch not behind its upstream for `.tandem/`, no pending legacy `.tandem` changes, and no linked worktree with `.tandem` changes relative to the current branch. `--dry-run` reports everything without writing.
2. Convert: add a uid to every record, add `workspaceId`, set protocol 0.4.0, record `migratedFrom: <source commit>`; IDs, references, Logs, Decisions, Rules, and event ledgers are otherwise unchanged.
3. Publish the converted board as the root commit of the remote `tandem` branch and hydrate the local board.
4. Create one source commit that removes `.tandem/` from tracking (`git rm --cached`, so the folder stays in place) and ignores it. The user reviews and pushes it normally. Legacy board history remains in source history.

### Other machines: `tandem migrate --adopt`

Run with the new binary **before** pulling the migration source commit (Tandem detects the situation and says so). Pulling that commit deletes the tracked legacy `.tandem/` files; Tandem then rebuilds the folder from the `tandem` branch and the adopted changes. It computes this machine's legacy changes (uncommitted, untracked, and unpushed commits) relative to the common legacy base, then applies them to the new board as local changes:

- Local legacy records that do not exist upstream become new records and receive fresh numbers, with references rewritten and an old → new report. This fixes the historical collision class.
- Edits merge by the rules in §7; ambiguous cases (both sides claimed, delivered, or logged) become held conflicts naming both sides.

Afterwards a normal `git pull` removes the legacy `.tandem/`. If files are left over, Tandem detects them and offers `--adopt` again. Nobody hand-edits `.tandem/`.

Before migration, finish or discard Workers whose branches changed `.tandem/`.

### Rollout order

1. Release the new Tandem and install it on every machine.
2. Migrate a disposable copy of each real workspace with `--dry-run`, then for real.
3. Order: this repository → `~/.dotfiles` → `~/.pi` (the Pi adapter must be updated in the same window, see §13).

## 12. Superseded behavior and decisions

- `tandem checkpoint` and `checkpoint --consolidate` are removed, along with the source-branch batching contract (tasks 14, 37–40, 43, 44, 46). Lifecycle outputs report sync status instead of `checkpoint`.
- **decision-7:** amended, not superseded. Same identity guarantee; the file location changes.
- **decision-8:** narrowly superseded: storage location, protocol 0.4.0, a one-time `migrate` command, and removal of `checkpoint`. Its CLI, role, Accord, and architecture decisions stand.
- New decision records these choices once approved.

## 13. Handoffs (separate Tasks in their owning repositories)

- **`~/.pi` (pi-tandem, agency, skills, prompts):** replace checkpoint/consolidate guidance and tools with sync status; Workers stop carrying `.tandem` on branches; `worker_finish`/`worker_start` drop metadata checkpoint steps; renderers for any new tool results.
- **`~/.dotfiles` (`sysup`):** optionally run `tandem sync` for known workspaces; `git pull --ff-only` no longer needs Tandem handling.

## 14. Alternatives rejected

| Alternative | Why rejected |
| --- | --- |
| Fetch or reserve before allocating | Two machines can still fetch the same maximum; fails offline. |
| Commit every mutation immediately | Replaces untracked-file failures with divergent history. |
| More consolidation rules | Shapes commit history; does not synchronize. |
| Union merge driver on `.tandem/` | Silently combines contradictory content. |
| Automatic refile/renumber after collisions | Keeps creating collisions and then repairs them; retained only as migration adoption. |
| Per-machine number ranges | Numbers jump by machine and every machine needs a range. |
| UUID-only or short-hash IDs | Loses the readable sequence Algorant wants. |
| Board hidden inside `.git/` | Safe, but not visible in the project; Algorant prefers the visible folder with a safety copy. |
| Visible folder without a safety copy | Old checkouts and `git clean -fdx` can destroy unsynced changes (verified). |
| Central server or database authority | Adds a service to run; unnecessary for one person with Git. |
| Event log as sync source | Events do not contain enough data to rebuild records. |

## 15. Resolved decisions (Algorant, 2026-09-30)

- **Independent sync channel** on the existing remote: approved.
- **Numbering:** hidden UUID identity; sequential human number assigned at first sync; existing IDs unchanged.
- **Direct Markdown edits:** supported, detected, and validated by sync.
- **Scheduling:** sync while Tandem is used; no daemon.
- **Q1 board location:** visible `.tandem/` with a low-overhead local safety copy that does not interfere with source Git.
- **Q2 archive vs concurrent edit:** ask which to keep.
- **Q3 branch name:** `tandem`.

## 16. Verification plan (task-49)

Native tests with independent clones and a local bare remote covering: safety-copy restore after an old checkout and after `git clean -fdx`; every §2 failure; Rule and same-parent Subtask creation; compatible and contradictory same-record edits; archive vs edit; offline create → reconnect; concurrent publish race; crash after push before local apply; interrupted fetch/push; fresh-clone hydration; linked-worktree use; direct valid and invalid edits; provisional-handle resolution and ambiguity; and migration plus adoption from representative workspace copies with legacy collisions. Each scenario asserts exact record content, references, event integrity, and unchanged source `HEAD`, index, and working-tree bytes.

## 17. Spike findings (task-49-1, 2026-09-30)

Disposable repositories only; real workspaces were inspected read-only and copied. The spike script and output are recorded in the task-49-1 evidence.

- **Real workspaces:** this repository 92 board files (584 KB), `~/.dotfiles` 70 (424 KB), `~/.pi` 403 (2.7 MB). All protocol 0.3.0, each a single worktree, up to date with upstream, no duplicate IDs. Only this repository has pending `.tandem` changes (the current work). No migration special cases found beyond the general adopt path.
- **Safety-copy overhead** on a copy of the `~/.pi` board: first snapshot 50 ms, incremental snapshot after an edit 20–25 ms. Source `git status`, index, branches, and log were unchanged. Superseded snapshots are ordinary unreachable objects.
- **Damage confirmed and repaired:** checking out a commit that tracked `.tandem/` silently replaced a board file with its historical version, and returning to the current branch deleted it; `git clean -fdx` deleted the whole board. Restoring from `refs/tandem/pending` took about 25 ms and reproduced the exact saved tree, including an unsynced file.
- **Transport:** plumbing snapshot, publish of a root `tandem` branch, fresh-clone hydration, and a stale push from an old base (rejected without force, then succeeded after fetch) all behaved as designed. A plain `git clone` also receives `origin/tandem`.
- **Linked worktrees:** the first entry of `git worktree list --porcelain` is the main worktree; a linked worktree has no `.tandem` of its own and resolves to the main board.
- **Implementation notes:** `GIT_INDEX_FILE` must be an absolute path (a relative one resolves against `--work-tree`). A GitHub round trip measured 0.6–0.9 s, so publication pushes optimistically on top of `refs/tandem/base` and fetches only when the push is rejected; the common single-machine case costs one round trip.

No finding requires a design change.
