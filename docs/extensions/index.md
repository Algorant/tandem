---
title: Extensions
description: Integrating with Tandem through the CLI.
---
The `tandem` CLI is Tandem's integration surface. Any agent or editor integration should call the installed `tandem` binary with an argument array (no shell interpolation) and use `--json` for reads. Tandem remains responsible for IDs, validation, relationships, storage, status, search, and events; an integration must not parse or mutate Tandem Markdown itself.

For example, an integration records small, non-blocking friction with `tandem add task --kind papercut` and lists it with `tandem list --kind papercut --json`.

Tandem does not currently ship an official adapter for any agent framework. The former `pi-tandem` Pi adapter was retired from this repository.

See [Agents and adapters](/guides/agents-and-adapters/) for the framework-neutral integration contract.
