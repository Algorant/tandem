---
id: task-27
type: task
title: "Audit release cycle for shorten/remove opportunities"
state: todo
tags: ["docs", "validation"]
accord:
  status: "ready"
  acceptance: ["Current release cycle documented step-by-step with time/effort per step", "Concrete list of shorten/remove/automate candidates with rationale and risk", "Recommended streamlined release flow with next actions"]
  updatedAt: "2026-09-09T15:32:04Z"
createdAt: "2026-09-09T15:32:04Z"
updatedAt: "2026-09-11T14:24:35Z"
references: ["task-1", "task-31"]
relatedFiles: ["justfile", "tandem/RELEASE.md", "scripts/release_checks.sh", ".github/workflows/aur-tandem-bin.yml"]
---
## Goal
Audit the end-to-end release cycle: enumerate each step, measure or estimate time/effort, and identify bounded shorten/remove/automate opportunities with rationale and risk. Output recommendations before changing release automation.

## Concrete follow-up from the papercut audit
Task-1 is resolved: AUR currently publishes tandem-bin0.13.1-1 and GitHub AUR workflow34518338245 succeeded. The just release recipe and tandem/RELEASE.md still describe AUR as read-only and skip its verification. Audit and recommend correcting these stale assumptions, keeping AUR failures non-blocking under the standing project policy. Do not create another package-availability papercut for this obsolete warning.

Task-31 records the0.13.1 release checks and the docs security-audit retry. Existing tandem/RELEASE.md performance measurements are a baseline, not proof that the current end-to-end audit is complete. Broader automation edits remain outside the current papercut implementation wave.