---
id: task-24
type: task
title: "Define a runnable validation form in the protocol and assignment read"
state: todo
priority: "medium"
effort: "small"
relatedFiles: ["protocol/README.md", "protocol/assignment.md"]
tags: ["protocol", "validation", "delegation"]
accord:
  status: "ready"
  acceptance: ["protocol/README.md defines a validation entry beginning with `$ ` as a runnable check: the remainder is a shell command run from the Task's repository root, exit 0 is pass; all other entries are manual checks.", "`assignment --json` returns `plannedValidation` items as `{ \"kind\": \"command\" | \"manual\", \"text\": \"...\" }` with the `$ ` prefix stripped for command entries; `show --json` keeps the raw strings unchanged.", "The protocol states that adapters should require captured command output on delivery for command entries and may refuse integration on a non-zero exit, and that Tasks with no command entries are unaffected.", "An explicit adapter handoff is recorded for pi-tandem/pi-agency (see ~/.pi task-105); this Task does not modify extensions/pi-tandem."]
  validation: ["$ cargo test -p tandem", "Live: add a Task with --validation '$ true' --validation 'Read the output'; `tandem assignment <id> --json` returns one command and one manual item; `tandem show --json` returns the two raw strings."]
  constraints: ["String convention only; no new CLI flag or structured frontmatter field in this Task.", "Do not execute validations inside Tandem; it classifies, adapters run."]
  updatedAt: "2026-09-09T14:22:46Z"
createdAt: "2026-09-09T14:22:46Z"
updatedAt: "2026-09-09T14:22:46Z"
---

## Description


## Source

Same review as the attempt-count Task. In the dotfiles epic every Accord's acceptance and validations were prose, so the only correctness check was the orchestrator reading Worker prose against Task prose. Making a validation entry runnable lets the Worker check its own definition of done and lets the adapter refuse a merge on a failing check. Tandem's part is small: name the convention in the protocol and classify entries in the assignment projection so adapters do not each invent their own parsing.

The Pi adapter will use the `$ ` prefix immediately (~/.pi task-105) and switch to the `kind` field once this lands.

