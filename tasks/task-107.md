---
id: task-107
uid: b23a7f31-8cab-4b75-bce8-80e7253c8c80
type: task
title: "Public repo review 2 of 6: tandem cleanup, simplification and look"
state: todo
priority: "medium"
effort: "medium"
tags: ["docs", "ui"]
accord:
  status: "delivered"
  acceptance: ["README, docs and banner are reviewed against Algorant's repo style rules (global memory note repo-style-rules.md) with the pi README as the worked example, and a proposed README plus banner are shown rendered as GitHub shows them.", "Cleanup and simplification findings name concrete files or sections with evidence for removal, and clearly separate what applies to an already-public repo from release-readiness checks.", "One Sideshow post in a new session presents current README, proposed README, cleanup list with evidence and a recommendation; nothing in the repository is changed before Algorant approves."]
  deliveredAt: "2026-11-11T18:33:36Z"
  summary: "Shipped Algorant's approved public-face review as a356a45 and released 0.17.7: new README and banner, GitHub About filled, planning trees and extensions removed, docs ownership consolidated, teaser dropped, hostname-free board sync commits. Desktop and homelab verified on 0.17.7; x1nano was offline."
  evidence: ["Commit a356a45 (pushed to main): README rewritten with docs/banner.svg and no H1; banner copied to docs/ and site/public/; plan/, protocol/plan/, tandem/plan/, extensions/ removed (29 files); tandem/README.md cut from 306 to ~40 lines after its unique config/web facts moved into docs/reference/configuration.md and docs/tui/index.md; AGENTS.md locked-decisions and layout sections replaced with pointers; docs compatibility collapsed to one line; product name lowercased across docs and site title; orphan tui-session-chooser.png and the teaser mp4/poster removed; docs/index uses the banner; gitguard comment self-contained; Brainfile linked.", "GitHub About set via API: description 'local-first coordination for people and agents, in plain Markdown', homepage https://trytandem.dev, topics agentic/agents/cli/local-first/markdown/project-management/rust/tui; license MIT detected. Tree no longer shows plan/ or extensions/.", "Verification: cargo fmt, strict clippy, release build, just dev-check (all suites) and just site-build passed; site link check 175 links across 20 pages. Live GitHub page shows the banner, intro and Install block; trytandem.dev shows the banner and lowercase title after docs workflow 43077318001 succeeded (captures kept locally).", "Release 0.17.7 verified: GitHub Release with required assets/checksums and curated notes; desktop and cart-lab upgraded and both actual binaries report tandem 0.17.7 with SHA256 6dc87bfd…; x1nano unreachable (SSH timeout) and remains on 0.17.6. AUR job failed again on the open Arch openssl packaging issue (task-104); release stands per never-5.", "Teaser decided: dropped rather than re-recorded, because its agent-side lines show the retired pi-tandem adapter and re-filming 88 s is not a small change. Kept: site/maintainers theming pages, tandem/AUR.md, RELEASES.md."]
  updatedAt: "2026-11-11T18:33:36Z"
createdAt: "2026-11-11T17:01:22Z"
updatedAt: "2026-11-11T18:33:36Z"
---

## Description

Requested by Algorant via Herdra m62a9ffc3. His words: "Ok start the review of public repo 2. make sure to factor in or try to apply the guidelines we were using and collecting from our pi repo investigation. tandem is basically already public so much of it probably won't apply as far as 'ready to release' but definitely as far as cleaning up things or simplifying things and having appropriate look/format, yes."

Review scope (read-only until approved):
1. README and public docs against the repo style rules (lowercase names, first-person voice, short showroom tone, short "How to use this repo", no sentence under headings, hierarchy not a pile, no full stops on list items, banner leading the page with no H1, License last as `## License` → `[MIT](LICENSE)`, etc.), using github.com/Algorant/pi's README as the worked example.
2. Plain MIT LICENSE file; main pushed; nothing stale on the public face.
3. Cleanup and simplification: cruft, stale or deprecated files and scripts, docs that restate each other, anything that does not earn its place, with evidence per removal.
4. Banner proposal in the pi banner's family (~/.pi docs/banner.svg).

Deliverable: one Sideshow post in a new session "Public repos: tandem" with the current README rendered as GitHub shows it, the proposed README rendered the same way, the cleanup list with evidence, and a recommendation. gitguard scans the repo separately; no history rewrite.
