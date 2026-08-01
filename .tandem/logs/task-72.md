---
id: task-72
type: task
title: "Investigate concise GitHub Release notes generation for Tandem releases"
priority: "medium"
relatedFiles: ["justfile", "tandem/RELEASE.md"]
tags: ["release", "docs", "github", "changelog"]
createdAt: "2026-06-30T12:03:37Z"
updatedAt: "2026-06-30T19:39:19Z"
subtasks:
  - id: task-72-1
    title: "Inspect current release recipe and GitHub Release note source"
    completed: false
  - id: task-72-2
    title: "Research GitHub CLI release note options and generated-notes behavior"
    completed: false
  - id: task-72-3
    title: "Recommend a concise release-note format and source of truth"
    completed: false
  - id: task-72-4
    title: "Identify how to preserve detailed validation/install checklists separately"
    completed: false
accord:
  status: "accepted"
  assignee: "shep:task-72-release-notes"
  claimedAt: "2026-06-30T19:21:17Z"
  deliveredAt: "2026-06-30T19:27:40Z"
  deliverables: ["Created tandem/GITHUB_RELEASE_NOTES.md as concise public release-note body.", "Updated tandem/RELEASE.md to act as reusable release checklist and compare release-note workflow options.", "Updated just release to use tandem/GITHUB_RELEASE_NOTES.md for gh release create notes file.", "Updated tandem/README.md to point to the release notes/checklist split."]
  validation:
    commands: ["just --list passed", "Python smoke for release/checklist version substitution passed", "git diff --check passed"]
  summary: "Accepted objective release workflow changes after justfile parsing and release-note substitution smoke passed."
  evidence: ["Commit 3df3cbf splits public GitHub release notes from reusable release checklist."]
  filesChanged: ["justfile", "tandem/RELEASE.md", "tandem/GITHUB_RELEASE_NOTES.md", "tandem/README.md"]
  reviewer: "orchestrator"
  updatedAt: "2026-06-30T19:39:14Z"
completedAt: "2026-06-30T19:39:19Z"
completion:
  summary: "Split concise public GitHub Release notes into tandem/GITHUB_RELEASE_NOTES.md and kept tandem/RELEASE.md as reusable release checklist; updated just release to publish from the concise notes file."
  validation: "just --list passed; Python version-substitution smoke passed; git diff --check passed; committed as 3df3cbf."
  reviewer: "orchestrator"
---

## Description

Investigate how Tandem should update GitHub Release notes during release updates so each release has concise, useful, version-specific notes instead of copying the current verbose generic boilerplate from tandem/RELEASE.md. Compare options such as curated per-release notes, generated changelogs from commits/tags, GitHub auto-generated release notes, and maintaining a separate reusable release checklist versus public release notes. Recommend a release-note workflow that keeps install/validation details available without making the GitHub Release body noisy.
