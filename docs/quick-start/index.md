---
title: Quickstart
description: Install Tandem and run one task from idea to completed log.
---
This guide takes one task through the Tandem loop: install, initialize, add work, start and claim it, deliver it for validation, accept it, complete it into logs, and open the TUI.

The workflow below is CLI/TUI-only. Agent and editor integrations are optional; see [Extensions](/extensions/) when you are ready to connect them.

## 1. Install Tandem

Choose the lane that matches your environment.

### Recommended installer

Install the latest released Tandem binary with the branded installer command:

```sh
curl -fsSL https://trytandem.dev/install.sh | sh
tandem --version
```

The branded URL redirects to the cargo-dist generated installer on the latest GitHub Release. The GitHub Release assets are the source of truth for OS/architecture detection, release asset selection, checksums, and installation behavior.

The initial binary target set is Linux x86_64, Linux ARM64, macOS Intel, and macOS Apple Silicon. Windows binaries are not published initially.

The installer is user-local and does not require `sudo`. It installs into cargo-dist's user bin directory for this platform, typically a directory such as `~/.local/bin` or `~/.cargo/bin`. If `tandem --version` is not found after installation, add the reported install directory to your shell `PATH`. For example:

```sh
mkdir -p ~/.local/bin
printf '\nexport PATH="$HOME/.local/bin:$PATH"\n' >> ~/.profile
. ~/.profile
```

If your install reported `~/.cargo/bin` instead, add that directory instead:

```sh
printf '\nexport PATH="$HOME/.cargo/bin:$PATH"\n' >> ~/.profile
. ~/.profile
```

### Cargo / Rust: available now

If you have Rust and Cargo installed, install from the current tagged source:

```sh
cargo install --git https://github.com/Algorant/tandem.git --tag tandem-v0.4.0 --path tandem --locked
tandem --version
```

From a local checkout of this repository, use:

```sh
cargo install --path tandem --locked
tandem --version
```

### AUR binary

Arch Linux users can install the x86_64 binary package from AUR once the release automation has published it:

```sh
yay -S tandem-bin
tandem --version
```

The initial AUR package is `tandem-bin` for x86_64 only. It installs the published GitHub Release binary and uses the release `sha256.sum`; it does not build Tandem from source.

## 2. Initialize a workspace

Run this at the root of the project repository you want to coordinate:

```sh
tandem init --title "My Project"
```

Tandem creates `.tandem/tandem.md`, active board files in `.tandem/board/`, completed logs in `.tandem/logs/`, and lifecycle event logs. The default active task states are `todo`, `in-progress`, and `validation`.

## 3. Add a task

```sh
tandem add --title "Write project brief" --description "Draft and validate the first docs slice."
tandem list
tandem show task-1
```

The task is a Markdown file with YAML frontmatter. You can read it in any editor, then use Tandem commands for safe state and accord updates.

## 4. Start and claim the work

Move the task into active work and claim the accord:

```sh
tandem move task-1 --state in-progress
tandem accord claim task-1 --assignee alice
```

Claiming from `todo` also moves a task to `in-progress`; the explicit `move` above is useful when you want the board state change to be visible before the accord claim.

## 5. Deliver through the accord

After doing the work, record what changed and how it was checked:

```sh
tandem accord deliver task-1 \
  --summary "Drafted the brief and checked the rendered docs" \
  --deliverable "Updated docs/index.md" \
  --validation "Ran cd site && bun run check:docs" \
  --file-changed docs/index.md
```

Delivery moves the task to `validation`. The worker records evidence; a reviewer or orchestrator decides whether the delivery is accepted.

## 6. Validate and accept

Inspect the delivered task:

```sh
tandem list --state validation
tandem show task-1
```

If the work is acceptable, accept the accord:

```sh
tandem accord accept task-1 --reviewer bob --summary "Looks good"
```

If it needs another pass, request rework instead:

```sh
tandem accord rework task-1 --note "Please add the install path and validation command."
```

## 7. Complete and search the log

After acceptance, archive the task as completed history:

```sh
tandem complete task-1 \
  --summary "Published the project brief" \
  --validation "Reviewed by Bob" \
  --file-changed docs/index.md
```

The v0 CLI may warn when separate `review.status` metadata is missing. Treat that warning as a policy reminder: complete only when the responsible reviewer or orchestrator has intentionally accepted the work.

Search or inspect the completed log:

```sh
tandem log list
tandem log show task-1
tandem search "project brief"
```

Logs are first-class history, not trash. They preserve what shipped, why it was accepted, which validation evidence was available, and which files changed.

## 8. Open the TUI

Use the terminal interface for day-to-day board work:

```sh
tandem tui
```

The TUI centers on the Board, including the `todo`, `in-progress`, and `validation` task states, plus Logs, Rules, and Decisions views.

## Next steps

- Read [Concepts](/concepts/) for the mental model behind states, accords, validation, decisions, and logs.
- Read [CLI](/cli/) for the command families used by the quickstart.
- Read [TUI](/tui/) for keyboard, mouse, and theme behavior.
