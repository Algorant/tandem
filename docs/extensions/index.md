---
title: Extensions
description: Tandem integration adapters.
---
Tandem integrations are thin adapters over an installed Tandem implementation. Read the framework-neutral [Agents and adapters](/guides/agents-and-adapters/) guide before implementing one.

## Adapter principle

```text
human or agent → integration adapter → tandem CLI → .tandem workspace
```

Tandem-owned protocol documentation defines documents, IDs, relationships, lifecycle operations, Rules, Decisions, and Logs. Active workspace Rules define repository policy. An adapter owns framework-native tool schemas, prompts, approvals, output formatting, and diagnostics.

A conforming adapter:

- discovers the workspace from the intended working directory;
- inspects workspace health and all rule categories before mutation;
- uses safe, non-interpolated command arguments;
- consumes structured read output when available;
- preserves Tandem-returned relationships, warnings, and diagnostics;
- asks before workspace initialization or protocol upgrade.

It does not parse or mutate Tandem Markdown as a second implementation, allocate IDs, reclassify relationships, manage actor identity, or infer lifecycle authority from command availability.

## Integration sequence

1. Diagnose the installed Tandem implementation and nearest workspace.
2. Read configured states and all workspace Rules.
3. Inspect the target and resolve parents, blockers, references, Decisions, and Logs needed for context.
4. Ask the responsible actor to approve any initialization, upgrade, or terminal lifecycle action that is not already authorized.
5. Invoke the Tandem operation and preserve its output.
6. Present validation as evidence, not as automatic acceptance.

## Current adapter example: Pi

The repository includes `extensions/pi-tandem/`. It maps Pi-specific tools and commands to the installed `tandem` CLI. Its tool names, prompt hooks, renderer, and local tests are adapter implementation details, not universal Tandem semantics.

Global Pi configuration promotion is a separate explicit task. Repository-local extension development does not edit personal configuration directly.
