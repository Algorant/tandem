---
id: task-93
uid: 20050a50-e6ab-49a9-aacc-8d9d09e435ff
type: task
title: "Live read-only SSH workspaces in the TUI"
state: "in-progress"
priority: "medium"
effort: "large"
references: ["task-92", "task-86"]
relatedFiles: ["tandem/src/app/project.rs", "tandem/src/app/queries.rs", "tandem/src/app/dto.rs", "tandem/src/cli/model.rs", "tandem/src/cli/commands.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/state.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/rules.rs", "tandem/src/tui/decisions.rs", "tandem/src/tui/validation.rs", "tandem/src/tui/workflow_prompt.rs", "tandem/src/tui/theme.rs", "tandem/Cargo.toml", "tandem/Cargo.lock", "docs/tui/index.md", "docs/cli/index.md"]
tags: ["tui", "ssh"]
accord:
  status: "blocked"
  acceptance: ["Local-primary TUI accepts repeatable HOST:/absolute/path addresses through --add-workspace and the chooser; canonical session entries are ephemeral and visibly identify host/remote/read-only, while local workspaces retain current behavior.", "One native JSON snapshot command supplies complete read-only Board/Logs/Rules/Decisions/details/activity/config/diagnostic state using shared app/protocol projections, with an opaque content-sensitive conditional revision covering records, rules including enabled state, events and board-side display/theme inputs; snapshots exist only in client RAM.", "Remote operations invoke the owning machine's compatible installed Tandem over short-lived SSH; sameTandemminor0.18.x (patchesmaydiffer) and matchingboardprotocol are required with clear errors. No local board copies/mounts/phantom localproject, daemon/service or remote mutation/editor/PTY API is introduced.", "Every remote board write affordance and shortcut is hidden/disabled with a plain read-only/edit-on-owner hint before any mutation; filters/navigation/details/session controls remain usable. Tests prove no writes are sent remotely or accidentally applied to the local primary board.", "Active remote reads ~2s, visible chooser summaries ~5s and owner-native sync freshness ~30s run off the input/render loop; unchanged revisions avoid repeated full payload/Git snapshot work, changes and selection update correctly, and all owned child work is reaped on switching/exit/error without offline cache/queue machinery.", "Temporary isolated local/SSH fixture checks cover address/quoting/compatibility, complete/tolerant data and revision changes, allread-only gates/localeditparity, liveness/idleI/O and cleanup. Native tests/fmt/strictclippy and applicable docs build pass; actual renders and final address/compatibility guidance are supplied for the authorized0.18.0 rollout/Herdra task41 handoff."]
  claimedAt: "2026-10-22T19:34:16Z"
  validation: ["$ cargo fmt --manifest-path tandem/Cargo.toml -- --check", "$ cargo clippy --manifest-path tandem/Cargo.toml --all-targets --all-features -- -D warnings"]
  constraints: ["HOLD by Algorant via Herdra m7b9810e6: preserve the stopped retained Worker and current dirty compiling seam unchanged. No source edits, builds, tests, SSH/SCP, fixtures or further investigation until explicit owner resume.", "When resumed, behavioral validation is E2E-only under the new global testing rule: no unit tests. Earlier unit/full-suite test directions are superseded; choose concrete E2E checks on resume, without running just dev-check's unit suite.", "Read-only remote product scope remains unchanged; the proposed larger selected-session/read-model migration is NOT approved by this hold. Resolve scope before further implementation.", "No remote record-authoring/editor/PTY API, no copies/mounts/services/offline machinery or new infrastructure; no real cart-lab record writes ever.", "Parent owns future scope approval/resume, integration, release, upgrades and cleanup. Do not discard, reset, commit or remove retained dirty work while held."]
  note: "Algorant via Herdra m7b9810e6: HOLD while focusing on cart-lab. Retain exact stopped Worker/dirty compiling seam, no further work until explicit resume. Worker already settled(done); worker_pause correctly made no runtime changes. On resume E2E-only/no unit tests, and expanded model migration still needs approval. No source/build/test/SSH activity performed."
  updatedAt: "2026-10-22T19:57:05Z"
createdAt: "2026-10-22T19:33:57Z"
updatedAt: "2026-10-22T19:57:05Z"
assignee: "worker-task-93-1b445c3a"
---

