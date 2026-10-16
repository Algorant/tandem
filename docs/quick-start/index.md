---
title: Quickstart
description: Install Tandem and take a Task from start to finish.
---

## Install and verify

On Linux or macOS, install the latest release and check that the command is available:

```sh
curl -fsSL https://trytandem.dev/install.sh | sh
tandem --version
```

The installer is user-local and does not require `sudo`. With Rust, install the current source with `cargo install --git https://github.com/Algorant/tandem.git --locked tandem`.

## Complete a Task

In a Git repository, initialize the board and create a Task with an acceptance criterion:

```console
$ tandem init
$ tandem add task "Add a greeting" --acceptance "README contains the greeting"
Created task
ID: task-1
Title: Add a greeting
Sync: saved locally (no Git remote to sync with)
$ tandem accord claim task-1 --assignee you
Accord task-1: claimed (record written; saved locally (no Git remote to sync with))
```

Add the greeting to `README.md`, then record the result and check:

```sh
printf '\nHello from Tandem.\n' >> README.md
```

```console
$ tandem accord deliver task-1 --summary "Added a greeting" --evidence "README greeting verified"
Accord task-1: delivered (record written; saved locally (no Git remote to sync with))
$ tandem complete task-1
Completed task-1 (record written; saved locally (no Git remote to sync with))
$ tandem list --scope archived
task-1	Add a greeting
```

This is output from a fresh Git repository without a remote; a configured remote can show a different sync message. `complete` archives the delivered Task in Logs. Use `tandem show task-1` or `tandem search greeting --scope archived` to read it later.

See [Concepts](/concepts/) for board storage and sync, or [Agents](/agents/) to give an agent project-specific instructions.
