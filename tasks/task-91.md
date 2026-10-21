---
id: task-91
uid: 4be7369b-f79d-419d-8f66-98a0da36b402
type: task
kind: "papercut"
title: "Bun dry-run with lockfile-only unexpectedly writes the site lock"
state: todo
priority: "low"
effort: "small"
relatedFiles: ["site/bun.lock", "tandem/RELEASE.md"]
accord:
  status: "ready"
  acceptance: ["Dependency inspection guidance uses a verified no-write approach and explicitly avoids the observed mutating dry-run/lockfile-only combination; any repro is disposable and removed afterward."]
  updatedAt: "2026-10-21T02:36:38Z"
createdAt: "2026-10-21T02:36:38Z"
updatedAt: "2026-10-21T02:36:38Z"
---

## Description

Observed during0.17.3 release with Bun1.4.3: `bun update vite --dry-run --lockfile-only` said one package WOULD update, then Saved bun.lock, actually changing62 lines. Plain dry-run had not written it. This contradicted CLI help's 'without making changes'. The subsequent Vite lock refresh was explicitly authorized by Herdra and passed build/links/highaudit; no audit/release gate was bypassed. Capture only, not permission for release-machinery changes or an unrelated Bun update campaign.
