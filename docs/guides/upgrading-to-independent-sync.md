---
title: Upgrading to independent sync
description: Move existing repositories to the tandem branch and set up machines, for people and agents.
---
Tandem 0.15 stores each repository's board on its own `tandem` branch instead of in source commits. This guide moves existing repositories over and sets up each machine. It is written so an agent can follow it step by step. Stop and ask the owner when a step's check does not pass.

Once the new Tandem is installed, every repository still on protocol 0.3.0 refuses to run until it is migrated. Do the whole rollout in one sitting.

## Before you start

1. **Install the release on every machine**, for example with `sysup mise`. Check with `tandem --version` on each machine.
2. **Update the agent integration first.** Older Pi guidance calls `tandem checkpoint`, which no longer exists. Install the Pi update that talks about `sync` instead before migrating `~/.pi`.
3. **Finish or discard Workers** whose branches changed `.tandem/`. Their board changes would otherwise arrive as source-commit changes after the board has moved.
4. **Push board changes from every machine that is easy to reach** with the old Tandem before installing the new one (commit, `tandem checkpoint --consolidate`, push). Machines you forget can still bring their work over with `--adopt` later, but pushing first is simpler.

## Order

Migrate one repository at a time: this Tandem repository, then `~/.dotfiles`, then `~/.pi` (together with its Pi update).

## First machine: migrate

In the repository, on the branch everyone pulls from:

```sh
git pull
tandem migrate --dry-run
tandem migrate
git push
```

Check:

- `tandem migrate` reported a source commit, and `git log -1` shows `chore(tandem): move the Tandem board to the tandem branch`.
- `git status` is clean and `git ls-files .tandem` prints nothing.
- `tandem list` shows the board, and `tandem sync status` reports the remote and no local changes.
- `git ls-remote origin tandem` shows the new branch.

## Every other machine

Decide which case applies:

```sh
git status --short .tandem        # uncommitted or untracked board changes?
git log --oneline @{u}.. -- .tandem   # unpushed commits that change the board?
```

**No board changes on this machine:** just pull. The first Tandem command downloads the board.

```sh
git pull
tandem list
```

**This machine has board changes that were never pushed:** adopt them *before* pulling.

```sh
tandem migrate --adopt --dry-run
tandem migrate --adopt
git pull
tandem list
```

`--adopt` merges this machine's changes into the shared board. Records that only existed here get new numbers; the command prints each old and new ID. If it listed unpushed commits that changed `.tandem/`, `git pull` stops on those files: their content is already on the board, so remove them from the source with `git rm -r --cached .tandem` and continue the pull or rebase.

Check: `git status` is clean, `tandem list` shows the board including this machine's work, and `tandem sync status` shows no conflicts.

## New machine

Clone the repository and run any Tandem command. The board downloads automatically.

```sh
git clone <repository>
cd <repository>
tandem list
```

## Day to day

Nothing to remember. Tandem syncs after every change, refreshes before reads that are more than a minute old, and syncs in the background while the TUI or web view is open. Source `git pull` and `git push` are unaffected by the board.

- **Offline:** changes are saved and reported as `pending`. New records show a temporary ID such as `task-new-3f9a2c1d` until they sync, then get their number. The temporary ID keeps working in commands.
- **Conflict:** when two machines changed the same thing differently, only that record waits. `tandem sync status` lists it; run `tandem sync resolve <id> --keep local|remote|edited`.
- **Held edit:** a hand edit that cannot be published (for example broken YAML) stays on this machine with the reason in `tandem sync status`. Fix the file; it syncs on the next change or `tandem sync`.
- **Old checkout:** checking out a commit from before the migration puts old board files in place. Tandem shows them read-only with a warning; switch back and the current board returns.

## Never

- Hand-edit, delete, or rename files in `.tandem/` to fix sync. Use `tandem sync status` and `tandem sync resolve`.
- Pull the migration commit on a machine with unpushed board changes before running `tandem migrate --adopt`.
- Force-push, delete, or rewrite the `tandem` branch.
- Copy a clone's `.git` directory to another machine; clone instead. If a copy already exists, delete `.git/tandem-actor-id` in the copy.
