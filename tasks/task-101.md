---
id: task-101
uid: 99fa1bfb-098d-48f9-85f5-20023e8f9dff
type: task
title: "Update x1nano's installed Tandem when it reconnects"
state: todo
priority: "low"
effort: "small"
references: ["task-100"]
tags: ["config", "needs-x1nano"]
accord:
  status: "ready"
  acceptance: ["When x1nano reconnects, update its existing mise-managed github:Algorant/tandem latest install without restarting running TUIs or cores.", "Verify the actual latest release selected executable and PATH command version, and its SHA-256 against the current verified release artifact; record the exact installed version."]
  updatedAt: "2026-11-08T05:24:49Z"
createdAt: "2026-11-08T05:24:49Z"
updatedAt: "2026-11-08T05:24:49Z"
---

## Description

Queued by Herdra mfe005aa9 after the approved 0.17.5 release/update: x1nano was unreachable (SSH connection timeout), so no update was attempted. This is the owning Tandem card, not a Herdra mirror.

Reconnect trigger: needs-x1nano workflow notifies the Tandem Ceph when available. At that point use the existing owner-managed latest path: `MISE_USE_VERSIONS_HOST=0 mise up github:Algorant/tandem`; verify PATH shim/command and `mise which tandem` actual executable version/hash against the then-current published latest release. Do not pin 0.17.5 if a newer verified latest exists. No new install path, core restart, TUI restart, desktop/config change or visibility change. Leave the card unstarted until the machine is reachable.
