---
id: task-117
uid: 85e5db09-7084-405e-b50c-b1bd012ce788
type: task
kind: "research"
title: "Decide whether Tandem web earns its place"
state: "in-progress"
priority: "medium"
effort: "medium"
references: ["decision-13", "task-118"]
tags: ["ui", "web", "triaged"]
accord:
  status: "delivered"
  acceptance: ["Explain what web shows today, evidence of use (or its limits), and concrete maintenance cost.", "Compare actual overlap with Herdra and Sideshow windows in Herdr, browser Sideshow, tandem tui, and Herdra's conversation list above the Pi editor; name any genuinely unique web use.", "Give a clear keep-or-deprecate verdict in a visual-first Sideshow report: keep means one or two concrete moments in Algorant’s day plus a rough screen mockup; deprecate means what stopping entails for code, docs, links and AUR/packaging, plus what would be lost.", "Handoff to Tandem Ceph for Herdra review; no implementation or public changes."]
  claimedAt: "2026-11-21T03:08:52Z"
  deliveredAt: "2026-11-21T03:13:28Z"
  constraints: ["Research begins only after Herdra releases the manus on Astra high.", "No builds, code changes, redesign implementation, removal, live app changes, publication, or automated captures.", "Preserve parked work and unrelated holds. Ask Herdra or Tandem Ceph for clarification if needed; do not ask Algorant to diagnose."]
  summary: "Recommend deprecating Tandem web, pending Algorant’s judgement after Herdra review. The whole-board local read-only browser is unique, but no recurring dependent job is known; no implementation is authorized or done."
  evidence: ["Visual-first Sideshow review: http://desktop.tail1cefc.ts.net:8228/session/jLMkuE8ueiY (also http://localhost:8228/session/jLMkuE8ueiY). Post read-back confirmed the decision and rendered diagram. Both workspace URLs returned HTTP 200.", "Checked all five comparison surfaces against Tandem 7b9753b and Herdra 61b586f. Ceph knew of no recurring reader, saved link or HTTP client; this is not telemetry. Release notes 0.10.3, kind lanes and rules changes demonstrate real upkeep.", "Independently verified 2,321 dedicated lines including tests, 9 API routes, 15 in-module tests, three browser scripts on the release path, the /web/ docs redirect, and one shared release/AUR binary. Conditional retirement and loss detailed in the report.", "No runnable Task validations. tandem web --help, tandem tui --help, real list/show reads and git diff --check succeeded; git status --porcelain was empty. No builds, captures, source changes, releases or public actions."]
  updatedAt: "2026-11-21T03:13:28Z"
createdAt: "2026-11-21T03:08:05Z"
updatedAt: "2026-11-21T03:17:53Z"
assignee: "pi:tandem-web-usefulness-research"
---

## Description

Algorant, verbatim (2026-11-21, via Herdra m2a567ca6): “I don't really care that muhc about tandem web. its just ugly and unused right now and I wanted to dedicate some time to it. can we have the tandem ceph use astra on high thinking as a research task and help me come up with a robust use for how a tandem web view might be useful to my workflow? if it can't, then we should probably deprecate the feature and stop using it or servicing it.” And: “please make sure the astra on high agent has full knowledge of what we discussed and make sure it can ask clarifying questions of you if needed.”

Herdra's confirmed framing: decide whether the web view should exist at all, not how to make it prettier. This research supersedes the deferred orchestration redesign (task-75); parked styling (task-116) and its local draft are not to be resumed or used as a design commitment. Earlier web image removal from the README was deliberate; do not restore it. Keep the current live app and published surfaces untouched.

Start the manus with its first turn idle; Herdra switches it to openai/gpt-6.2-astra at high thinking and releases it before research. The manus may send clarifications directly to Herdra (terminal herdra) or to the Tandem Ceph, not to Algorant for diagnosis. Results go to Tandem Ceph, who sends verdict and Sideshow link to Herdra for review before Algorant sees it.

Open with the verdict and a visual, keep detail below, label facts Observed or Inferred, distinguish lack of evidence from no use, and use highest quality for any images. Research only: no building, redesign implementation, removal, releases or public changes.


## Findings — research recommendation, awaiting judgement

Deprecate the standalone browser reader; do not redesign it. It uniquely offers ad-hoc browser navigation of a complete, automatically refreshed, read-only local board without Herdr or a posted artifact. No recurring human or program was identified that needs it. Algorant’s account and the Tandem Ceph’s known verification/docs uses are not usage counts or proof of zero external readers. Herdra already owns the cross-project ask and live agent context; Sideshow owns presented evidence and feedback; the TUI owns full records and actions.

At Tandem 7b9753b (0.17.7), web.rs plus five embedded assets occupy 2,321 lines including tests; there are nine HTTP read routes, 15 in-module tests and a three-script release check. Prior release 0.10.3 fixed a web performance regression. If separately authorized, announce the eventual command/HTTP break, remove only web-specific code/entry gates, update guides and legacy documentation links, and ship the usual binary. No distinct AUR package, board migration or replacement service. No implementation, release, capture or change to the parked draft.

Visual comparison, conditional removal scope and source citations: http://desktop.tail1cefc.ts.net:8228/session/jLMkuE8ueiY . Checked the saved post and both localhost and MagicDNS session URLs (HTTP 200). No runnable Task validations; installed CLI help, real list/show reads, git diff --check and clean source status verified instead of builds. Research goes to Tandem Ceph for Herdra review before Algorant; no choice has been made on his behalf.