---
id: task-91
uid: 4be7369b-f79d-419d-8f66-98a0da36b402
type: task
kind: "papercut"
title: "Bun dry-run with lockfile-only unexpectedly writes the site lock"
priority: "low"
effort: "small"
relatedFiles: ["site/bun.lock", "tandem/RELEASE.md"]
accord:
  status: "ready"
  acceptance: ["Dependency inspection guidance uses a verified no-write approach and explicitly avoids the observed mutating dry-run/lockfile-only combination; any repro is disposable and removed afterward."]
  updatedAt: "2026-10-21T02:36:38Z"
createdAt: "2026-10-21T02:36:38Z"
updatedAt: "2026-11-08T03:00:35Z"
archivedAt: "2026-11-08T03:00:35Z"
resolution:
  outcome: "canceled"
  note: "Already moved on: since filing against Bun 1.4.3, Bun 1.4.4 is installed and the site lock refreshed in 0a35c54 (0.17.4 release audit). In a disposable copy of current package.json/bun.lock, `bun update vite --dry-run --lockfile-only` reported an actual available update (8.3.4 → 8.3.6) but both SHA-256 hashes remained unchanged; no Saved lockfile message. `bun outdated` and `bun audit --json` also left both hashes unchanged. The observed bug is not reproducible on current tooling; no Bun update campaign or stale prohibition needed. Temporary repro removed after verification."
---

## Description

Observed during0.17.3 release with Bun1.4.3: `bun update vite --dry-run --lockfile-only` said one package WOULD update, then Saved bun.lock, actually changing62 lines. Plain dry-run had not written it. This contradicted CLI help's 'without making changes'. The subsequent Vite lock refresh was explicitly authorized by Herdra and passed build/links/highaudit; no audit/release gate was bypassed. Capture only, not permission for release-machinery changes or an unrelated Bun update campaign.
