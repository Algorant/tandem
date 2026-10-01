---
id: task-53
uid: 26e2bc8c-edb8-46d2-98d8-00e628cd44a0
type: task
title: "Seed prebuilt Tandem rules and agent-file pointer on tandem init"
state: todo
priority: "medium"
tags: ["init", "rules"]
accord:
  status: "ready"
  acceptance: ["`tandem init` creates a documented default set of structured rules with stable IDs in the new workspace.", "If `AGENTS.md` and/or `CLAUDE.md` exists at the project root, `tandem init` appends one line directing agents to reference the Tandem rules; it is idempotent and never creates those files when absent.", "Init output reports which rules were seeded and which agent files were updated.", "Tests cover rule seeding and agent-file update (present, absent, already-contains-line cases)."]
  updatedAt: "2026-10-01T18:27:09Z"
createdAt: "2026-10-01T18:27:09Z"
updatedAt: "2026-10-01T18:27:09Z"
---

## Description

Requested by Algorant. `tandem init` should create a default set of prebuilt rules that help Tandem function properly (e.g. use Tandem tools/CLI for coordination, read rules at session start, don't edit `.tandem/` by hand, archived records are read-only). It should also detect an existing `AGENTS.md` or `CLAUDE.md` at the project root and append a short line telling agents to reference the Tandem rules (`tandem rules list`).

Bucket classification (always/never/prefer/context) of the seeded rules is deferred to a follow-up task; initial placement may be provisional.
