---
id: task-77
uid: 49829e7d-bd9b-49cf-a941-0ea494dddf1d
type: task
kind: "papercut"
title: "Explain runnable $ validation syntax in add/update CLI help"
state: todo
priority: "low"
relatedFiles: ["tandem/src/cli", "docs/cli/index.md", "protocol/assignment.md"]
tags: ["config", "delegation"]
accord:
  status: "ready"
  acceptance: ["tandem add task --help and tandem update --help explain that `$ <command>` creates a runnable command validation and unprefixed text is a manual check.", "Relevant user documentation includes one command validation example using the required `$ ` prefix."]
  updatedAt: "2026-10-12T16:29:42Z"
createdAt: "2026-10-12T16:29:42Z"
updatedAt: "2026-10-12T16:29:42Z"
---

## Description

Observed while dispatching task-72: an Accord validation string 'just dev-check' looked executable in show output, but assignment correctly classified it as manual; Worker dispatch refused medium effort as having no runnable validations. protocol/assignment.md requires a leading '$ ' to classify a command. add/update --help currently shows only --validation <VALIDATION> without explaining the distinction. Preserve protocol semantics; make the authoring guidance explicit instead of forcing users to discover the prefix after dispatch failure.
