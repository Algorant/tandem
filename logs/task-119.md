---
id: task-119
uid: fc2a40e5-d87b-419b-b374-7532bf5c8687
type: task
title: "Remove AUR support"
priority: "medium"
effort: "medium"
relatedFiles: [".github/workflows/aur-tandem-bin.yml", "justfile", "scripts/release_checks.sh", "scripts/tests/", "tandem/AUR.md", "tandem/RELEASE.md", "tandem/README.md", "AGENTS.md", "docs/"]
tags: ["release", "docs"]
accord:
  status: "accepted"
  acceptance: ["Every AUR piece is gone from the repo and the release path: workflow, AUR guide, release-stage AUR wait, release-check role and fixtures, checklist/audit/validation text, README/AGENTS pointers; no dead references remain.", "Install docs present the shell script first and then the verified mise line, with no AUR or other package-manager guidance left; site build and link checks pass.", "The existing tandem-bin package's fate is decided and recorded (leave, deprecate with a note, or request deletion) with any action that needs Algorant's AUR account stated plainly."]
  deliveredAt: "2026-11-21T04:21:07Z"
  summary: "AUR support is removed from the repo and release path and the install docs now say shell script then mise; the AUD package stays frozen at 0.19.0 as approved."
  evidence: ["Commit 4ba1f46 on main removes the AUR workflow and AUR.md, the recipe's AUR stage, the aur release-check role and fixtures, the RELEASE.md AUR rows/commands, and the README/AGENTS pointers; RELEASES.md history untouched.", "Release-check workflow tests and dev-check (12 test suites, strict clippy, Git-backed smoke) passed; site build and 171 built-site links passed; git diff --check clean.", "Docs deploy 44865542422 succeeded; live /quick-start/ shows the shell script first then `mise use -g \"github:Algorant/tandem@latest\"` and the MISE_USE_VERSIONS_HOST update line, with no AUR guidance. Live / returns 200.", "AUR tandem-bin is untouched and stays published at 0.19.0; no credentials or secrets were read or altered. The two AUR repo secrets remain for Herdra to raise with Algorant."]
  updatedAt: "2026-11-21T04:21:12Z"
createdAt: "2026-11-21T04:16:49Z"
updatedAt: "2026-11-21T04:21:12Z"
archivedAt: "2026-11-21T04:21:12Z"
resolution:
  outcome: "completed"
  note: "AUR support removed in 4ba1f46; deploy 44865542422 live with the mise install line and no AUR guidance; tandem-bin left frozen at 0.19.0 as approved. AUR repo secrets untouched, pending Algorant."
---

## Description

Algorant, verbatim (2026-11-21, via Herdra m4730d23e): "we can remove AUR support from tandem. it will just keep failing and I don't really use the AUR anymore. make a plan to deprecate and remove this, and suggest that people use mise as a package manager for installing it if they don't want to use the shell script."

Decision: stop maintaining AUR support. Plan first; no edits until the plan is agreed. Replacement install guidance: the shell script first, then mise. No other package managers.

## Plan (for approval)

### 1. Repo pieces to remove
- `.github/workflows/aur-tandem-bin.yml` — the whole workflow (build deps, AUR SSH key, PKGBUILD generation, push). Removing it also makes c2449ff's `pacman -Syu` fix moot.
- `tandem/AUR.md` — the package/recovery guide.
- `justfile` — the release recipe's AUR stage: `wait_for_workflow aur`, the `aur_status` branch and the WARNING text.
- `scripts/release_checks.sh` — the `aur` workflow role; `scripts/tests/fixtures/workflow-runs.json` and `scripts/tests/test_release_checks.sh` lose their `aur` cases.
- `tandem/RELEASE.md` — checklist step 5's AUR wait/report, the audit-table AUR row, the "restore AUR verification" candidate line, the AUR comment in the validation block and the `gh run list --workflow "Update tandem-bin AUR package"` line, plus the packaging paragraph.
- `tandem/README.md` — the `AUR.md` pointer; `AGENTS.md` — the layout line's `AUR.md`.
- Historical `RELEASES.md` sections stay truthful and unchanged.
- Docs/site: current AUR mentions are only in those internal files; quick-start offers installer + cargo and gains the mise line, and nothing loses an AUR link.

### 2. The existing `tandem-bin` AUR package
- Leave published as-is: freezes at 0.19.0, needs nothing from him; users see a stale version with no warning.
- Deprecation note: one line saying it is unmaintained, with the script and mise as alternatives. Needs his AUR account (maintainer of `tandem-bin`).
- Request deletion: file an AUR deletion request; a TU removes it. Needs his AUR account/mailing-list identity, and anyone installed via AUR must reinstall with the script or mise.

### 3. Replacement install docs
Order: shell script, then mise.
Shell script (unchanged): `curl -fsSL https://trytandem.dev/install.sh | sh`.
mise: `mise use -g "github:Algorant/tandem@latest"`, then `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem` to update promptly. Dry-run here resolves 0.19.0 for `@latest`; a bare `mise use -g github:Algorant/tandem` resolved stale 0.17.7, so the `@latest` form is the one to document. Cargo stays the source-build option.

### 4. Shipping
These are CI and documentation changes, so they take effect on push with the normal docs deploy; no binary behaviour change, so no version bump is required. A `0.19.1` patch is optional if he wants a marker.

### 5. Out of scope
No other package managers, no renaming, no board/protocol change, no touching the 0.18.0 SSH reservation, no revival of the parked web work.
