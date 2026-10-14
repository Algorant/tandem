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
  status: "delivered"
  acceptance: ["Two concurrent `tandem add` calls on a Git-backed board each return their final published `task-N` ID in `data.id` whenever `sync.status` is `synced`.", "After a successful save, failure of existing final-ID resolution must retain command success and the known ID, with a clear warning naming the resolution problem; no full-UID refinement or new registry.", "tandem add task --help and tandem update --help explain that `$ <command>` creates a runnable command validation and unprefixed text is a manual check.", "Relevant user documentation includes one command validation example using the required `$ ` prefix."]
  claimedAt: "2026-10-14T02:12:29Z"
  deliveredAt: "2026-10-14T02:26:39Z"
  validation: ["$ just dev-check", "$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["Fresh Luna/high Worker, plan before code, after task-64; no new ID registry or speculative resolver fallback.", "Fold task-77 help/docs into this CLI-owned outcome. Coordinate docs/cli/index.md before task-76 content Worker.", "Do not introduce another post-save refusal while fixing final-ID response identity. Temporary tests only; no push/release."]
  summary: "Concurrent synced add-task response IDs and task77 validation help/docs independently verified at97d3e9d. Existing resolver only; post-save lookup error/unmatched provisional retains success+knownID+clearwarning. Parent6 concurrentAdds/titlechecks, assignmentclassification, native suite/site/help pass; temporary native tests exercise both warningbranches andwere removed. Ready technicalacceptance, no newUIDscope or push/release."
  evidence: ["Parent full3-file diff review: onlytask-add response mapping through initiatingrenames then existingcurrent_id forSynced; Result caught into saved-and-synced warning+knownID, helperreturnsString, JSONoktrue/humanNormalexit preserved. Pending/localonly/notgit unchanged; no UID/refinement/registry/preflightownership.", "Parent independently ran justdev-check,fmt,strictclippy,diffcheck,sitebuild and996linkcheck/21HTML: allpassed337unit+76integration nativeproject checks. Sourceclean97d3e9d; no permanenttests/fixtures.", "Parent nativeGit-backed/bare-remote3concurrentpairs: all6 syncedIDsnumbered; showidtitle matchedOWNprocess title. Three emptyrename-mapresponses nowreturnedfinalIDs. Evidence /tmp/task63-parent-verified-evidence.json. Temporaryrepos/helperremoved.", "Worker temporary nativeunit tests induced current_id Err via syntheticUID-prefixcollision andOkunmatchedprovisional; both retainedknownID, existingappwarning andclear saved-and-synced/resolutionwarning. Tests/fixturesremoved; Parent verified sourcebyte-identical97d3e9d. This is faultinjection, NOT practical observedcollision; no extra machinery.", "Parent built-releaseaddtask/updatehelp both state $<command> runnable vsunprefixedmanual. docscli example --validation '$ just dev-check' correct. Parent nativeassignment withquoted$ andmanualtext returned plannedValidation [{kind:command,text:justdev-check},{kind:manual,text:Inspectrendering}], no semanticchange.", "Task77 criteria fullyverified fromthisdelivery; parentwillcancel77asabsorbed aftertask63completion. Context1 authorizes parenttechnicalacceptance; worker_finishowns integration/lifecycle/checkoutcleanup. No push/release/deploy."]
  filesChanged: ["tandem/src/cli/commands.rs", "tandem/src/cli/model.rs", "docs/cli/index.md"]
  updatedAt: "2026-10-14T02:26:39Z"
createdAt: "2026-10-03T03:11:29Z"
updatedAt: "2026-10-14T02:26:39Z"
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