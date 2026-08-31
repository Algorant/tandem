---
id: task-114
type: task
title: "Fix cargo-dist release build profile configuration"
priority: "high"
parentId: "task-108"
relatedFiles: ["dist-workspace.toml", ".github/workflows/release.yml", "tandem/Cargo.toml", "tandem/RELEASE.md"]
tags: ["config", "release", "cargo-dist", "bugfix"]
createdAt: "2026-07-07T01:48:22Z"
updatedAt: "2026-07-07T01:52:38Z"
accord:
  status: "accepted"
  assignee: "shep"
  claimedAt: "2026-07-07T01:48:36Z"
  deliveredAt: "2026-07-07T01:52:06Z"
  deliverables: ["Added `[profile.dist]` with `inherits = \"release\"` and `lto = \"thin\"` to `tandem/Cargo.toml`.", "Updated `tandem/RELEASE.md` release command to 0.4.2 and added guidance for deleting/retrying a failed tag when no GitHub Release/assets were created.", "Worker branch/worktree: shep/task-114-fix-cargo-dist-release-build-profile-con at /home/ivan/.pi/agent/worktrees/tandem/task-114-fix-cargo-dist-release-build-profile-con.", "Commit: 87f6c23 Fix cargo-dist dist profile configuration."]
  validation:
    commands: ["cargo build --manifest-path tandem/Cargo.toml --profile dist: passed.", "dist generate --mode=ci --check: passed.", "dist plan --output-format=json: passed.", "dist build --artifacts=local --target x86_64-unknown-linux-gnu --output-format=json: passed and built Linux artifact/checksum with cargo profile dist.", "cargo fmt --manifest-path tandem/Cargo.toml --check: passed.", "git diff --check: passed.", "git status --short in worker worktree: clean."]
  summary: "Accepted task-114: cargo-dist release profile is now configured in tandem/Cargo.toml and validated with targeted cargo-dist build; recovery docs added. Integrated on main as commit 65187eb."
  evidence: ["Reviewed commit/diff from worktree branch shep/task-114-fix-cargo-dist-release-build-profile-con.", "Commit 87f6c23."]
  filesChanged: ["tandem/Cargo.toml", "tandem/RELEASE.md"]
  updatedAt: "2026-07-07T01:52:26Z"
completedAt: "2026-07-07T01:52:38Z"
completion:
  summary: "Fixed cargo-dist release build profile configuration and documented failed-tag recovery."
  validation: "Reviewed worker commit 87f6c23 and cherry-picked to main as 65187eb. Validations: cargo build --manifest-path tandem/Cargo.toml --profile dist; dist generate --mode=ci --check; dist plan --output-format=json; dist build --artifacts=local --target x86_64-unknown-linux-gnu --output-format=json; cargo fmt --manifest-path tandem/Cargo.toml --check; git diff --check. All passed."
  reviewer: "pi"
---

## Description

The first cargo-dist release attempt for `tandem-v0.4.2` failed in all platform build jobs.

Observed failure from GitHub Actions Release run 28835466975:

```text
error: profile `dist` is not defined
× failed to find bin tandem for path+file:///.../tandem/tandem#0.4.2
```

Context:
- task-109 added `dist-workspace.toml` with `members = ["cargo:tandem"]` and generated `.github/workflows/release.yml`.
- cargo-dist generated build commands use Cargo profile `dist` and `--workspace`.
- Tandem is a monorepo with Rust crate under `tandem/Cargo.toml` and no root Cargo workspace.
- Need to configure the required cargo-dist Cargo profile in the correct location for this layout, likely under `tandem/Cargo.toml` or another cargo-dist-supported config path.
- No GitHub Release object was created for `tandem-v0.4.2`; the remote tag exists and may need delete/retry guidance after the fix.

Acceptance criteria:
- Add the required cargo-dist/Cargo profile configuration so CI build jobs no longer fail with `profile dist is not defined`.
- Keep configuration compatible with monorepo layout and cargo-dist `cargo:tandem` member.
- Regenerate/check cargo-dist config/workflow if needed.
- Validate locally as far as practical, e.g. cargo-dist generate/check/plan and targeted cargo build using `--profile dist` for the tandem crate.
- Update release docs/checklist if needed, including recovery steps for the failed `tandem-v0.4.2` tag attempt.
- Do not cut a new release unless explicitly asked.
