---
id: task-68
uid: 86b15909-5258-4de2-a53a-4aa121650aa9
type: task
kind: "papercut"
title: "just release needs cargo-dist on PATH, but nothing provisions it"
state: "in-progress"
priority: "low"
references: ["task-27"]
tags: ["config"]
accord:
  status: "delivered"
  acceptance: ["`just release` on a fresh machine gets cargo-dist at the version pinned in dist-workspace.toml without a manual install (e.g. a project mise.toml pin or a mise x wrapper in the recipe), or fails up front with a clear message naming the required version."]
  claimedAt: "2026-10-13T14:15:14Z"
  deliveredAt: "2026-10-13T14:15:17Z"
  summary: "Covered by task-27: release reads the existing dist-workspace.toml pin and executes cargo-dist through mise with explicit setup/version preflight errors."
  evidence: ["Verified/covered by accepted task-27 delivery on main bd46f7d.", "Code uses mise x aqua:axodotdev/cargo-dist@<pin> -- dist for version and manifest; pin has one authority and no global config change or alternate fallback.", "Existing pinned cargo-dist 0.32.0 invocation validated; disposable fake git/mise test demonstrated failed provisioning stops before release work and names required version. No publication occurred."]
  filesChanged: ["justfile"]
  updatedAt: "2026-10-13T14:15:17Z"
createdAt: "2026-10-10T14:46:15Z"
updatedAt: "2026-10-13T14:15:17Z"
assignee: "tandem-ceph"
---

## Description

During the 0.16.3 release on desktop, `just release` failed with `dist: command not found` (line 137, `dist manifest`). Neither desktop nor archbox has `dist`. The run worked with `mise x aqua:axodotdev/cargo-dist@0.32.0 -- just release 0.16.3`, which matches `cargo-dist-version = "0.32.0"` in dist-workspace.toml. The audit gate (`bun audit --audit-level=high`) also tripped on new advisories in the same run (sharp, source-map-js, http-cache-semantics), so a stale lockfile is only discovered mid-release.
