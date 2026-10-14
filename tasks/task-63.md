---
id: task-63
uid: 701b0825-d104-4961-9bdb-804d826d3524
type: task
kind: "papercut"
title: "Concurrent add can return its provisional task-new ID after a parallel sync already numbered it"
state: "in-progress"
priority: "medium"
effort: "medium"
relatedFiles: ["tandem/src/cli/commands.rs", "tandem/src/cli/model.rs", "tandem/src/project/mod.rs", "docs/cli/index.md", "protocol/assignment.md"]
tags: ["protocol"]
accord:
  status: "rework"
  acceptance: ["Two concurrent `tandem add` calls on a Git-backed board each return their final published `task-N` ID in `data.id` whenever `sync.status` is `synced`.", "After a successful save, failure of existing final-ID resolution must retain command success and the known ID, with a clear warning naming the resolution problem; no full-UID refinement or new registry.", "tandem add task --help and tandem update --help explain that `$ <command>` creates a runnable command validation and unprefixed text is a manual check.", "Relevant user documentation includes one command validation example using the required `$ ` prefix."]
  claimedAt: "2026-10-14T02:12:29Z"
  deliveredAt: "2026-10-14T02:23:59Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Fresh Luna/high Worker, plan before code, after task-64; no new ID registry or speculative resolver fallback.", "Fold task-77 help/docs into this CLI-owned outcome. Coordinate docs/cli/index.md before task-76 content Worker.", "Do not introduce another post-save refusal while fixing final-ID response identity. Temporary tests only; no push/release."]
  summary: "Synced `add task` responses now map through the existing `current_id` resolver after applying the initiating sync report's renames. Lookup errors or unresolved provisional IDs retain the known ID and add a save-success warning. Added validation semantics to add/update help and a quoted runnable command example to the CLI reference. Committed as `97d3e9d`."
  evidence: ["Synced concurrent add responses identify their own final published records.: Three native concurrent pairs produced six synced responses. Each `data.id` was a numbered `task-N`; showing each ID returned the exact title belonging to that process. Empty-rename-map responses returned `task-1`, `task-4`, and `task-5` correctly.", "A post-save final-ID lookup problem preserves command success and states the problem.: For synced task adds, both `current_id` error and unresolved provisional return paths append a clear warning and retain the report-mapped ID; neither propagates an error. Existing outcome warnings are preserved in JSON and human output. The error branch was inspected but not forced at runtime.", "Validation help/docs explain runnable commands versus manual checks.: Built-release help shows the requested distinction for both add task and update. `docs/cli/index.md` now includes `--validation '$ just dev-check'` and states unprefixed text is a manual check; assignment semantics were unchanged."]
  filesChanged: ["tandem/src/cli/commands.rs", "tandem/src/cli/model.rs", "docs/cli/index.md"]
  note: "Parent diff/native337+76/fmt/strictclippy/site996links/help review PASSED; implementation scoped and correct. Before finish, resolve the reported verification gap on Herdra's explicit post-save-success rule: add TEMPORARY project-native tests of add_task_response_id for (a) existing current_id Err and (b) Ok but still unmatched provisional. Assert returned knownID, clear save-success/resolution warning, and existing warnings preserved; helper returnsString/no ? is the success boundary, JSONhuman callsite re"
  updatedAt: "2026-10-14T02:24:00Z"
createdAt: "2026-10-03T03:11:29Z"
updatedAt: "2026-10-14T02:24:00Z"
blockers: ["task-64"]
references: ["task-77"]
assignee: "worker-task-63-fdc87a3a"
---

## Description


## Description

Reported from the ~/.pi board (task-327). Reproduced on 0.16.2 on 2026-10-03, 3 times out of 3, with two parallel `tandem add task … --json` calls in a Git-backed workspace with a local bare remote. One call returns its final `task-N` with a `renamed` map that holds both mappings. The other returns its provisional `data.id` (for example `task-new-5fe72d46`) with `sync.status: synced` and an empty `renamed` map. The board ends consistent. Agents must cross-map IDs by hand, and the result is misleading because it claims `synced` while reporting a provisional ID.

The CLI maps the outcome ID through its own sync report only (`report.renamed(&outcome.id)` in `tandem/src/cli/commands.rs`). It never resolves the record's current published ID by uid when another process already numbered it.


## Approved batch D, including task-77

After task-64, resolve synced concurrent-add outcomes through existing current-ID/UID support rather than the initiating command's renames alone. Reproduce with parallel native adds on a disposable Git-backed board and assert every synced data.id names the final published record. Reuse the existing authoritative resolver; no second registry or just-in-case fallback. Review post-save error behavior so the fix doesn't reintroduce the refused-mutation hazard.

Fold task-77: add/update --validation help explains `$ <command>` versus unprefixed manual text; add one verified command example in docs/cli/index.md without changing assignment semantics. Parent will cancel 77 as absorbed only once scope is settled/delivered, and complete from this criterion evidence. Serialize with docs prose so the overhaul imports current help/reference changes.

## Boundary confirmed by Herdra; bounded baseline verified

Herdra approved only the narrow existing current_id resolver/contract. Once saved, command never exits error for final-ID lookup failure: success with known ID and a clear warning naming the problem. No full-UID refinement/new ownership. Parent disposable bare-remote check: three concurrent native pairs, all six synced; three returned provisional IDs with empty rename maps. Existing native show/alias resolution resolved all six to correct published IDs; no ambiguity observed in this bounded practical check (not a proof impossible). Evidence /tmp/task63-parent-baseline-evidence.json; temporary repos removed. Plan now approved: mapped ID then existing current_id for synced outcomes, catch resolver error into warning rather than propagate post-save; help/docs fold77 unchanged. Before extra machinery, report any genuine ambiguity reachability observed.