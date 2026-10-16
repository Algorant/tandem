---
id: task-83
uid: 2d604094-079a-4f7e-962f-e9c26925d309
type: task
title: "Release Tandem 0.17.1 after accepted docs"
priority: "high"
effort: "medium"
blockers: ["task-76"]
references: ["task-81", "task-27", "task-50", "task-64", "task-63", "task-77"]
relatedFiles: ["tandem/RELEASE.md", "scripts/release_checks.sh", "dist-workspace.toml", "tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md", ".github/workflows/release.yml", ".github/workflows/aur-tandem-bin.yml"]
tags: ["release"]
accord:
  status: "accepted"
  acceptance: ["Only after Algorant accepts the rendered task-76 docs and old-URL handling: release 0.17.1 includes the integrated fix batch, accepted visuals and docs, with version and curated release notes correct.", "Maintained release gates pass, including native dev-check with strict clippy, site build/link/high-severity audit gates and pinned cargo-dist; no global tooling changes.", "Annotated tandem-v0.17.1 tag and GitHub Release are published with every configured release asset and valid checksums; a clean isolated installer installs the released 0.17.1 binary.", "AUR publication and bounded metadata verification are reported; a downstream AUR failure does not block an otherwise verified GitHub release. No unapproved network infrastructure or publication before docs acceptance."]
  claimedAt: "2026-10-16T12:24:58Z"
  deliveredAt: "2026-10-16T12:38:04Z"
  validation: ["$ just dev-check", "$ just site-build", "$ cd site && bun run check:links"]
  constraints: ["Algorant explicitly approved release 0.17.1 via Herdra on 2026-10-14, ordered AFTER docs acceptance. This record captures authorization, not permission to start before task-76 completes.", "Plan first, Luna/high if delegated. Follow current tandem/RELEASE.md and new release gates; do not reintroduce removed release machinery.", "No source push, tag push or docs deployment until Algorant accepts docs/URL handling. Release publication is then authorized; surface any separate deployment decision that remains unresolved.", "Temporary verification installs/downloads only; do not commit runtime state or add permanent tests/fixtures."]
  summary: "Published Tandem0.17.1 after Algorant accepted docs rev5: main+annotatedtag pushed, docs deployed/live verified, release/newgates passed,13assets/checksums verified, live-site install line installed0.17.1 in isolation, AUR0.17.1-1 verified. Browsers/previews stopped; mainclean/synced. Existing low-severity audit/site warnings only."
  evidence: ["Algorant approval via Herdra mcc1d0efb authorized ordered integration/push/deploy/release. Task76 finished via owning Worker lifecycle; docs commit190ad105 plus five held commits pushed before publication. Deploy docs workflow38900358759 successful. Live https://trytandem.dev/ sevenpages loaded with correcttitles/teaser/currentcontent.", "Release commit273659950936cf4b76b2321a5e6648829d1f610d; annotated tandem-v0.17.1 tag andmain pushed. just release0.17.1 exit0, log /tmp/tandem-release-0.17.1.log. Requiredpinnedcargo-dist0.32.0/curatednotes/nativeformat/releasetests337unit+76integration/build/dist/strictclippy/version/site176links/highaudit/websyntax/diff gatespassed beforetagpush. Independent justdev-check passed afterward.", "GitHub Release https://github.com/Algorant/tandem/releases/tag/tandem-v0.17.1 published2026-10-16T12:32:49Z, non-draft/non-prerelease. MatchingRelease workflow38900771655 succeeded. All13configuredassets downloaded/nonempty; aggregate andfive individualSHA256files verified every source/platformarchive. Checksums retained /tmp/tandem-0.17.1-checksums.txt.", "Live12legacyaliases expose correctstaticmeta-redirect destinations; alltargedpages/fragments correspondaccepteddecision12; threeretiredmaintainerpathsHTTP404; teaserHTTP200. Obscura verifiedpage/redirectDOM andclosedalltabs. Itdoesn'tauto-followmeta refresh, so liveverificationchecked redirectmarkup anddestinations, not falselyclaimedObscuranavigation.", "Live Quickstart shows curl-fsSL https://trytandem.dev/install.sh|sh. ExactURL302chain reaches0.17.1generatedinstaller; bytesmatchGitHubasset SHA25672a89fd6b1d3db3e4acd8c807f28565ed2b436097bca506136cb86ef726bf1bb. Executedline in isolatedunmanagedprefix/noPATHorHOMEchanges, binaryreported tandem0.17.1; installedCLIinit/add/claim/deliver/complete/historysmoke passed. Isolatedinstall/downloads removed, logsretained; no globaltoolingpreferences changed.", "MatchingAUR workflow38901296852 succeeded. PublicAUR Git commit a341f6e83529cd2170ddd061d92fd818ac4b9053 is Update tandem-bin to0.17.1; .SRCINFO/pkgver0.17.1/pkgrel1/sourceURLcorrect, LinuxarchiveSHA25671d6094e37c4a6fe7cb98fca73c3501570ee93959a2305de396f2fe3a327d1d5 matchesrelease. RPCinitiallylagged0.17.0-1 thenboundedfreshcheckconfirmed0.17.1-1. Readonlyclone removed; metadataevidence retained.", "No publicationblockers. ExistingAstro emptyi18n/404-entry warnings andonebelow-high auditadvisory remain; highseveritygatepassed. Stalemaintainercapabilityprose captured aslowprioritytask84, notfixedinrelease. Allbrowser tabsclosed; no listeners4321/4322/9224/19576; originaltask76Chromium/Astro alreadyterminated. Gitmaincleanandsynced; no extra servers/networkinfrastructure started."]
  filesChanged: ["tandem/Cargo.toml", "tandem/Cargo.lock", "RELEASES.md"]
  reviewer: "Algorant"
  updatedAt: "2026-10-16T12:38:28Z"
createdAt: "2026-10-14T04:01:41Z"
updatedAt: "2026-10-16T12:38:28Z"
assignee: "tandem"
archivedAt: "2026-10-16T12:38:28Z"
resolution:
  outcome: "completed"
  reviewer: "Algorant"
---
## Explicit publication authorization

Algorant via Herdra approved docs rev5 and release 0.17.1. Parent has finished task-76 through owning Worker lifecycle; main190ad105 includes five held fix/visual commits plus accepted docs. Authorized order: push main, deploy site, verify live pages/redirects; prepare/version/curated notes and run maintained just release0.17.1; verify GitHub assets/checksums, AUR and exact live-site install line. No additional approval gate remains. Stop all browser/preview processes used.

## Direct operator plan

This release is tightly coupled operator publication; parent executes on cleanmain, not a separate publishing Worker. Verify merged site build/link/highaudit before push; match Docs workflow to pushed190ad105 and wait for successful deployment; verify sevenlivecanonicalpages, twelve aliases and three404s plus media/installerURL. Then update Cargo package+lock to0.17.1, add one concise RELEASES section for fixes/docs, commit, and run the authoritative justrelease recipe with pinned cargo-dist/all newgates. Verify all configured published assets/checksums with isolated installer; observe/report bounded nonblocking AUR result. No remote tag rewriting or new network infra. Current docs accepted diagrams/crop unchanged. Retained preview processes already stopped; restart none unless live verification requires Obscura, close it immediately afterward.

Known old README-like prose in maintainer RELEASE.md is not release behavior; do not expand this release into unrelated guide cleanup.