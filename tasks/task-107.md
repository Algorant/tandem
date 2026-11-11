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
  status: "claimed"
  acceptance: ["README, docs and banner are reviewed against Algorant's repo style rules (global memory note repo-style-rules.md) with the pi README as the worked example, and a proposed README plus banner are shown rendered as GitHub shows them.", "Cleanup and simplification findings name concrete files or sections with evidence for removal, and clearly separate what applies to an already-public repo from release-readiness checks.", "One Sideshow post in a new session presents current README, proposed README, cleanup list with evidence and a recommendation; nothing in the repository is changed before Algorant approves."]
  deliveredAt: "2026-11-11T17:09:54Z"
  summary: "Reviewed tandem's public face read-only against the repo style rules and published one Sideshow post: current and proposed README renders, a pi-family banner proposal, a cleanup table with evidence, and a recommendation. Nothing in the repository was changed."
  evidence: ["Read the global repo-style-rules note in full and used github.com/Algorant/pi's README as the worked example. Proposed README leads with docs/banner.svg, has no H1, keeps the intro to two first-person sentences, a short 'How to use this repo', a three-entry Core hierarchy and License last as [MIT](LICENSE); list items have no full stops.", "Rendered the current and proposed README with GitHub's own markdown stylesheet (github-markdown-css dark, 819px) and captured the live public page: LICENSE is plain MIT (21 lines, GitHub reports MIT), main is clean at 56344c9, and the About panel has no description, website or topics (gh api: both null, topics []).", "Cleanup evidence: plan/ + protocol/plan/ + tandem/plan/ + extensions/plan/ are 29 tracked files and 8,539 lines with no references outside archived board logs; extensions/ is 76 lines with no code, retired adapter per decision-10; site/maintainers/docs-site.md (207 lines) restates site/README.md (147); install and rules-pointer text repeat across README, quickstart, docs/agents, docs/cli and docs/tui.", "Banner: 1280×320 SVG in pi's family (flat #0d1117, mark #ebdbb2, caption #8ec07c, floor #736f5e), all text as paths (no fonts), contrast 13.8:1 mark and 8.99:1 caption on the plate; rendered on a light page and on GitHub dark.", "Published as one post in a new Sideshow session TImMSjKQcGU (post f_xHL4p0f0c) and verified HTTP 200 on both localhost and the desktop MagicDNS URL. No repository files, board records of other projects, or history were changed."]
  updatedAt: "2026-11-11T18:33:35Z"
createdAt: "2026-11-11T17:01:22Z"
updatedAt: "2026-11-11T18:33:35Z"
---

## Description

Requested by Algorant via Herdra m62a9ffc3. His words: "Ok start the review of public repo 2. make sure to factor in or try to apply the guidelines we were using and collecting from our pi repo investigation. tandem is basically already public so much of it probably won't apply as far as 'ready to release' but definitely as far as cleaning up things or simplifying things and having appropriate look/format, yes."

Review scope (read-only until approved):
1. README and public docs against the repo style rules (lowercase names, first-person voice, short showroom tone, short "How to use this repo", no sentence under headings, hierarchy not a pile, no full stops on list items, banner leading the page with no H1, License last as `## License` → `[MIT](LICENSE)`, etc.), using github.com/Algorant/pi's README as the worked example.
2. Plain MIT LICENSE file; main pushed; nothing stale on the public face.
3. Cleanup and simplification: cruft, stale or deprecated files and scripts, docs that restate each other, anything that does not earn its place, with evidence per removal.
4. Banner proposal in the pi banner's family (~/.pi docs/banner.svg).

Deliverable: one Sideshow post in a new session "Public repos: tandem" with the current README rendered as GitHub shows it, the proposed README rendered the same way, the cleanup list with evidence, and a recommendation. gitguard scans the repo separately; no history rewrite.
