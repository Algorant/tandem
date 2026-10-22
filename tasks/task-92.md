---
id: task-92
uid: 903edb6b-5b88-4c2d-9b32-277488a49c51
type: task
kind: "research"
title: "Plan live editable SSH workspaces in the Tandem TUI"
state: "validation"
priority: "medium"
effort: "medium"
relatedFiles: ["tandem/src/app/project.rs", "tandem/src/cli/model.rs", "tandem/src/cli/commands.rs", "tandem/src/tui/mod.rs", "tandem/src/tui/reload.rs", "tandem/src/tui/editor.rs"]
tags: ["tui", "ssh"]
accord:
  status: "delivered"
  acceptance: ["A source-grounded plan is sent through Herdra to Algorant BEFORE implementation, naming read/write commands, live refresh, touched modules, approximate size and material risks, with no more than four questions.", "The plan honors on-demand owner-machine Tandem execution over SSH, fully editable remote TUI boards, explicit compatible-version errors, and no copies/mounts/always-on service or special offline machinery."]
  claimedAt: "2026-10-22T18:50:46Z"
  deliveredAt: "2026-10-22T18:59:15Z"
  validation: ["$ tandem tui --help"]
  constraints: ["Planning/read-only investigation only; no source/protocol changes, remote-board writes, new infrastructure or implementation dispatch until Algorant approves the plan.", "Herdra research task39's mirror/readonly recommendation is superseded by Algorant's approved SSH-owner execution shape; do not implement htt here."]
  summary: "Source-grounded SSH workspace plan submitted throughHerdra beforecode; 3productquestions pending. LargeLocal/SSHbackend refactor with nativeaggregate read, owner-machine CLIwrites, separate live cadence and minor-release rollout; no source/remote changes."
  evidence: ["Read Herdra research/task39 and explicitly superseded its mirror/readonly path with approved SSH-owner execution shape.", "Two bounded read-only source audits completed: ssh-tui-seams and ssh-cli-contract. Parent checked concreteTuiApp/localreload/editor, ReadSnapshot/DTO/CLI compactlist gaps, Validation accept/rework serverpreconditions and sync cadence. Existing rawEDITOR path lacks CLI parity; no hidden claim of full coverage.", "Native tandem0.17.3 tui --help verified current localPath-only addworkspace; gitstatus clean/synced. No source/spec change, remoteexperiment/write, mount/service, htt edit or implementationWorker launched.", "Plan mc82876f6 names new workspace snapshot JSON read, existing write families +missingthinfrontdoors, active2s/remote30s/chooser5s cadence, moduleblast radius/size, risks and3questions. AwaitHerdra/Algorant approval before concreteimplementationcontract/Worker."]
  updatedAt: "2026-10-22T18:59:15Z"
createdAt: "2026-10-22T18:48:19Z"
updatedAt: "2026-10-22T18:59:17Z"
assignee: "tandem"
validation.criterion: "A source-grounded plan is sent through Herdra to Algorant BEFORE implementation, naming read/write commands, live refresh, touched modules, approximate size and material risks, with no more than four questions."
validation.note: "Plan and3questions submitted viaHerdra mc82876f6; sourceauditcomplete, no code. Need decisions about remoteEDITOR/fulltaskeditparity, remote-onlystartupscope and initialsame-minorcompatibility. Nativeaddress/backend/snapshot design requires approval before implementation."
validation.requestedAt: "2026-10-22T18:59:17Z"
validation.reviewer: "Algorant"
---

## Description

Algorant via Herdra m6b55556c requests native remote addresses such as --add-workspace cartlab:/home/ivan/cart-lab using ssh aliases. Remote machine's own compatible Tandem must carry reads and writes; ownership/locks/sync remain there. Live full editable TUI like local; nothing left running between on-demand operations, no copies/mounts/network web service. Connectivity given; fail clearly instead of offline frameworks. Read ~/Projects/herdra/research/task-39.md as prior-source investigation, not current direction. Plan few lines plus <=3–4questions, then approval before code. Future delivery is a minor Tandem release and coordinated desktop/archbox/x1nano upgrades, followed by Herdra task41 htt integration/realcartlab verification with Algorant.


## Source-grounded proposal submitted to Herdra (mc82876f6)

0.17.3 TuiApp/roster/reload/actions hold concrete local TandemProject and filesystem paths. Existing list JSON has only id/title; a coherent editable view needs one native workspace snapshot --json aggregate using shared app read projections, all Rules/events/config/diagnostics, conditional opaque revision covering all render inputs. No dataset persisted locally. Preserve local tolerant diagnostics, rather than silently replacing them with stricter CLI loading behavior.

Use one TUI-facing Local/SSH backend boundary, retaining local app authority and executing remote own Tandem CLI for reads/writes. Existing mutation families add/update/accord/review/complete/cancel/link/rules cover structured actions; thin CLI entrypoints needed for UI-only Validation preconditions and remote raw editor. App logic/locks/IDs/actor ledgers/sync stay remote; no protocol duplication, copies, mounts, service, generic storage framework or htt code here.

Active remote snapshot checks ~2s, mutation-triggered refresh, remote sync ~30s, chooser-visible summaries ~5s, all off UI thread. No250msSSH loop or per-poll fullGitpublish; conditional revisions avoid full history transfer when unchanged. Clear errors, no write retry/queue or silent offline fallback. Personal presentation preferences local; remote workspace config consumed as data. Address grammarHOST:/absolute/path, alias-basedSSH, clear host/path labels, ephemeral canonical-deduped roster.

Likely20–30files/2–4kchangedlines (rough): app/project workspace/snapshot/SSH, CLI model/commands/DTO, TUI state/roster/reload/editor/theme and action modules, docs. Risks: argument quoting; exact validation preconditions; saved-versus-sync-pending/unknown outcome; history payload/revision correctness; idleI/O; remote editorPTY. Existing e opens active tasks in localEDITOR and is the current full raw task editing route; excluding it would reduce like-local task editing parity.

Questions for Algorant: (1) remote-hostEDITOR over temporarySSHPTY (recommended), versus explicitly defer raw task editing? (2) keep local-primary mixed sessions ashtt uses (recommended), versus remote-only startup too? (3) sameTandemminor0.18.x plus matchingboardprotocol for initial compatibility, patch versions may differ?

After approval, obtain isolatedLuna/highWorker concreteplan BEFORE edits, verify isolatedSSHfixtures/allactions/cleanup/nativechecks/actualrenders, shipminor0.18.0 and upgradeallmachines, then handaddress/compatibilitycontract toHerdra task41 forrealcartlabverification withAlgorant. No realcartlabwrites during planning/pre-release tests.