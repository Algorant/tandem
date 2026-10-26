---
id: task-88
uid: 46aebb1d-3297-4a8f-a81f-8ca95f3a6dd2
type: task
kind: "papercut"
title: "mise picks up a new Tandem release late through its versions host"
state: todo
priority: "low"
tags: ["docs", "release"]
accord:
  status: "ready"
  acceptance: ["tandem/RELEASE.md's post-release steps (or the release notes' upgrade guidance) tell mise users how to pick up a just-published release immediately, e.g. MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem, and to confirm with tandem --version"]
  updatedAt: "2026-10-19T18:37:51Z"
createdAt: "2026-10-19T18:37:51Z"
updatedAt: "2026-10-26T04:00:50Z"
---

## Description

Right after `just release 0.17.2` verified the GitHub Release, `mise up github:Algorant/tandem` on another workstation still resolved `latest` to 0.17.1: mise's versions host (mise-versions.jdx.dev /releases/latest) served the stale tag even with MISE_FETCH_REMOTE_VERSIONS_CACHE=0. `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem` installed 0.17.2 immediately. Coordinated "upgrade together" releases depend on every machine actually getting the new version.
