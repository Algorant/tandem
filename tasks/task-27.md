---
id: task-27
uid: 292e96bc-4a30-4c67-90da-02b534755d42
type: task
title: "Streamline release checks and remove release friction"
state: "in-progress"
tags: ["config", "release"]
accord:
  status: "delivered"
  acceptance: ["Current release cycle documented step-by-step with time/effort per step", "Concrete list of shorten/remove/automate candidates with rationale and risk", "Recommended streamlined release flow with next actions", "Strict release-equivalent clippy runs in dev-check; relevant site/build/link/high-audit checks happen before publication.", "just release obtains/executes cargo-dist at the existing dist-workspace.toml pin without manual global setup or an unnamed fallback; required setup errors are clear up front.", "The matching AUR workflow is verified again, with accurate success/failure reporting and nonblocking downstream failure after verified GitHub publication; stale read-only claims are removed."]
  claimedAt: "2026-10-13T13:54:57Z"
  deliveredAt: "2026-10-13T14:11:15Z"
  validation: ["$ just test-release-checks", "$ just dev-check", "$ just site-build", "$ cd site && bun run check:links", "$ cd site && bun audit --audit-level=high"]
  constraints: ["Herdra approved the batch plan; fresh Luna/high Worker must return its own bounded plan before edits.", "No actual release/tag/push/AUR publication in validation; use publication-free checks and temporary stubs.", "No new release machinery, speculative parallel scheduler or global mise config edits; use existing pin and restore matching AUR verification with nonblocking downstream reporting.", "Temporary tests/harnesses only; remove them before delivery. Own tandem/RELEASE.md ahead of docs prose."]
  summary: "Implemented and committed the approved release-friction changes as deaa015. Added strict clippy to dev-check; moved the existing docs build/link/high-audit checks before publication; provisioned cargo-dist through mise at the dist-workspace.toml pin with clear preflight diagnostics; restored matching AUR verification with bounded downstream failure reporting; and updated the maintained release guide with measured versus estimated effort. No real tag, push, GitHub Release, or AUR publication was performed."
  evidence: ["Current release cycle documented step-by-step with time/effort per step; bounded candidates and recommended flow recorded.: tandem/RELEASE.md now distinguishes timings explicitly reported in /tmp/tandem-release-0.17.0.log from operator estimates, does not claim an end-to-end timing total, lists bounded shorten/remove/automate choices with rationale and risk, and gives the recommended next actions.", "Strict release-equivalent clippy runs in dev-check; site/build/link/high-audit checks happen before publication.: scripts/dev.sh runs `cargo clippy --all-targets --all-features -- -D warnings` in check mode. justfile runs site build, `bun run check:links`, and `bun audit --audit-level=high` before `git tag` and `git push`; all required local validations passed.", "just release uses pinned cargo-dist with clear setup errors, and matching AUR verification reports outcomes without invalidating verified GitHub publication.: justfile reads the sole version from dist-workspace.toml, verifies/provisions it with `mise x aqua:axodotdev/cargo-dist@<pin>`, and names the pinned version in missing/setup diagnostics. After published GitHub assets/notes pass validation, it matches and waits for the AUR workflow (5-minute discovery plus 5-minute completion bound), reports success or failure/timeout, and exits successfully on downstream AUR failure. Disposable stub runs verified success and nonblocking failure paths."]
  filesChanged: ["justfile", "scripts/dev.sh", "tandem/RELEASE.md"]
  updatedAt: "2026-10-13T14:11:15Z"
createdAt: "2026-09-09T15:32:04Z"
updatedAt: "2026-10-13T14:11:15Z"
references: ["task-1", "task-31", "task-59", "task-68", "task-80"]
relatedFiles: ["justfile", "scripts/dev.sh", "scripts/release_checks.sh", "tandem/RELEASE.md", "dist-workspace.toml"]
effort: "medium"
priority: "medium"
assignee: "worker-task-27-e6e81b4a"
---
## Approved batch A

Algorant/Herdra approved root task-27 to audit and implement release-friction fixes, covering tasks 59/68/80 under this one Worker. Those covered Tasks remain separately auditable; parent completes them from actual criterion evidence when this delivery lands, not preemptively.

Document the current cycle with measured/estimated step effort (0.17.0 log /tmp/tandem-release-0.17.0.log is evidence), bounded shorten/remove candidates and risks, then implement the signed-off small changes: strict release-equivalent clippy in dev-check; existing site build/links/high-audit gate before publication; one authoritative cargo-dist path through mise using the version already pinned in dist-workspace.toml (no global config changes/fallback strategy); matching AUR verification restored, downstream failure explicitly reported without invalidating an otherwise verified GitHub release. Do not add new release machinery or broad parallel orchestration.

Own justfile, scripts/dev.sh, scripts/release_checks.sh, tandem/RELEASE.md and pin consumption. AUR workflow edits only if an observed defect demands them—escalate plan widening first. Public docs overhaul must not overwrite this maintained release guide. Plan first, Luna/high; no code until parent approval. No publication in tests and no push/release authorized.