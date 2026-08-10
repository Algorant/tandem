---
title: Extensions
description: Tandem integration adapters.
---
Extensions connect Tandem with the tools you use to plan and coordinate work. This page lists official Tandem extensions as they become available.

## Official Pi extension

### pi-tandem

`pi-tandem` is a thin adapter over an installed `tandem` CLI. It exposes Task, Accord, Log, Rule, Decision, Papercut, search, initialization, and status tools without parsing or mutating Tandem Markdown itself.

Use `tandem_papercut` with `add`, `list`, `show`, or `resolve` for small, non-blocking friction. Read actions use CLI JSON. Tandem remains responsible for Papercut IDs, validation, references, storage, status, search, and events.

See the [`pi-tandem` source](https://github.com/Algorant/tandem/tree/main/extensions/pi-tandem).
