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
  status: "ready"
  acceptance: ["README, docs and banner are reviewed against Algorant's repo style rules (global memory note repo-style-rules.md) with the pi README as the worked example, and a proposed README plus banner are shown rendered as GitHub shows them.", "Cleanup and simplification findings name concrete files or sections with evidence for removal, and clearly separate what applies to an already-public repo from release-readiness checks.", "One Sideshow post in a new session presents current README, proposed README, cleanup list with evidence and a recommendation; nothing in the repository is changed before Algorant approves."]
  updatedAt: "2026-11-11T17:01:22Z"
createdAt: "2026-11-11T17:01:22Z"
updatedAt: "2026-11-11T17:01:22Z"
---

## Description

Requested by Algorant via Herdra m62a9ffc3. His words: "Ok start the review of public repo 2. make sure to factor in or try to apply the guidelines we were using and collecting from our pi repo investigation. tandem is basically already public so much of it probably won't apply as far as 'ready to release' but definitely as far as cleaning up things or simplifying things and having appropriate look/format, yes."

Review scope (read-only until approved):
1. README and public docs against the repo style rules (lowercase names, first-person voice, short showroom tone, short "How to use this repo", no sentence under headings, hierarchy not a pile, no full stops on list items, banner leading the page with no H1, License last as `## License` → `[MIT](LICENSE)`, etc.), using github.com/Algorant/pi's README as the worked example.
2. Plain MIT LICENSE file; main pushed; nothing stale on the public face.
3. Cleanup and simplification: cruft, stale or deprecated files and scripts, docs that restate each other, anything that does not earn its place, with evidence per removal.
4. Banner proposal in the pi banner's family (~/.pi docs/banner.svg).

Deliverable: one Sideshow post in a new session "Public repos: tandem" with the current README rendered as GitHub shows it, the proposed README rendered the same way, the cleanup list with evidence, and a recommendation. gitguard scans the repo separately; no history rewrite.
