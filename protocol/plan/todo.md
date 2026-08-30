# Tandem Protocol Todo

**Protocol:** 0.3.0  
**Status:** normative cutover specification accepted

## Complete

- [x] Define typed `.tandem/tasks`, `decisions`, `rules`, `logs`, and per-actor `events` storage.
- [x] Define Task/Decision documents and fixed Epic → Task → Subtask hierarchy.
- [x] Define mandatory active Task Accord, exceptional validation, and atomic archive flows.
- [x] Define tagged-Task Papercuts and per-file Rules.
- [x] Define structured event envelope and actor-local identity.
- [x] Define global JSON, generated help, exit codes, scope, clear, and list replacement semantics.

## Implementation follow-up

- [ ] Implement 0.3.0 types and validation in `tandem/src/protocol/`.
- [ ] Implement new storage discovery and per-file Rules in `tandem/src/project/`.
- [ ] Implement Accord, review escalation, deterministic updates, and archive operations in `tandem/src/app/`.
- [ ] Replace handwritten CLI with clap derive and semantic process tests.
- [ ] Adapt TUI and web read models without adding compatibility paths.
