---
id: task-63
uid: 701b0825-d104-4961-9bdb-804d826d3524
type: task
kind: "papercut"
title: "Concurrent add can return its provisional task-new ID after a parallel sync already numbered it"
state: todo
priority: "medium"
effort: "medium"
relatedFiles: ["tandem/src/cli/commands.rs", "tandem/src/cli/model.rs", "tandem/src/project/mod.rs", "docs/cli/index.md", "protocol/assignment.md"]
tags: ["protocol"]
accord:
  status: "ready"
  acceptance: ["Two concurrent `tandem add` calls on a Git-backed board each return their final published `task-N` ID in `data.id` whenever `sync.status` is `synced`.", "tandem add task --help and tandem update --help explain that `$ <command>` creates a runnable command validation and unprefixed text is a manual check.", "Relevant user documentation includes one command validation example using the required `$ ` prefix."]
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Fresh Luna/high Worker, plan before code, after task-64; no new ID registry or speculative resolver fallback.", "Fold task-77 help/docs into this CLI-owned outcome. Coordinate docs/cli/index.md before task-76 content Worker.", "Do not introduce another post-save refusal while fixing final-ID response identity. Temporary tests only; no push/release."]
  updatedAt: "2026-10-13T13:54:48Z"
createdAt: "2026-10-03T03:11:29Z"
updatedAt: "2026-10-13T13:54:48Z"
blockers: ["task-64"]
references: ["task-77"]
---

## Description


## Description

Reported from the ~/.pi board (task-327). Reproduced on 0.16.2 on 2026-10-03, 3 times out of 3, with two parallel `tandem add task … --json` calls in a Git-backed workspace with a local bare remote. One call returns its final `task-N` with a `renamed` map that holds both mappings. The other returns its provisional `data.id` (for example `task-new-5fe72d46`) with `sync.status: synced` and an empty `renamed` map. The board ends consistent. Agents must cross-map IDs by hand, and the result is misleading because it claims `synced` while reporting a provisional ID.

The CLI maps the outcome ID through its own sync report only (`report.renamed(&outcome.id)` in `tandem/src/cli/commands.rs`). It never resolves the record's current published ID by uid when another process already numbered it.


## Approved batch D, including task-77

After task-64, resolve synced concurrent-add outcomes through existing current-ID/UID support rather than the initiating command's renames alone. Reproduce with parallel native adds on a disposable Git-backed board and assert every synced data.id names the final published record. Reuse the existing authoritative resolver; no second registry or just-in-case fallback. Review post-save error behavior so the fix doesn't reintroduce the refused-mutation hazard.

Fold task-77: add/update --validation help explains `$ <command>` versus unprefixed manual text; add one verified command example in docs/cli/index.md without changing assignment semantics. Parent will cancel 77 as absorbed only once scope is settled/delivered, and complete from this criterion evidence. Serialize with docs prose so the overhaul imports current help/reference changes.