---
id: task-101
uid: 99fa1bfb-098d-48f9-85f5-20023e8f9dff
type: task
title: "Update x1nano's installed Tandem when it reconnects"
state: "in-progress"
priority: "low"
effort: "small"
references: ["task-100"]
tags: ["config"]
accord:
  status: "delivered"
  acceptance: ["When x1nano reconnects, update its existing mise-managed github:Algorant/tandem latest install without restarting running TUIs or cores.", "Verify the actual latest release selected executable and PATH command version, and its SHA-256 against the current verified release artifact; record the exact installed version."]
  claimedAt: "2026-11-08T05:53:36Z"
  deliveredAt: "2026-11-08T05:54:12Z"
  summary: "On reconnect, upgraded x1nano's existing mise-managed Tandem latest install from0.17.4 to0.17.5 and verified its PATH shim and actual selected executable against the current release. No core/TUI restart or configuration/visibility change."
  evidence: ["GitHub latest was stable non-draft tandem-v0.17.5 before update. Existing `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem` reported only Tandem upgraded0.17.4→0.17.5; exited0.", "On x1nano, PATH command `~/.local/share/mise/shims/tandem` and actual `mise which tandem` executable `~/.local/share/mise/installs/github-algorant-tandem/0.17.5/tandem` both returned tandem0.17.5.", "Actual executable SHA256 bfcd6248ca9fb6f270a42feb4d3757fc975a7c79aa49a64efda0c77a5f786bee matches the already verified0.17.5 release archive binary and desktop/homelab installs.", "Native /proc snapshot found no Tandem TUI/web process before or after update. No core, TUI, Pi restart/reload or board-state edits were performed. needs-x1nano tag removed before completion."]
  updatedAt: "2026-11-08T05:54:12Z"
createdAt: "2026-11-08T05:24:49Z"
updatedAt: "2026-11-08T05:54:12Z"
assignee: "tandem"
---

## Description

Queued by Herdra mfe005aa9 after the approved 0.17.5 release/update: x1nano was unreachable (SSH connection timeout), so no update was attempted. This is the owning Tandem card, not a Herdra mirror.

Reconnect trigger: needs-x1nano workflow notifies the Tandem Ceph when available. At that point use the existing owner-managed latest path: `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem`; verify PATH shim/command and `mise which tandem` actual executable version/hash against the then-current published latest release. Do not pin 0.17.5 if a newer verified latest exists. No new install path, core restart, TUI restart, desktop/config change or visibility change. Leave the card unstarted until the machine is reachable.
