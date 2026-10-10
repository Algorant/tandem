---
id: task-62
uid: 838c6f00-4180-4205-b18a-6154027f96a6
type: task
title: "A validation-invalid local edit blocks every command and has no supported repair"
state: "in-progress"
priority: "medium"
effort: "medium"
relatedFiles: ["tandem/src/project/sync.rs", "tandem/src/cli/commands.rs", "tandem/tests/sync_behavior.rs"]
tags: ["protocol", "validation"]
accord:
  status: "claimed"
  acceptance: ["With one validation-invalid record held on a Git-backed board, reads list the other records with a warning that names the held file, and unrelated mutations still succeed (task-49-10 parity).", "`tandem sync status` lists validation-held edits with their reason, matching what `tandem sync` reports.", "A supported native command (e.g. `sync resolve <id> --keep remote`) restores a validation-held record to the shared version without hand-editing `.tandem/`.", "Regression tests in tandem/tests/sync_behavior.rs cover the three behaviors above."]
  claimedAt: "2026-10-10T14:09:47Z"
  validation: ["$ just dev-check"]
  updatedAt: "2026-10-10T14:09:47Z"
createdAt: "2026-10-03T03:11:28Z"
updatedAt: "2026-10-10T14:09:47Z"
assignee: "worker-task-62-b417d4a2"
---

## Description


## Description

Found with task-337 on the ~/.pi board, and reproduced on 0.16.2 on 2026-10-03.

When a local record parses but fails board validation (for example a subtask whose parent was cleared), sync holds it with "would make the shared board invalid: …". Unlike the unparsable-file case fixed by task-49-10, every later command (`show`, unrelated `update`, and even the update that would make the record valid again) fails on that validation error. `tandem sync resolve <id> --keep remote` refuses with "no open sync conflict for <id>", so the only repair is restoring the file by hand from the `tandem` branch, which agents may not do.

Related inconsistency: `tandem sync` reports the held edit and says "see `tandem sync status`", yet `sync status` shows `held: []`. `project/sync.rs` `status()` returns only the snapshot's unparsable-file holds, not the validation holds that merge adds as `extra_held`.

