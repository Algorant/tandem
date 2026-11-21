---
id: task-117
uid: 85e5db09-7084-405e-b50c-b1bd012ce788
type: task
kind: "research"
title: "Decide whether Tandem web earns its place"
state: "in-progress"
priority: "medium"
effort: "medium"
references: ["task-75", "task-116"]
tags: ["ui", "web"]
accord:
  status: "claimed"
  acceptance: ["Explain what web shows today, evidence of use (or its limits), and concrete maintenance cost.", "Compare actual overlap with Herdra and Sideshow windows in Herdr, browser Sideshow, tandem tui, and Herdra's conversation list above the Pi editor; name any genuinely unique web use.", "Give a clear keep-or-deprecate verdict in a visual-first Sideshow report: keep means one or two concrete moments in Algorant’s day plus a rough screen mockup; deprecate means what stopping entails for code, docs, links and AUR/packaging, plus what would be lost.", "Handoff to Tandem Ceph for Herdra review; no implementation or public changes."]
  claimedAt: "2026-11-21T03:08:52Z"
  constraints: ["Research begins only after Herdra releases the manus on Astra high.", "No builds, code changes, redesign implementation, removal, live app changes, publication, or automated captures.", "Preserve parked work and unrelated holds. Ask Herdra or Tandem Ceph for clarification if needed; do not ask Algorant to diagnose."]
  updatedAt: "2026-11-21T03:08:52Z"
createdAt: "2026-11-21T03:08:05Z"
updatedAt: "2026-11-21T03:08:52Z"
assignee: "pi:tandem-web-usefulness-research"
---

## Description

Algorant, verbatim (2026-11-21, via Herdra m2a567ca6): “I don't really care that muhc about tandem web. its just ugly and unused right now and I wanted to dedicate some time to it. can we have the tandem ceph use astra on high thinking as a research task and help me come up with a robust use for how a tandem web view might be useful to my workflow? if it can't, then we should probably deprecate the feature and stop using it or servicing it.” And: “please make sure the astra on high agent has full knowledge of what we discussed and make sure it can ask clarifying questions of you if needed.”

Herdra's confirmed framing: decide whether the web view should exist at all, not how to make it prettier. This research supersedes the deferred orchestration redesign (task-75); parked styling (task-116) and its local draft are not to be resumed or used as a design commitment. Earlier web image removal from the README was deliberate; do not restore it. Keep the current live app and published surfaces untouched.

Start the manus with its first turn idle; Herdra switches it to openai/gpt-6.2-astra at high thinking and releases it before research. The manus may send clarifications directly to Herdra (terminal herdra) or to the Tandem Ceph, not to Algorant for diagnosis. Results go to Tandem Ceph, who sends verdict and Sideshow link to Herdra for review before Algorant sees it.

Open with the verdict and a visual, keep detail below, label facts Observed or Inferred, distinguish lack of evidence from no use, and use highest quality for any images. Research only: no building, redesign implementation, removal, releases or public changes.
