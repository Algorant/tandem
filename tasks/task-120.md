---
id: task-120
uid: 53e9bc83-e4bf-4dfe-ad92-0710d149ae6d
type: task
kind: "papercut"
title: "Appending to a long Task body breaks read-then-write"
state: todo
priority: "low"
effort: "small"
relatedFiles: ["tandem/src/cli/", "tandem/src/app/", "protocol/README.md", "docs/cli/index.md"]
tags: ["cli", "accord"]
accord:
  status: "ready"
  acceptance: ["A CLI operation appends to a Task or Decision body without the caller reading, concatenating and rewriting the whole body, and its JSON result reports the resulting record and sync status.", "Long-body appends are verified end-to-end on a disposable Git-backed board, including a body large enough that reading it back is impractical for an agent, with no truncation or lost content."]
  updatedAt: "2026-11-23T02:25:01Z"
createdAt: "2026-11-23T02:25:01Z"
updatedAt: "2026-11-23T02:25:01Z"
---

## Description

## Description

Reported by Herdra m5b712e07, from algOS task-181 (hit again on algOS task-169 and during board triage) and from Herdra's own tasks 208/177/165/212.

Agents repeatedly need to add a section to a Task or Decision that already has a long body. The only path today is: read the whole body, concatenate the new text, then send the whole body back through `tandem update --body`. That breaks down at scale:

- the read is what the tool truncates first (file reads cap at 2,000 lines or 50 KB), so the agent cannot reliably reconstruct the body;
- the fallback becomes shelling out to bash with saved full output, which bypasses the board tools and risks writing a body that dropped content;
- each round trip moves the entire body for a few appended lines.

## Desired outcome
A first-class append that does not require the caller to hold the existing body, plus a documented body-size rule so predictable behaviour is possible: e.g. `tandem update <id> --append <text|@file>` (and the JSON envelope reporting the record and sync status), or an equivalent documented operation. Preserve every existing byte of the body, keep frontmatter untouched, and do not rewrite archived records.

## Notes
- Observed in the Tandem worktree during this board's own triage: appending a triage section required the read-modify-write fallback.
- Not a redesign of records or of `update`'s existing field semantics; the smallest operation that makes appending safe.
- Capture-equivalent scope: no implementation until triaged/authorized.

## Triage 2026-11-21
Verdict: refine-scope then park. The intent is clear and small enough to be one bounded change, but two things must be settled first: whether append lives on `update` or beside it, and the exact body-size rule (what a caller may rely on before it must use append). A documented append is a protocol-visible CLI contract change, so it needs the normative description in `protocol/README.md` with the code.

Re-check trigger: the next release that touches `update`/CLI contract work, or Algorant authorizes it directly. Not on his decide list yet.
