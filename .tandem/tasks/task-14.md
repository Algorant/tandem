---
id: task-14
type: task
title: "CLI: review accepts a criterion that is not in the accord"
state: todo
priority: "low"
tags: ["papercut"]
accord:
  status: "ready"
  acceptance: ["review rejects a criterion string that does not match an existing accord acceptance entry, or the exact-criterion requirement is dropped from protocol/README.md."]
  updatedAt: "2026-09-04T01:36:35Z"
createdAt: "2026-09-04T01:36:35Z"
updatedAt: "2026-09-04T01:36:35Z"
---

## Description

Observed while checking whether Pi could rely on the CLI to catch a mistyped criterion.

protocol/README.md says review `requires the exact unresolved criterion`. It is not enforced. Reproduction on 0.12.2:

```
tandem add task alpha --acceptance 'criterion one'
tandem accord claim task-1 --assignee me
tandem review task-1 --criterion 'totally made up' --note n -j
→ {"data":{"id":"task-1","state":"validation"},"ok":true,"warnings":[]}   exit 0
```

The task enters validation and `validation.criterion` stores the arbitrary string while `accord.acceptance` still holds only the real criterion. A human reviewing the escalation sees a criterion that exists nowhere in the agreement.

Not a Pi dependency. Either enforce the match or drop the claim from the protocol text so the two agree.
