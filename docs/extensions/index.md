---
title: Extensions
description: Tandem integration adapters.
---

# Extensions

Tandem integrations should be thin adapters over the installed `tandem` CLI.

## Adapter principle

```text
LLM / editor agent → integration adapter → tandem CLI → .tandem workspace
```

Adapters own tool schemas, editor or agent ergonomics, output formatting, and diagnostics. They should not duplicate Tandem protocol parsing or mutation behavior.

## Current adapter

`extensions/pi-tandem/` provides a Pi adapter that exposes Tandem task, accord, rule, decision, and log operations through Pi tools and commands.

Global Pi config promotion is a separate explicit task. Repository-local extension development should not edit `~/.pi/agent` directly.
