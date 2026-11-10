---
id: task-104
uid: d5559ea8-566d-482c-bd57-bbdfb1481807
type: task
kind: "papercut"
title: "AUR packaging job cannot install dependencies after Arch's openssl 4.1.0 update"
state: todo
priority: "low"
tags: ["release"]
accord:
  status: "ready"
  acceptance: ["The AUR workflow completes for a released tag, regenerating PKGBUILD/.SRCINFO and pushing tandem-bin, with the dependency installation step fixed in a way verified in a real Arch container."]
  updatedAt: "2026-11-10T15:59:50Z"
createdAt: "2026-11-10T15:59:50Z"
updatedAt: "2026-11-10T15:59:50Z"
---

## Description

The 0.17.6 release verified on GitHub, but the `Update tandem-bin AUR package` workflow failed (run 42850013273) before packaging.

Failure is host-level, not Tandem code: the job's `archlinux:base-devel` container ran `pacman -Sy --noconfirm --needed git github-cli openssh jq sudo`, and the transaction aborted:

```
:: installing openssl (4.1.0-1) breaks dependency 'libcrypto.so=3-64' required by cryptsetup/curl/libarchive/libldap/libngtcp2/libsasl/pacman
```

A partial `-Sy` install pulls openssl 4.1.0 while the image's installed packages still link openssl 3. Per never-5 the verified GitHub Release stands; AUR tandem-bin remains at the previous version until fixed.

Candidate fix (unverified here: no accessible container runtime on this machine, so it was not shipped): upgrade the container consistently, e.g. `pacman -Syu --noconfirm --needed git github-cli openssh jq sudo`, then re-run just the AUR workflow for tandem-v0.17.6 to confirm PKGBUILD/.SRCINFO regeneration and the AUR push. Do not pin the base image. Verify in a real Arch container before merging.
