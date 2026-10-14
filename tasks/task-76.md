---
id: task-76
uid: 5f373353-a0d5-4373-87fc-5c533f439c89
type: task
title: "Docs and site overhaul: write the approved seven pages"
state: "todo"
priority: "medium"
effort: "large"
references: ["task-51", "task-67", "task-70", "task-71", "task-72", "task-73", "task-74", "task-75", "task-81", "task-27", "task-50", "task-63"]
relatedFiles: ["docs/", "site/", "README.md", "tandem/README.md", "protocol/README.md", "plan/docs-overhaul.md"]
tags: ["docs"]
accord:
  status: "ready"
  acceptance: ["The public site retains the approved seven-page shape, starts with minimal reader-facing prose and incorporates Herdra's approved style guide when available.", "Content accurately explains shipped 0.17.0 session boards, add/remove paths, full-TUI switching, chooser activity, active-board sync versus inactive local snapshots, and agent setup; optional Pi /task routing is not presented as a Tandem CLI command. Web stays single-workspace/read-only.", "Workspace layout, lifecycle/review claims, installation/version guidance and Decision examples from tasks 51/67 are verified and corrected across docs and READMEs.", "Real 0.17.0 product captures, authorized Excalidraw diagram exports and existing teaser presentation are legible and reviewed in the existing docs Sideshow session with main-versus-branch comparisons.", "Site build and link checks pass; public pages contain no placeholders; final integration/deployment occurs only after Algorant's approval.", "A dedicated board-storage/sidecar-sync section within the seven-page shape accurately explains source Git versus the Tandem-managed tandem branch, multi-machine use, pending, true conflicts and held edits, with the required Excalidraw diagram and real sync-status/TUI-conflict captures."]
  validation: ["$ just site-build", "$ cd site && bun run check:links"]
  constraints: ["Plan first; Luna/high. Herdra approves this docs writing plan before implementation.", "Currentmain integrated fixes+acceptedvisuals is source; old docs-overhaul57381c9 is preserved reference, not allowed to overwrite currentCLI docs or maintained releaseguide.", "Minimal real-reader prose, seven pages, approved sparse chrome; no narrator/padding/future claims. Apply approved styleguide ifavailable.", "Reuse accepted diagrams/captures/logo/teaser unchanged; no vault/exporter/product/CLI/protocol semantic change.", "Rendered pages and old-URL handling choices go to Herdra in J0qH-JN7PqI before Algorant; no URL decision silently encoded.", "Hold integration/push/deploy until Algorant docs acceptance. Then approved release0.17.1 through maintained gates."]
  note: "Algorant restarted planning via Herdra after 0.17.0 landed. Refreshed plan awaits sign-off; no page writing or implementation Worker yet. Rebased scaffold onto main; Excalidraw export/use decision surfaced."
  updatedAt: "2026-10-14T04:00:49Z"
createdAt: "2026-10-12T15:51:52Z"
updatedAt: "2026-10-14T04:00:49Z"
blockers: ["task-81", "task-50", "task-63"]
---
## Current authorization — Algorant via Herdra

Visuals task81 accepted as-is (combined Work and Review WITH current labels, D4, ninecaptures), integrated main20a2538. Fixes27/59/68/80,50,64,63(+77) integrated; no sourcepush yet. Start ONE Luna/high content/layout Worker now on currentmain. PLAN FIRST; HERDRA approves the plan, not parent alone. Rendered pages go to Herdra in Sideshow J0qH-JN7PqI BEFORE Algorant. Bring old-URL handling choices/map with rendered review; do not silently implement redirects or decide removal. NO push/deploy/integration until Algorant docs acceptance. Approved release0.17.1 follows accepted docs through newrelease gates, not before.

## Seven-page shape and real-reader content

1 Home: one-line purpose, existing encoded teaser/poster, minimal install link. 2 Quickstart /quick-start/: install+verify, native init→claim→deliver→complete→Logs workedTask with real short textoutput. 3 Concepts /concepts/: Task/Accord/lifecycle/Decision/Rules basics, combinedwork/review image, dedicated sourceGit versus literal tandem-branch storage/sync section with D4. 4 Agents /agents/: tool-neutral short AGENTS.md snippet/example; optional Pi routing metadata clearly integration-specific, NOT TandemCLI. 5 TUI and web /tui/: shipped ephemeralmanual working set (w/Ctrl-A/Ctrl-D inactive removal/fullTUI selection), local activity/recordedassignees, activeboard ongoingsync vs inactive localsnapshots; single-workspace read-onlyweb. 6 CLI /cli/: compactcommand/keyflag/JSONreference, repeatable --add-workspace, preserve verified runnable '$ ' guidance. 7 Configuration /reference/configuration/: supportedtheme/badge settings and precedence; no persistent roster.

Write minimal for actualreader, no tour/narrator/padding/speculativefuture features. Herdra task19 remains unstarted (read2026-10-14); apply known taste: startsmall, fewer shortpages, plainnames, smallheadings/tightspacing/noanchor icons/pagination, linkconfigs/projects instead ofexplaining everything. Any publishedapproved guide later supersedes interimstyle.

## Source/scaffold/assets

Worker checkout must start currentmain with integrated fixes+11visual assets. Historical approved scaffold is clean docs-overhaul57381c9 at /home/algorant/Projects/worktrees/tandem/docs-overhaul, fourcommits0f86813/44d228d/324801e/57381c9. READ-ONLY reference until planapproval; propose minimal import/adaptation, do not overwrite currentCLI validation docs or task27 tandem/RELEASE.md. Oldscaffold plan/docs-overhaul.md is historical0.16.3 audit, notcurrentbrief. Preserve rootREADME routingdescription metadata and alreadyapproved logo. README/main-area claims may be trimmed/corrected, notbroad protocol semantic rewrite. Maintainer docs leavepublicsite/search but keep maintained releaseguide correct.

Assets unchanged: site/public/images/work-and-review.png, source-and-tandem-sync.png, nineactual0.17captures; no old shared-board/task-lifecycle PNGreferences. Editable EA sources+PNGpairs in authorized vault alreadyverified; no newdiagram/exporttool or vaultedits. Terminalprovenance stays reviewnotes only; web isgenuineChromium, no CSSmasking. Existing site/public/media/tandem-teaser.mp4+poster retained, original fframes source read-only. Use relevantcapture budget; no placeholders/privatehosts/paths/names. NoHerdrdiscovery/permanentworkspaces.toml/multiworkspaceweb. Newstructured eventcontract requiredts/event/id/actor/seq/data, no newsummary, legitimatehistory retained. Resolverpostsave problems preserve successwithwarning; don't claimoldpublishedbinary contains unreleasedfixes. Plan versionwording for upcoming0.17.1 without claiming itreleased beforepublication.

## Required plan/review/verification

Concrete pageoutline and approximatecontentbudget, assetplacement/caption budget, exactfiles and scaffoldimportmethod, cuts/maintainer relocation, Quickstartcommands+verification, versionclaims, oldURLinventory/options tobringwith renders. No implementation until Herdraapproval. Native '$ just site-build' and '$ cd site && bun run check:links'; check installedbun audit highseverity, light/dark desktop+mobile renderedpages, actuallinks/alt/media, no placeholders. Verify usercommands againstcurrentnative build/protocol0.5.0 on disposableboards. Everymeaningfulreview revision NEW post in SAME J0qH-JN7PqI with MAIN-vs-BRANCH renderedcomparisons, outline/cutlist and boundedMagicDNSHTTPcheck. Parent independently verifies actualpages then Herdrareview then Algorant acceptance. Do not push/deploy or startrelease beforeacceptance.