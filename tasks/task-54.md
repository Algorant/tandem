---
id: task-54
uid: 21fd4ca1-27cc-4101-93f0-ecbe3996ffad
type: task
title: "Classify seeded init rules into always/never/prefer/context with jev"
state: todo
priority: "low"
blockers: ["task-53"]
references: ["task-53"]
tags: ["init", "rules"]
accord:
  status: "ready"
  acceptance: ["Each seeded init rule has an agreed category (always/never/prefer/context), reviewed with jev.", "`tandem init` seeds rules in their agreed categories, with tests updated."]
  updatedAt: "2026-10-01T18:27:14Z"
createdAt: "2026-10-01T18:27:14Z"
updatedAt: "2026-10-01T18:27:14Z"
---

## Description

Follow-up to task-53, for some later day. Work through the prebuilt rules seeded by `tandem init` together with jev and decide which category each belongs in: `always`, `never`, `prefer`, or `context`. Update the seeded defaults accordingly.
