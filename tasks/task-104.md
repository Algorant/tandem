---
id: task-104
uid: d5559ea8-566d-482c-bd57-bbdfb1481807
type: task
kind: "papercut"
title: "AUR packaging job cannot install dependencies after Arch's openssl 4.1.0 update"
state: "in-progress"
priority: "low"
tags: ["release"]
accord:
  status: "delivered"
  acceptance: ["The AUR workflow completes for a released tag, regenerating PKGBUILD/.SRCINFO and pushing tandem-bin, with the dependency installation step fixed in a way verified in a real Arch container."]
  claimedAt: "2026-11-21T04:15:13Z"
  deliveredAt: "2026-11-21T04:15:31Z"
  summary: "AUR packaging is fixed: the workflow upgrades the Arch container before installing tools, so openssl transitions cleanly; the real Arch container run succeeded and tandem-bin 0.19.0 is published."
  evidence: ["Failure cause from run44861347249: the container's partial `pacman -Sy` pulled openssl4.1.0 against image libraries still requiring libcrypto.so=3-64, aborting the transaction before packaging.", ".github/workflows/aur-tandem-bin.yml now runs `pacman -Syu --noconfirm --needed ...` with a comment; c2449ff on main. The real archlinux:base-devel container run44865050319 succeeded (openssl upgraded, dependencies installed, PKGBUILD generated and pushed).", "AUR git for tandem-bin publishes pkgver0.19.0 in PKGBUILD and .SRCINFO via cgit. AUR RPC still served cached0.17.5 immediately after the push."]
  updatedAt: "2026-11-21T04:15:31Z"
createdAt: "2026-11-10T15:59:50Z"
updatedAt: "2026-11-21T04:15:31Z"
assignee: "pi:tandem"
---

## Description

The 0.17.6 release verified on GitHub, but the `Update tandem-bin AUR package` workflow failed (run 42850013273) before packaging.

Failure is host-level, not Tandem code: the job's `archlinux:base-devel` container ran `pacman -Sy --noconfirm --needed git github-cli openssh jq sudo`, and the transaction aborted:

```
:: installing openssl (4.1.0-1) breaks dependency 'libcrypto.so=3-64' required by cryptsetup/curl/libarchive/libldap/libngtcp2/libsasl/pacman
```

A partial `-Sy` install pulls openssl 4.1.0 while the image's installed packages still link openssl 3. Per never-5 the verified GitHub Release stands; AUR tandem-bin remains at the previous version until fixed.

Candidate fix (unverified here: no accessible container runtime on this machine, so it was not shipped): upgrade the container consistently, e.g. `pacman -Syu --noconfirm --needed git github-cli openssh jq sudo`, then re-run just the AUR workflow for tandem-v0.17.6 to confirm PKGBUILD/.SRCINFO regeneration and the AUR push. Do not pin the base image. Verify in a real Arch container before merging.
