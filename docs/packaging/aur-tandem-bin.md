---
title: tandem-bin AUR release automation
description: How Tandem updates the x86_64 tandem-bin AUR package from GitHub Release artifacts.
---
The `Update tandem-bin AUR package` workflow updates the Arch User Repository binary package after Tandem GitHub Release artifacts exist.

## Package scope

- AUR package: `tandem-bin`
- Initial architecture: `x86_64` only
- Source artifact: `tandem-x86_64-unknown-linux-gnu.tar.xz` from the GitHub Release
- Checksum source: the release `sha256.sum` entry for that artifact
- The AUR package installs the published binary; it does not build Tandem from source.

## Required GitHub secrets

- `AUR_SSH_PRIVATE_KEY`: private SSH key with push access to `ssh://aur@aur.archlinux.org/tandem-bin.git`.
- `AUR_KNOWN_HOSTS` (optional but recommended): pinned `aur.archlinux.org` host key line. If omitted, the workflow uses `ssh-keyscan` at runtime.

Do not commit SSH keys, tokens, or generated secret material to this repository or to AUR.

## AUR remote setup

1. Create or adopt the `tandem-bin` package on AUR.
2. Add the public half of `AUR_SSH_PRIVATE_KEY` to the AUR account that maintains `tandem-bin`.
3. Confirm the key can push to:

   ```text
   ssh://aur@aur.archlinux.org/tandem-bin.git
   ```

The workflow clones that remote, replaces `PKGBUILD`, regenerates `.SRCINFO` with `makepkg --printsrcinfo`, commits, and pushes to `master`.

## Triggering behavior

- Automatic: after the `Release` workflow completes successfully for a tag push such as `tandem-v0.4.1`.
- Manual recovery: run `Update tandem-bin AUR package` with the release tag input, for example `tandem-v0.4.1`.

The workflow downloads release assets with `gh release download`, so it fails early if the binary archive or `sha256.sum` does not exist yet.

## Manual recovery steps

If automation fails after a release:

1. Verify the GitHub Release contains `tandem-x86_64-unknown-linux-gnu.tar.xz` and `sha256.sum`.
2. Re-run the workflow manually with the same `tandem-vX.Y.Z` tag.
3. If SSH fails, verify `AUR_SSH_PRIVATE_KEY`, the AUR account public key, and the known-hosts value.
4. If package generation fails, clone `ssh://aur@aur.archlinux.org/tandem-bin.git`, update `PKGBUILD`, run `makepkg --printsrcinfo > .SRCINFO` on Arch, commit both files, and push `master`.
5. Never paste or commit private keys while debugging; rotate the AUR key if it may have been exposed.
