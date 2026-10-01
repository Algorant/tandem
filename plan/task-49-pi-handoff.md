# task-49 handoff: independent board sync for Pi and sysup

Status: ready for the owning repositories (`~/.pi`, `~/.dotfiles`). Tandem does not change adapter code (rule `never-2`); this note is the contract to implement there. Supersedes the checkpoint contracts in `task-14-pi-handoff.md`, `task-39-pi-handoff.md`, and `task-40-pi-handoff.md`.

## What changed in Tandem (protocol 0.4.0)

- **The board no longer lives in source commits.** In a Git repository `.tandem/` is ignored by the source branch and syncs through a Tandem-managed `tandem` branch on the same remote. Tandem does all fetching, merging, and pushing of that branch.
- **`tandem checkpoint` and `--consolidate` are gone.** There is no metadata step at commit or push time, and `.tandem/` never appears in `git status` of a migrated repository.
- **Mutation JSON reports `data.sync` instead of `data.checkpoint`:**

  ```json
  {"ok":true,"data":{"id":"task-52","sync":{"status":"synced","message":null,"renamed":{"task-new-3f9a2c1d":"task-52"},"conflicts":[],"held":[]}},"warnings":[]}
  ```

  `status` is `synced`, `pending` (saved locally; `message` says why, e.g. offline), `local-only` (no remote), or `not-git`. `pending` is not a failure. `data.id` is always the record's current ID; `renamed` maps temporary IDs that were numbered during this call. `add`, `update`, `accord *`, `review`, `complete`, `cancel`, `rules add|edit|delete`, and `init` all include it.
- **Temporary IDs.** A record created while its board cannot publish gets `<prefix>-new-<8 hex>` (for example `task-new-3f9a2c1d`) and is numbered on the next successful sync. Commands keep accepting the temporary ID afterwards.
- **Exact JSON keys (0.15.0):** `sync status` returns `data` with `git`, `remote`, `published`, `pending`, `lastFetch` (RFC 3339 or null), `lastError`, `conflicts[{id,reason}]`, `held[{path,reason}]`. `sync` returns `data.sync`; `sync resolve` returns `data.id` and `data.sync`. `migrate` returns `dryRun`, `remote`, `records`, `files`, `migratedFrom`, `sourceCommit`; `migrate --adopt` returns `dryRun`, `remote`, `newRecords`, `changedRecords`, `renumbered[{old,new}]`, `unpushedCommits`, `needsPull`, `sync`. `status: "pending"` covers both offline and held or conflicted changes: branch on the `conflicts` and `held` arrays, and do not match message text or the generic `io` error code. An unparsable record is skipped by every command with a `held from sync` warning.
- **New commands:** `tandem sync` (sync now), `tandem sync status` (local, no network: remote, pending, last fetch, last error, conflicts, held edits), `tandem sync resolve <id> --keep local|remote|edited`, and `tandem migrate [--dry-run] [--adopt]`.
- **Conflicts.** A record changed differently on two machines keeps the shared version on the board; mutations to it fail with `unresolved sync conflict` until `tandem sync resolve`. Everything else keeps syncing.
- **Worktrees.** One board per clone, in the main worktree. A linked worktree (every Worker checkout) has no `.tandem/` and Tandem commands run inside it use the main worktree's board. Each checkout keeps its own actor identity in its Git directory (`<git-dir>/tandem-actor-id`); adapters stay identity-unaware.
- **Reads** refresh from the remote when the last fetch is older than 60 seconds; reads may add a warning such as `the board may be out of date: offline: …`.
- **Old checkouts.** If `.tandem/tandem.md` currently comes from a pre-migration commit, reads work with a `predates the tandem branch` warning and mutations are refused.
- **0.3.0 boards** are refused until migrated, with a message naming `tandem migrate` or `tandem migrate --adopt`.

## Required `~/.pi` changes

1. **pi-tandem tools:** remove `tandem_checkpoint`; add a thin `tandem_sync` tool over `sync`, `sync status`, and `sync resolve` (argument arrays, `--json`, no parsing of board files). Keep the ten-tool surface or document the change.
2. **Renderers (pi-ui `custom-tools/tandem.ts`):** render `data.sync` for every mutation (status, renamed IDs, conflicts, held edits) instead of `checkpoint`; render `tandem_sync` results.
3. **Guidance (pi-tandem `guidance.ts`, `index.ts` prompt text, `skills/tandem`, `skills/worktrunk`, `prompts/push.md`, agency README):** remove every `.tandem` staging, checkpoint, consolidate, and push-boundary rule. Replace with: Tandem syncs the board itself; never stage, commit, push, edit, delete, or rename `.tandem/` or the `tandem` branch; treat `pending` as saved; surface conflicts and held edits to Algorant and resolve only with `tandem sync resolve` when authorized.
4. **Workers (`worker_start`, `worker_finish`, `worker_discard`, `worker_integrate`, assignment reads):** stop expecting `.tandem/` inside the Worker checkout and stop treating pending `.tandem/` as required repository state through start and merge. Assignment and milestone commands run from a Worker checkout operate on the main worktree's board and publish immediately. Remove any checkpoint call from finish/integrate.
5. **Papercuts/ID handling:** accept temporary IDs in tool results and tool arguments.

Validate with the pi-tandem test suite against a disposable repository with a bare remote and two clones: mutation results show `data.sync`, a Worker worktree reads and changes the main board, and no command references `checkpoint`.

## Required `~/.dotfiles` changes

- `sysup pi` keeps `git -C ~/.pi pull --ff-only`; board changes no longer block it. Optionally run `tandem sync` in known Tandem repositories after pulling; not required.
- Remove any `.tandem` handling from sysup or related scripts if present.

## Rollout order

The Pi-first versus Tandem-first cutover is decided in `~/.pi` task-305; the upgrade guide describes one possible order. Follow `docs/guides/upgrading-to-independent-sync.md`: install the release on every machine, land and install the Pi changes above, then migrate this repository, `~/.dotfiles`, and `~/.pi` in that order in one sitting.