## Description

Approved reducedscope fromAlgorant viaHerdra m56365c6d; researchplan task92 iscompleted. Add native live READ-ONLY remote workspace addresses HOST:/abs/path in current local-primary TUI session/chooser. Execute remote machine's own Tandem on demand; no copies/mounts/services/leftoverremoteprocesses. SnapshotJSON aggregates all renderable state using app-owned projections; existingcompactlistJSON isinsufficient. Conditionalrevision must cover records, Rules, events andworkspaceconfig (existingReadSnapshotrevision omits someinputs, don'treuseblindly). Client keeps only RAM readmodel; no local filesystem access through remote paths or dummyTandemProject.

CurrentTuiApp concreteworkspace/reload/logs/events/theme andactionentrypoints arelocalcoupled. Concreteplan mustname minimalreadbackend/data path and enforce readonly atmenus,shortcuts,prompt submission anddirecthandlers so no remoteoperation or local-primarymutation escapes. Retain read/navigation/filter/search/detail/session controls and safe selection. Localactualwrites remainunchanged; no editor or write backend forremote.

Use same0.18.x minorandboardprotocol, patchesallowed. Snapshotpoll~2sactive/~5schooservisible; nativeowner sync refresh~30s selectedboard. AsyncboundedSSH child work only, safeaddress/path/revisionarguments, clearfailures,nofallback/offlinequeue. Preserve task86 no fullGitpublish perunchangedpoll. First concreteplan shouldspell read-command, revision semantics, versionhandshake, sync/childcleanup, displaysettings ownership, fixture/teststrategy androughsize. If scopeexceeds~15–22files/1.5–2.5kLOC materially orneedsdifferentmachinery, stopforparentdirectionbeforeediting.

Baseline main66a87b0 released0.17.3, desktop/archbox/x1nano already17.3. x1nanohasoneexistingTUI; do nottouch realrunningTUIs. Tests mayuseisolatedtemporaryfixtures/binaryartifacts with explicitparentcoordination, no realcart-labedits or configchanges. Publicdocs/artifacts genericfictionalhosts/paths. Parent willship0.18.0afterverifieddelivery,upgradeallmachines andhandaddress/compatcontract toHerdra task41; no httchanges here.


## Concrete plan approval — code GO, fixture setup deferred

Revisedneeds-inputplan reviewed. Centralplan accepted with these explicitbounds: native workspace snapshot --json [--if-revision REV] [--path ABS_PATH], same0.18.x+boardprotocol, contentrevisionallinputs, Local/SSHreadidentity/readonlygates, active2s/chooser5s and selectedREMOTE owner-native30ssync. EarlybranchCargo/lock0.18.0 bump permitted, no tag/push/release. ~20–25existingfiles isacceptable ifextraactionfiles aremechanicalreadonlyguards andLOC~1.5–2.2k.

Preserve dependency direction: app/project cannot import tui types/functions. Extract onlyneutral tolerantreadcomposition/diagnostics downward where needed; UI rendering and personaltheme merge stayTUI. Reuse/extend authoritative app/protocol readprojections rather than frontendMarkdown parsing, fakeLocalproject or independentdomainlogic. No genericstorage/action/RPCframework. Wiremetadata should use unambiguous tandemVersion and boardprotocol protocolVersion (not duplicateboardProtocolVersion/undefinedtransportversion).

Prefer existing TUI trackedthread/job and stdprocess style forSSH; Tokio currently servesweb and doesnotjustifyaddinga TUIruntime/framework. Ifruntime/dependency/semanticrefactor becomesnecessary, stopfirst. SSH hostvalidation/POSIXquoting mustcoverreadpath/revision; rejectcontrolchars, handleapostrophes/spaces, standarddeadlines/reaponlyownedchildren,no multiplexer/service.

No newloopbacksshd/account/SSHprofile orotherinfrastructure. Code/build/temporarylocalchecks mayproceed; ANY remoteSSH/SCPfixture setup requires a focusedparentquestion naming exactexistingalias,tmpdirectory,binarydeployment/invocation strategy andcleanup. Useexistingarchbox/x1links andgeneratedtempboards only; neverrealcart-labrecordmutation. No newproductiontestflags. Parent coordinatesfixturecommands afterreview.