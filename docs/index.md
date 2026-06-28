---
title: Welcome to Tandem
description: Local-first coordination for humans and agents.
---

# Tandem documentation

Tandem is a local-first protocol and toolchain for human/agent project coordination. It uses Markdown files, explicit work agreements called accords, validation-focused review, and completed-work logs to keep project state understandable in a normal repository.

This docs tree is the canonical Markdown source for Tandem documentation. The `site/` project renders these files for the web.

## Start here

- [Concepts](./concepts/index.md) explains Tandem's core vocabulary.
- [Protocol](./protocol/index.md) describes the `.tandem/` file model.
- [CLI](./cli/index.md) covers the `tandem` command.
- [TUI](./tui/index.md) covers the terminal interface.
- [Extensions](./extensions/index.md) covers integration adapters.
- [Guides](./guides/index.md) collects task-oriented workflows.

## Current status

Tandem is in early v0 implementation. The CLI surface is implemented for the current known scope, and the Ratatui TUI is under active development. Documentation should stay small, source-oriented, and easy to update as implementation details settle.
